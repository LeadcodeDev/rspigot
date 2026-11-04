use std::fmt::Display;

use rspigot_domain::effect::EffectInstance;
use rspigot_domain::player::PlayerId;
use rspigot_domain::{Rotation, Vec3};

use crate::commands::CommandContext;
use crate::events::ClientEvent;
use crate::player::Player;
use crate::plugins::decisions::{
    Asked, Continuation, Held, Lane, MoveStep, Pending, Ready, Settlement, Taken, Undo, Undone,
};
use crate::plugins::events::{PlayerMove, PlayerMovePatch, PlayerTeleportCause};
use crate::plugins::player::SPEED_SCALE;
use crate::plugins::{
    Access, Answer, EffectSpec, EventPatch, EventPayload, FromPlugin, HandlerSpec, Location, Order,
    Phase, PlayerSnapshot, PlayerState, Question, ToPlugin, Verdict,
};
use crate::ports::ClientLink;
use crate::server::Server;

macro_rules! patch_of {
    ($patch:expr, $kind:ident) => {
        match $patch {
            Some(EventPatch::$kind(patch)) => patch,
            _ => Default::default(),
        }
    };
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn next_event_id(&mut self) -> u64 {
        self.plugins.next_event_id()
    }

    pub(crate) fn notify_plugins(
        &mut self,
        id: u64,
        payload: &EventPayload,
        cancelled: bool,
        phase: Phase,
    ) {
        let listeners =
            self.plugins
                .listeners(payload.kind(), Access::Observe, &self.config.plugins);
        let (cancelled, heard) = audience(listeners, payload, cancelled, phase);
        for (connection, handlers) in heard {
            self.plugins.send(
                connection,
                ToPlugin::Event {
                    id,
                    handlers,
                    phase,
                    cancelled,
                    payload: payload.clone(),
                },
            );
        }
    }

    pub(crate) fn notify_owner(&mut self, connection: u64, payload: EventPayload) {
        let owned = self
            .plugins
            .listeners(payload.kind(), Access::Observe, &self.config.plugins)
            .into_iter()
            .filter(|(listener, _)| *listener == connection)
            .collect();
        let (cancelled, heard) = audience(owned, &payload, false, Phase::Settled);
        if let Some((connection, handlers)) = heard.into_iter().next() {
            let id = self.next_event_id();
            self.plugins.send(
                connection,
                ToPlugin::Event {
                    id,
                    handlers,
                    phase: Phase::Settled,
                    cancelled,
                    payload,
                },
            );
        }
    }

    pub(crate) fn snapshot_of(&self, id: PlayerId) -> Option<PlayerSnapshot> {
        self.players.get(&id).map(|player| self.describe(player))
    }

    pub(crate) fn plugin_message(&mut self, connection: u64, message: FromPlugin) {
        if !self.plugins.is_current(connection) {
            return;
        }
        match message {
            FromPlugin::Decision { event, verdicts } => {
                self.take_decision(connection, event, verdicts);
            }
            FromPlugin::Order(order) => self.apply_order(connection, order),
            FromPlugin::Question { request, question } => {
                let answer = self.answer_question(connection, question);
                self.plugins
                    .send(connection, ToPlugin::Answer { request, answer });
            }
        }
    }

    fn apply_order(&mut self, connection: u64, order: Order) {
        match order {
            Order::Player { player, order } => self.apply_player_order(connection, player, order),
            Order::Inventory { player, order } => {
                self.apply_inventory_order(connection, player, order);
            }
            Order::Server(order) => self.apply_server_order(connection, order),
            Order::World(order) => self.apply_world_order(connection, order),
            Order::Block { pos, order } => self.apply_block_order(connection, pos, order),
            Order::Entity { entity, order } => self.apply_entity_order(connection, entity, order),
            Order::CustomInventory { inventory, order } => {
                self.apply_custom_inventory_order(connection, inventory, order);
            }
        }
    }

    pub(super) fn plugin_name(&self, connection: u64) -> String {
        self.plugins.name(connection).unwrap_or_default().to_owned()
    }

    pub(super) fn warn_plugin(&self, connection: u64, order: &str, reason: impl Display) {
        self.plugins.send(
            connection,
            ToPlugin::Warning(format!("{order} was not applied: {reason}")),
        );
    }

    fn answer_question(&mut self, connection: u64, question: Question) -> Answer {
        match question {
            Question::Player(id) => self.state_of(id).map_or(Answer::NotFound, Answer::Player),
            Question::Players => Answer::Players(self.online_snapshots()),
            Question::PlayerNamed(name) => Answer::Players(
                CommandContext::find_player(self, &name)
                    .and_then(|id| self.snapshot_of(id))
                    .into_iter()
                    .collect(),
            ),
            Question::Inventory(id) => self
                .inventory_state(id)
                .map_or(Answer::NotFound, Answer::Inventory),
            Question::TargetBlock {
                player,
                max_distance,
            } => self.target_block(player, max_distance),
            Question::TargetEntity {
                player,
                max_distance,
            } => self.target_entity(player, max_distance),
            Question::Server => Answer::Server(self.server_state()),
            Question::Whitelist => Answer::Names(self.admin.whitelist.clone()),
            Question::Operators => Answer::Names(self.admin.operators.clone()),
            Question::Bans => Answer::Bans(self.ban_entries()),
            Question::IpBans => Answer::IpBans(self.ip_ban_entries()),
            Question::World => Answer::World(self.world_state()),
            Question::Block(pos) => self
                .block_state(pos)
                .map_or(Answer::NotFound, Answer::Block),
            Question::Entities { near, kind } => self
                .listed_entities(near, kind.as_deref())
                .map_or_else(Answer::Refused, Answer::Entities),
            Question::Entity(id) => self
                .entity_details(id)
                .map_or(Answer::NotFound, Answer::Entity),
            Question::Spawn { kind, position } => self
                .spawn_for_plugin(&kind, position)
                .map_or_else(Answer::Refused, |id| self.spawned(id)),
            Question::DropItem { position, item } => self
                .drop_for_plugin(position, &item)
                .map_or_else(Answer::Refused, |id| self.spawned(id)),
            Question::CustomInventory(inventory) => self
                .custom_inventory_state(connection, inventory)
                .map_or(Answer::NotFound, Answer::CustomInventory),
        }
    }

    pub(crate) fn decide_move(
        &mut self,
        id: PlayerId,
        from: Vec3,
        to: Vec3,
        rotation: Option<Rotation>,
        on_ground: bool,
    ) {
        let step = MoveStep {
            player: id,
            from,
            to,
            rotation,
            on_ground,
        };
        self.fire(id, Continuation::PlayerMove(step), |server| {
            server.move_payload(id, from, to, rotation)
        });
    }

    pub(crate) fn fire(
        &mut self,
        lane: impl Into<Lane>,
        then: Continuation,
        describe: impl FnOnce(&Self) -> Option<EventPayload>,
    ) {
        self.fire_as(lane.into(), None, then, describe);
    }

    pub(crate) fn fire_deferred(&mut self, lane: impl Into<Lane>, then: Continuation) {
        let lane = lane.into();
        if self.decisions.busy(lane) {
            let event = self.next_event_id();
            self.decisions.defer(event, lane, then);
            return;
        }
        self.fire_head(lane, None, then);
    }

    fn fire_head(&mut self, lane: Lane, order: Option<u64>, then: Continuation) {
        let Some(then) = self.due(then) else {
            return;
        };
        let kind = then.kind();
        let listened = self.plugins.listens(kind, Access::Observe)
            || self.plugins.listens(kind, Access::Decide);
        let payload = if listened {
            self.due_payload(&then)
        } else {
            None
        };
        self.fire_as(lane, order, then, move |_| payload);
    }

    fn fire_as(
        &mut self,
        lane: Lane,
        order: Option<u64>,
        then: Continuation,
        describe: impl FnOnce(&Self) -> Option<EventPayload>,
    ) {
        let kind = then.kind();
        if then.hot()
            && self
                .decisions
                .in_flight(lane, |queued| queued.same_as(&then))
        {
            return;
        }
        let deciding = self.plugins.listens(kind, Access::Decide);
        if !deciding && !self.decisions.busy(lane) {
            let observed = if self.plugins.listens(kind, Access::Observe) {
                describe(self)
            } else {
                None
            };
            if let Some(payload) = observed {
                let event = order.unwrap_or_else(|| self.next_event_id());
                self.notify_plugins(event, &payload, false, Phase::Settled);
            }
            let dropped = self.conclude(then, false, None, Vec::new());
            self.notify_undone(dropped, Phase::Settled);
            return;
        }
        let Some(payload) = describe(self) else {
            return;
        };
        let event = order.unwrap_or_else(|| self.next_event_id());
        let deciders = if deciding {
            self.plugins
                .listeners(kind, Access::Decide, &self.config.plugins)
        } else {
            Vec::new()
        };
        let asked = deciders
            .iter()
            .map(|(connection, spec)| Asked {
                connection: *connection,
                handler: spec.id,
                kind: spec.event,
            })
            .collect();
        for (connection, handlers) in recipients(deciders, false, Phase::Decide) {
            self.plugins.send(
                connection,
                ToPlugin::Event {
                    id: event,
                    handlers,
                    phase: Phase::Decide,
                    cancelled: false,
                    payload: payload.clone(),
                },
            );
        }
        let arrival = self.current_tick();
        let settings = &self.config.plugins;
        let deadline = arrival
            .saturating_add(settings.decision_ticks)
            .saturating_add(1);
        let until = arrival.saturating_add(settings.late_cancel_ticks);
        self.decisions.queue(
            event,
            Pending::new(lane, payload, asked, deadline, until, then),
        );
    }

    pub(crate) fn hold_or_apply(&mut self, id: PlayerId, held: Held) {
        if let Err(held) = self.decisions.hold(id, held) {
            self.apply_move(id, held.position, held.rotation, held.on_ground);
        }
    }

    pub(crate) fn tick_plugins(&mut self) {
        self.decisions.expire(self.current_tick());
        self.settle_events();
    }

    pub(crate) fn plugin_disconnected(&mut self, connection: u64) {
        self.plugins.disconnect(connection);
        self.forget_plugin_inventories(connection);
        self.release_departed_plugins();
    }

    pub(crate) fn release_departed_plugins(&mut self) {
        let plugins = &self.plugins;
        self.decisions
            .release(|connection| plugins.is_current(connection));
        self.settle_events();
    }

    fn take_decision(&mut self, connection: u64, event: u64, verdicts: Vec<Verdict>) {
        let (taken, warnings) =
            self.decisions
                .decide(connection, event, verdicts, self.current_tick());
        for warning in warnings {
            self.plugins.send(connection, ToPlugin::Warning(warning));
        }
        match taken {
            Taken::Counted => self.settle_events(),
            Taken::Ignored => {}
            Taken::RolledBack(rollback) => {
                self.undo(rollback.lane, rollback.undo);
                self.notify_undone(rollback.undone, Phase::RolledBack);
                self.notify_undone(rollback.dropped, Phase::Settled);
            }
            Taken::TooLate(what) => {
                if self.plugins.issued(event) {
                    self.plugins.send(
                        connection,
                        ToPlugin::Warning(format!("{what} of event {event} arrived too late")),
                    );
                }
            }
        }
    }

    fn settle_events(&mut self) {
        let tick = self.current_tick();
        loop {
            if let Some(due) = self.decisions.next_due() {
                self.decisions.enter(due.lane);
                self.fire_head(due.lane, Some(due.event), due.then);
                self.decisions.leave();
                continue;
            }
            match self.decisions.next_settled(tick) {
                Some(ready) => self.settle(ready),
                None => break,
            }
        }
    }

    fn settle(&mut self, ready: Ready) {
        let Ready {
            event,
            lane,
            mut payload,
            settlement,
            then,
            held,
            mut record,
        } = ready;
        let (cancelled, patch) = match settlement {
            Settlement::Cancelled(written) => (true, written),
            Settlement::Applied(patch) => (false, patch),
        };
        if let Some(patch) = patch.clone() {
            patch.settle(&mut payload);
        }
        self.notify_plugins(event, &payload, cancelled, Phase::Settled);
        if !cancelled {
            record.applied = Some(payload);
        }
        self.decisions.remember(event, record);
        self.decisions.enter(lane);
        let dropped = self.conclude(then, cancelled, patch, held);
        self.decisions.leave();
        self.notify_undone(dropped, Phase::Settled);
    }

    fn conclude(
        &mut self,
        then: Continuation,
        cancelled: bool,
        patch: Option<EventPatch>,
        held: Vec<Held>,
    ) -> Undone {
        match then {
            Continuation::PlayerMove(step) if cancelled => {
                let dropped = self.decisions.drop_pending(step.player);
                self.send_back(step.player, step.from);
                dropped
            }
            Continuation::PlayerMove(step) => {
                self.finish_move(step, patch_of!(patch, PlayerMove), held)
            }
            Continuation::Notice(_) => Vec::new(),
            Continuation::PreLogin(reply) => {
                self.answer_pre_login(&reply, patch_of!(patch, AsyncPlayerPreLogin));
                Vec::new()
            }
            Continuation::Login(key) => {
                self.finish_login(key, patch_of!(patch, PlayerLogin));
                Vec::new()
            }
            Continuation::Join(id) => {
                self.announce_join(id, patch_of!(patch, PlayerJoin));
                Vec::new()
            }
            Continuation::Leave(step) => {
                self.announce_leave(step, patch_of!(patch, PlayerLeave));
                Vec::new()
            }
            Continuation::Kick(_) if cancelled => Vec::new(),
            Continuation::Kick(step) => {
                self.finish_kick(step, patch_of!(patch, PlayerKick));
                Vec::new()
            }
            Continuation::Settings(step) => {
                if let Some(settings) = step.settings {
                    self.apply_settings(step.player, settings);
                }
                Vec::new()
            }
            Continuation::Chat(_) | Continuation::Command(_) if cancelled => Vec::new(),
            Continuation::Chat(step) => {
                self.finish_chat(step, patch_of!(patch, AsyncPlayerChat));
                Vec::new()
            }
            Continuation::Command(step) => {
                self.finish_command(step, patch_of!(patch, PlayerCommandPreprocess));
                Vec::new()
            }
            Continuation::Teleport(step) => {
                self.finish_teleport(step, cancelled, patch_of!(patch, PlayerTeleport));
                Vec::new()
            }
            Continuation::Velocity(_) if cancelled => Vec::new(),
            Continuation::Velocity(step) => {
                self.finish_velocity(step, patch_of!(patch, PlayerVelocity));
                Vec::new()
            }
            Continuation::Respawn(step) => {
                self.finish_respawn(step, patch_of!(patch, PlayerRespawn));
                Vec::new()
            }
            Continuation::Death(step) => {
                self.finish_death(*step, patch_of!(patch, PlayerDeath));
                Vec::new()
            }
            Continuation::CommandSend(id) => {
                self.finish_command_send(id, patch_of!(patch, PlayerCommandSend));
                Vec::new()
            }
        }
    }

    fn undo(&mut self, lane: Lane, undo: Undo) {
        match (lane, undo) {
            (Lane::Player(player), Undo::SendBack(position)) => self.send_back(player, position),
            (Lane::Login(_), Undo::SendBack(_)) => {}
        }
    }

    fn due(&self, then: Continuation) -> Option<Continuation> {
        Some(then)
    }

    fn due_payload(&self, _then: &Continuation) -> Option<EventPayload> {
        None
    }

    fn finish_move(&mut self, step: MoveStep, patch: PlayerMovePatch, held: Vec<Held>) -> Undone {
        if let Some(to) = patch.to.map(|to| to.position).filter(|to| *to != step.to) {
            let dropped = self.decisions.drop_pending(step.player);
            self.ask_teleport(
                step.player,
                to,
                None,
                PlayerTeleportCause::Plugin,
                Some(step),
            );
            return dropped;
        }
        self.apply_move(step.player, Some(step.to), step.rotation, step.on_ground);
        for packet in held {
            if self
                .players
                .get(&step.player)
                .is_none_or(|p| p.vitals.is_dead())
            {
                break;
            }
            self.apply_move(
                step.player,
                packet.position,
                packet.rotation,
                packet.on_ground,
            );
        }
        Vec::new()
    }

    fn notify_undone(&mut self, undone: Undone, phase: Phase) {
        for (event, payload) in undone {
            self.notify_plugins(event, &payload, true, phase);
        }
    }

    fn send_back(&mut self, id: PlayerId, position: Vec3) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.position != position {
            self.teleport_player(id, position);
            return;
        }
        let teleport = player.next_teleport_id();
        player.link.send(ClientEvent::Teleport {
            id: teleport,
            position,
            rotation: player.rotation,
        });
    }

    fn move_payload(
        &self,
        id: PlayerId,
        from: Vec3,
        to: Vec3,
        rotation: Option<Rotation>,
    ) -> Option<EventPayload> {
        self.snapshot_of(id).map(|mut player| {
            player.position = from;
            let from = Location {
                world: player.world.clone(),
                position: from,
                rotation: player.rotation,
            };
            let to = Location {
                position: to,
                rotation: rotation.unwrap_or(player.rotation),
                ..from.clone()
            };
            EventPayload::PlayerMove(PlayerMove {
                player,
                from,
                to: Some(to),
            })
        })
    }

    fn describe(&self, player: &Player<L>) -> PlayerSnapshot {
        PlayerSnapshot {
            id: player.id,
            name: player.profile.name.clone(),
            entity_id: player.entity_id.0,
            world: self.world.dimension.key.to_string(),
            position: player.position,
            rotation: player.rotation,
            game_mode: player.game_mode,
        }
    }

    fn state_of(&self, id: PlayerId) -> Option<PlayerState> {
        self.players.get(&id).map(|player| {
            let abilities = player.abilities();
            let vitals = &player.vitals;
            PlayerState {
                player: self.describe(player),
                health: vitals.health,
                max_health: crate::health::max_health(player),
                absorption: vitals.absorption,
                food: vitals.food.food,
                saturation: vitals.food.saturation,
                exhaustion: vitals.food.exhaustion,
                level: vitals.experience.level,
                exp: vitals.experience.progress,
                total_exp: vitals.experience.total,
                operator: player.operator,
                allow_flight: abilities.may_fly,
                flying: abilities.flying,
                fly_speed: abilities.fly_speed * SPEED_SCALE,
                walk_speed: abilities.walk_speed * SPEED_SCALE,
                sneaking: player.sneaking,
                sprinting: player.sprinting,
                on_ground: player.on_ground,
                fire_ticks: vitals.fire_ticks,
                dead: vitals.is_dead(),
                effects: player.effects.iter().filter_map(effect_spec).collect(),
                ping: player.latency,
                locale: player.settings.locale.clone(),
                address: player.address,
                hidden: self.visibility.hidden_from(id),
            }
        })
    }

    pub(crate) fn online_snapshots(&self) -> Vec<PlayerSnapshot> {
        let mut online: Vec<PlayerSnapshot> = self
            .players
            .values()
            .map(|player| self.describe(player))
            .collect();
        online.sort_by_key(|snapshot| snapshot.entity_id);
        online
    }
}

