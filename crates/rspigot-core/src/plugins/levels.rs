use rspigot_domain::{BlockPos, Rotation, Vec3};

use crate::events::ClientEvent;
use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{
    SpawnChange, TimeSkip, TimeSkipPatch, TimeSkipReason, WorldInit, WorldLoad, WorldSave,
};
use crate::plugins::steps::TimeStep;
use crate::plugins::{Access, EventKind, EventPayload, Location, Phase};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

impl<L: ClientLink> Server<L> {
    pub(crate) fn world_initialised(&mut self) {
        self.fire(
            Lane::World,
            Continuation::Notice(EventKind::WorldInit),
            |server| {
                Some(EventPayload::WorldInit(WorldInit {
                    world: server.world_name(),
                }))
            },
        );
    }

    pub fn world_loaded(&mut self) {
        self.fire(
            Lane::World,
            Continuation::Notice(EventKind::WorldLoad),
            |server| {
                Some(EventPayload::WorldLoad(WorldLoad {
                    world: server.world_name(),
                }))
            },
        );
    }

    pub(crate) fn world_saving(&mut self) {
        self.fire(
            Lane::World,
            Continuation::Notice(EventKind::WorldSave),
            |server| {
                Some(EventPayload::WorldSave(WorldSave {
                    world: server.world_name(),
                }))
            },
        );
    }

    pub(crate) fn world_saving_at_shutdown(&mut self) {
        if !self.plugins.listens(EventKind::WorldSave, Access::Observe) {
            return;
        }
        let payload = EventPayload::WorldSave(WorldSave {
            world: self.world_name(),
        });
        let event = self.next_event_id();
        self.notify_plugins(event, &payload, false, Phase::Settled);
    }

    pub(crate) fn change_day_time(&mut self, time: i64, reason: Option<TimeSkipReason>) {
        let skipped = time.wrapping_sub(self.world.level.day_time);
        self.world.level.day_time = time;
        let update = self.time_event();
        self.push(Audience::All, update);
        let Some(reason) = reason.filter(|_| skipped != 0) else {
            return;
        };
        self.fire(
            Lane::World,
            Continuation::TimeSkip(TimeStep { skipped }),
            move |server| {
                Some(EventPayload::TimeSkip(TimeSkip {
                    skip_amount: skipped,
                    skip_reason: reason,
                    world: server.world_name(),
                }))
            },
        );
    }

    pub(crate) fn finish_time_skip(&mut self, step: TimeStep, patch: TimeSkipPatch) {
        let Some(amount) = patch.skip_amount.filter(|amount| *amount != step.skipped) else {
            return;
        };
        let level = &mut self.world.level;
        level.day_time = level
            .day_time
            .wrapping_add(amount.wrapping_sub(step.skipped));
        let update = self.time_event();
        self.push(Audience::All, update);
    }

    pub(crate) fn change_spawn(&mut self, pos: BlockPos) {
        let previous = self.world.spawn.block();
        if previous == pos {
            return;
        }
        self.world.level.spawn = Some(pos);
        self.world.spawn = pos.center();
        let yaw = self.world.level.spawn_angle;
        self.fire(
            Lane::World,
            Continuation::Notice(EventKind::SpawnChange),
            move |server| {
                let world = server.world_name();
                Some(EventPayload::SpawnChange(SpawnChange {
                    previous_location: Location {
                        world: world.clone(),
                        position: Vec3::new(
                            f64::from(previous.x),
                            f64::from(previous.y),
                            f64::from(previous.z),
                        ),
                        rotation: Rotation::new(yaw, 0.0),
                    },
                    world,
                }))
            },
        );
        self.push(
            Audience::All,
            ClientEvent::SpawnPosition {
                dimension: self.world.dimension.key.clone(),
                pos,
                yaw,
                pitch: 0.0,
            },
        );
    }
}
