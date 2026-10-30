use rspigot_domain::entity::EntityBase;
use rspigot_domain::entity::living::LivingState;
use rspigot_domain::{BlockPos, EntityId, Vec3};
use rspigot_entities::EntityKind;

use crate::control::{BodyRotationControl, JumpControl, LookControl, MoveControl};
use crate::goal::{Flag, GoalSelector};
use crate::goals::MobGoal;
use crate::random::MobRandom;
use crate::registry;
use crate::sensing::Sensing;
use crate::targeting;
use crate::world::{AiWorld, Living, MobView};
use crate::{BlockView, MobProfile, MobState, MobTick, Navigation, PathType, combat_fall_distance};

pub const UPDATE_GOAL_SELECTOR_EVERY_N_TICKS: i32 = 2;
pub const CONTROL_FLAG_INTERVAL: i32 = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    pub random: MobRandom,
    pub look: LookControl,
    pub movement: MoveControl,
    pub jump: JumpControl,
    pub body: BodyRotationControl,
    pub navigation: Navigation,
    pub sensing: Sensing,
    pub target: Option<EntityId>,
    pub profile: MobProfile,
    follow_range: f64,
}

impl Brain {
    pub fn new(kind: EntityKind, seed: i64) -> Self {
        let dimensions = kind.dimensions();
        let follow_range = kind
            .default_attribute("minecraft:follow_range")
            .unwrap_or(0.0);
        let mut profile = MobProfile::new(dimensions.width, dimensions.height);
        if kind.extends("Animal") {
            profile.set_malus(PathType::FireInNeighbor, 16.0);
            profile.set_malus(PathType::Fire, -1.0);
        }
        if kind == EntityKind::Chicken {
            profile.set_malus(PathType::Water, 0.0);
        }
        Self {
            random: MobRandom::new(seed),
            look: LookControl::default(),
            movement: MoveControl::default(),
            jump: JumpControl::default(),
            body: BodyRotationControl::default(),
            navigation: Navigation::with_follow_range(follow_range),
            sensing: Sensing::default(),
            target: None,
            profile,
            follow_range,
        }
    }
}

pub struct AiContext<'a, W> {
    pub mob: &'a MobView,
    pub world: &'a W,
    pub brain: &'a mut Brain,
}

pub fn mob_tick(mob: &MobView, game_time: i64) -> MobTick {
    MobTick {
        state: MobState {
            position: mob.position,
            on_ground: mob.on_ground,
            in_floatable_fluid: mob.fluid.in_water,
        },
        in_liquid: mob.in_liquid(),
        passenger: false,
        speed: mob.speed,
        game_time,
    }
}