fn effect_spec(effect: &EffectInstance) -> Option<EffectSpec> {
    Some(EffectSpec {
        key: effect.effect.name()?.to_owned(),
        duration: effect.duration.wire_ticks(),
        amplifier: i32::from(effect.amplifier),
        ambient: effect.ambient,
        particles: effect.visible,
        icon: effect.show_icon,
    })
}

fn audience(
    listeners: Vec<(u64, HandlerSpec)>,
    payload: &EventPayload,
    cancelled: bool,
    phase: Phase,
) -> (bool, Vec<(u64, Vec<u32>)>) {
    let cancelled = cancelled || payload.cancelled();
    (cancelled, recipients(listeners, cancelled, phase))
}

fn recipients(
    listeners: Vec<(u64, HandlerSpec)>,
    cancelled: bool,
    phase: Phase,
) -> Vec<(u64, Vec<u32>)> {
    let skip_filtered = cancelled && phase == Phase::Settled;
    let mut grouped: Vec<(u64, Vec<u32>)> = Vec::new();
    for (connection, spec) in listeners {
        if skip_filtered && spec.ignore_cancelled {
            continue;
        }
        match grouped.iter_mut().find(|(known, _)| *known == connection) {
            Some((_, handlers)) => handlers.push(spec.id),
            None => grouped.push((connection, vec![spec.id])),
        }
    }
    grouped
}

