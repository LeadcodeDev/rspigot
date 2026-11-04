use std::fmt::Display;

use rspigot_domain::effect::EffectInstance;
use rspigot_domain::player::PlayerId;
use rspigot_domain::{EntityId, Rotation, Vec3};

use crate::commands::CommandContext;
use crate::events::{ClientEvent, Hand};
use crate::player::Player;
use crate::plugins::decisions::{Held, PendingMove, SettledMove, Undone, Verdict};
use crate::plugins::player::SPEED_SCALE;
use crate::plugins::{
    Access, Answer, EffectSpec, EventKind, EventPayload, FromPlugin, HandlerSpec, Order, Phase,
    PlayerSnapshot, PlayerState, Question, ToPlugin,
};
use crate::ports::ClientLink;
use crate::server::Server;

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
        for (connection, handlers) in recipients(listeners, cancelled, phase) {
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
        if let Some((connection, handlers)) =
            recipients(owned, false, Phase::Settled).into_iter().next()
        {
            let id = self.next_event_id();
            self.plugins.send(
                connection,
                ToPlugin::Event {
                    id,
                    handlers,
                    phase: Phase::Settled,
                    cancelled: false,
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
            FromPlugin::Decision { event, cancel } => self.take_decision(connection, event, cancel),
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
        if !self.plugins.listens(EventKind::PlayerMove, Access::Decide) {
            let payload = if self.plugins.listens(EventKind::PlayerMove, Access::Observe) {
                self.move_payload(id, from, to)
            } else {
                None
            };
            self.apply_move(id, Some(to), rotation, on_ground);
            if let Some(payload) = payload {
                let event = self.next_event_id();
                self.notify_plugins(event, &payload, false, Phase::Settled);
            }
            return;
        }
        let Some(payload) = self.move_payload(id, from, to) else {
            return;
        };
        let event = self.next_event_id();
        let deciders =
            self.plugins
                .listeners(EventKind::PlayerMove, Access::Decide, &self.config.plugins);
        let asked = recipients(deciders, false, Phase::Decide);
        let waiting = asked.iter().map(|(connection, _)| *connection).collect();
        for (connection, handlers) in asked {
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
            PendingMove {
                player: id,
                from,
                to,
                rotation,
                on_ground,
                payload,
                waiting,
                cancelled: false,
                deadline,
                until,
                held: Vec::new(),
            },
        );
    }

    pub(crate) fn hold_or_apply(&mut self, id: PlayerId, held: Held) {
        if let Err(held) = self.decisions.hold(id, held) {
            self.apply_move(id, held.position, held.rotation, held.on_ground);
        }
    }

    pub(crate) fn tick_plugins(&mut self) {
        self.decisions.expire(self.current_tick());
        self.settle_moves();
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
        self.settle_moves();
    }

    fn take_decision(&mut self, connection: u64, event: u64, cancel: bool) {
        match self
            .decisions
            .decide(connection, event, cancel, self.current_tick())
        {
            Verdict::Counted => self.settle_moves(),
            Verdict::Ignored => {}
            Verdict::RolledBack(rollback) => {
                self.send_back(rollback.player, rollback.from);
                self.notify_undone(rollback.undone, Phase::RolledBack);
                self.notify_undone(rollback.dropped, Phase::Settled);
            }
            Verdict::TooLate => {
                if self.plugins.issued(event) {
                    self.plugins.send(
                        connection,
                        ToPlugin::Warning(format!("cancel of event {event} arrived too late")),
                    );
                }
            }
        }
    }

    fn settle_moves(&mut self) {
        let tick = self.current_tick();
        while let Some((event, settled)) = self.decisions.next_settled(tick) {
            let PendingMove {
                player,
                from,
                to,
                rotation,
                on_ground,
                payload,
                waiting,
                cancelled,
                until,
                held,
                ..
            } = settled;
            let mut remembered = SettledMove {
                player,
                from,
                silent: waiting,
                until,
                applied: None,
            };
            if cancelled {
                let dropped = self.decisions.drop_pending(player);
                self.decisions.remember(event, remembered);
                self.send_back(player, from);
                self.notify_plugins(event, &payload, true, Phase::Settled);
                self.notify_undone(dropped, Phase::Settled);
                continue;
            }
            self.notify_plugins(event, &payload, false, Phase::Settled);
            remembered.applied = Some(payload);
            self.decisions.remember(event, remembered);
            self.apply_move(player, Some(to), rotation, on_ground);
            for step in held {
                if self.players.get(&player).is_none_or(|p| p.vitals.is_dead()) {
                    break;
                }
                self.apply_move(player, step.position, step.rotation, step.on_ground);
            }
        }
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

    fn move_payload(&self, id: PlayerId, from: Vec3, to: Vec3) -> Option<EventPayload> {
        self.snapshot_of(id).map(|mut player| {
            player.position = from;
            EventPayload::PlayerMove { player, from, to }
        })
    }

    pub(crate) fn tell_plugins_of_join(&mut self, id: PlayerId) {
        self.settle_for_plugins(id, |player| EventPayload::PlayerJoin { player });
    }

    pub(crate) fn tell_plugins_of_interact(
        &mut self,
        id: PlayerId,
        target: EntityId,
        hand: Hand,
        sneaking: bool,
    ) {
        if !self
            .plugins
            .listens(EventKind::EntityInteract, Access::Observe)
        {
            return;
        }
        let Some(entity) = self.entity_view(target) else {
            return;
        };
        self.settle_for_plugins(id, |player| EventPayload::EntityInteract {
            player,
            entity,
            hand,
            sneaking,
        });
    }

    pub(crate) fn tell_plugins_of_leave(&mut self, id: PlayerId) {
        self.settle_for_plugins(id, |player| EventPayload::PlayerLeave { player });
    }

    fn settle_for_plugins(
        &mut self,
        id: PlayerId,
        payload: impl FnOnce(PlayerSnapshot) -> EventPayload,
    ) {
        if let Some(player) = self.snapshot_of(id) {
            let event = self.next_event_id();
            self.notify_plugins(event, &payload(player), false, Phase::Settled);
        }
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

    fn online_snapshots(&self) -> Vec<PlayerSnapshot> {
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
    use super::*;
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
}