impl<W: AiWorld> AiContext<'_, W> {
    pub fn tick_state(&self) -> MobTick {
        mob_tick(self.mob, self.world.game_time())
    }

    pub fn target(&self) -> Option<Living> {
        let id = self.brain.target?;
        let living = self.world.living(id)?;
        targeting::valid_target(self.world, living)
    }

    pub fn set_target(&mut self, target: Option<Living>) {
        self.brain.target = target
            .and_then(|living| targeting::valid_target(self.world, living))
            .map(|living| living.id);
    }

    fn path_profile(&mut self) -> MobProfile {
        let mob = self.mob;
        let mut profile = self.brain.profile.clone();
        profile.step_height = mob.step_height;
        profile.follow_range = mob.follow_range as f32;
        profile.max_fall_distance = if self.target().is_none() {
            MobProfile::DEFAULT_MAX_FALL_DISTANCE
        } else if mob.kind == EntityKind::Creeper {
            (mob.health - 1.0 + 3.0).floor() as i32
        } else {
            combat_fall_distance(mob.health, mob.max_health, self.world.difficulty())
        };
        profile
    }

    pub fn move_to_pos(&mut self, position: Vec3, speed_modifier: f64) -> bool {
        let profile = self.path_profile();
        let tick = self.tick_state();
        let follow = self.mob.follow_range;
        self.brain.navigation.move_to_pos(
            self.world,
            &profile,
            &tick,
            position,
            speed_modifier,
            follow,
        )
    }

    pub fn move_to_entity(&mut self, target: &Living, speed_modifier: f64) -> bool {
        let profile = self.path_profile();
        let tick = self.tick_state();
        let follow = self.mob.follow_range;
        self.brain.navigation.move_to_entity(
            self.world,
            &profile,
            &tick,
            target.position,
            speed_modifier,
            follow,
        )
    }

    pub fn stop_navigation(&mut self) {
        self.brain.navigation.stop();
    }

    fn recompute_path(&mut self) {
        let profile = self.path_profile();
        let tick = self.tick_state();
        let follow = self.mob.follow_range;
        self.brain
            .navigation
            .recompute_path(self.world, &profile, &tick, follow);
    }

    fn tick_navigation(&mut self) {
        let profile = self.path_profile();
        let tick = self.tick_state();
        let follow = self.mob.follow_range;
        let navigation = &self.brain.navigation;
        let climbing = navigation.is_climber() && navigation.is_done();
        if navigation.has_delayed_recomputation() && !climbing {
            self.brain
                .navigation
                .recompute_path(self.world, &profile, &tick, follow);
        }
        if let Some(wanted) = self.brain.navigation.tick(self.world, &profile, &tick) {
            self.brain
                .movement
                .set_wanted_position(wanted.position, wanted.speed_modifier);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobAi {
    goals: GoalSelector<MobGoal>,
    targets: GoalSelector<MobGoal>,
    brain: Brain,
}

impl MobAi {
    pub fn new(kind: EntityKind, seed: i64) -> Self {
        let mut ai = Self {
            goals: GoalSelector::new(),
            targets: GoalSelector::new(),
            brain: Brain::new(kind, seed),
        };
        registry::register_goals(kind, &mut ai.goals, &mut ai.targets, &mut ai.brain);
        ai
    }

    pub fn goals(&self) -> &GoalSelector<MobGoal> {
        &self.goals
    }

    pub fn targets(&self) -> &GoalSelector<MobGoal> {
        &self.targets
    }

    pub fn brain(&self) -> &Brain {
        &self.brain
    }

    pub fn brain_mut(&mut self) -> &mut Brain {
        &mut self.brain
    }

    pub fn target(&self) -> Option<EntityId> {
        self.brain.target
    }

    pub fn can_float(&self) -> bool {
        self.brain.profile.can_float
    }

    pub fn think<W: AiWorld>(&mut self, mob: &MobView, world: &W) {
        self.brain.sensing.tick();
        if self.brain.follow_range != mob.follow_range {
            self.brain.follow_range = mob.follow_range;
            self.brain
                .navigation
                .update_max_visited_nodes(mob.follow_range);
        }
        let mut context = AiContext {
            mob,
            world,
            brain: &mut self.brain,
        };
        let phase = mob.tick_count.wrapping_add(mob.id.0);
        if phase % UPDATE_GOAL_SELECTOR_EVERY_N_TICKS != 0 && mob.tick_count > 1 {
            self.targets.tick_running_goals(&mut context, false);
            self.goals.tick_running_goals(&mut context, false);
        } else {
            self.targets.tick(&mut context);
            self.goals.tick(&mut context);
        }
        context.tick_navigation();
    }

    pub fn act(&mut self, base: &mut EntityBase, living: &mut LivingState, view: &impl BlockView) {
        let brain = &mut self.brain;
        let profile = &brain.profile;
        let state = MobState {
            position: base.position,
            on_ground: base.on_ground,
            in_floatable_fluid: base.fluid.in_water,
        };
        brain
            .movement
            .tick(base, living, view, &mut brain.jump, |pos| {
                crate::walk::mob_path_type(view, profile, state, pos) == PathType::Walkable
            });
        let navigating = brain.navigation.is_in_progress();
        brain.look.tick(base, living, navigating);
        brain.jump.tick(living);
    }

    pub fn block_changed<W: AiWorld>(&mut self, pos: BlockPos, mob: &MobView, world: &W) {
        if !self
            .brain
            .navigation
            .should_recompute_path(pos, mob.position)
        {
            return;
        }
        let mut context = AiContext {
            mob,
            world,
            brain: &mut self.brain,
        };
        context.recompute_path();
    }

    pub fn turn_body(&mut self, base: &mut EntityBase, living: &mut LivingState) {
        self.brain.body.client_tick(base, living);
    }

    pub fn update_control_flags(&mut self) {
        self.goals.set_control_flag(Flag::Move, true);
        self.goals.set_control_flag(Flag::Jump, true);
        self.goals.set_control_flag(Flag::Look, true);
    }
}