#[cfg(test)]
mod tests {
    use rspigot_domain::player::GameMode;

    use super::*;
    use crate::plugins::events::{Action, EventResult, Face, PlayerInteract};
    use crate::plugins::{EventKind, Priority};

    fn listener(connection: u64, id: u32, ignore_cancelled: bool) -> (u64, HandlerSpec) {
        (
            connection,
            HandlerSpec {
                id,
                event: EventKind::PlayerMove,
                access: Access::Observe,
                priority: Priority::Normal,
                ignore_cancelled,
            },
        )
    }

    fn mixed() -> Vec<(u64, HandlerSpec)> {
        vec![
            listener(2, 4, false),
            listener(1, 1, true),
            listener(2, 5, true),
            listener(1, 2, false),
        ]
    }

    #[test]
    fn handlers_are_grouped_by_connection_in_the_order_they_were_listed() {
        assert_eq!(
            recipients(mixed(), false, Phase::Settled),
            [(2, vec![4, 5]), (1, vec![1, 2])]
        );
    }

    #[test]
    fn a_cancelled_event_skips_the_handlers_that_ignore_cancelled() {
        assert_eq!(
            recipients(mixed(), true, Phase::Settled),
            [(2, vec![4]), (1, vec![2])]
        );
    }

    #[test]
    fn a_rolled_back_event_still_reaches_the_handlers_that_ignore_cancelled() {
        assert_eq!(
            recipients(mixed(), true, Phase::RolledBack),
            [(2, vec![4, 5]), (1, vec![1, 2])]
        );
    }

    #[test]
    fn a_connection_whose_handlers_are_all_skipped_hears_nothing() {
        let only_filtered = vec![listener(1, 1, true), listener(3, 2, false)];
        assert_eq!(
            recipients(only_filtered, true, Phase::Settled),
            [(3, vec![2])]
        );
        assert!(recipients(Vec::new(), false, Phase::Settled).is_empty());
    }

    fn interaction(result: EventResult) -> EventPayload {
        EventPayload::PlayerInteract(PlayerInteract {
            action: Action::RightClickBlock,
            block: true,
            block_face: Face::Up,
            block_in_hand: false,
            clicked_block: None,
            clicked_position: None,
            hand: None,
            has_item: false,
            item: None,
            material: "minecraft:air".to_owned(),
            player: PlayerSnapshot {
                id: PlayerId::offline("Alice"),
                name: "Alice".to_owned(),
                entity_id: 1,
                world: "minecraft:overworld".to_owned(),
                position: Vec3::new(0.0, 64.0, 0.0),
                rotation: Rotation::default(),
                game_mode: GameMode::Survival,
            },
            use_interacted_block: result,
            use_item_in_hand: EventResult::Default,
        })
    }

    #[test]
    fn a_denied_result_skips_the_handlers_that_ignore_cancelled() {
        assert_eq!(
            audience(
                mixed(),
                &interaction(EventResult::Deny),
                false,
                Phase::Settled
            ),
            (true, vec![(2, vec![4]), (1, vec![2])])
        );
        assert_eq!(
            audience(
                mixed(),
                &interaction(EventResult::Allow),
                false,
                Phase::Settled
            ),
            (false, vec![(2, vec![4, 5]), (1, vec![1, 2])])
        );
    }
}
