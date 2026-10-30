use rspigot_entities::EntityKind;
use rspigot_items::ItemKind;

use crate::brain::Brain;
use crate::goal::GoalSelector;
use crate::goals::{
    FloatGoal, LookAtPlayerGoal, MobGoal, NearestAttackableTargetGoal, RandomLookAroundGoal,
    RandomStrollGoal, TargetFilter, TemptGoal, TemptItems,
};
use crate::world::TargetKind;

pub const ZOMBIE_AGGRESSIVE_TOWARDS_VILLAGER: bool = true;

struct Goals<'a> {
    goals: &'a mut GoalSelector<MobGoal>,
    targets: &'a mut GoalSelector<MobGoal>,
    brain: &'a mut Brain,
}

impl Goals<'_> {
    fn float(&mut self, priority: i32) {
        self.brain.profile.can_float = true;
        self.goals.add_goal(priority, MobGoal::Float(FloatGoal));
    }

    fn tempt(&mut self, priority: i32, speed: f64, items: TemptItems) {
        self.goals
            .add_goal(priority, MobGoal::Tempt(TemptGoal::new(speed, items)));
    }

    fn stroll(&mut self, priority: i32, speed: f64) {
        self.goals.add_goal(
            priority,
            MobGoal::Stroll(RandomStrollGoal::water_avoiding(speed)),
        );
    }

    fn look(&mut self, priority: i32, distance: f32) {
        self.goals.add_goal(
            priority,
            MobGoal::LookAtPlayer(LookAtPlayerGoal::new(distance)),
        );
    }

    fn look_around(&mut self, priority: i32) {
        self.goals.add_goal(
            priority,
            MobGoal::RandomLookAround(RandomLookAroundGoal::default()),
        );
    }

    fn target(&mut self, priority: i32, goal: NearestAttackableTargetGoal) {
        self.targets
            .add_goal(priority, MobGoal::NearestAttackableTarget(goal));
    }
}

pub fn register_goals(
    kind: EntityKind,
    goals: &mut GoalSelector<MobGoal>,
    targets: &mut GoalSelector<MobGoal>,
    brain: &mut Brain,
) {
    let mut add = Goals {
        goals,
        targets,
        brain,
    };
    match kind {
        EntityKind::Cow => {
            add.float(0);
            add.tempt(3, 1.25, TemptItems::Tag("minecraft:cow_food"));
            add.stroll(5, 1.0);
            add.look(6, 6.0);
            add.look_around(7);
        }
        EntityKind::Pig => {
            add.float(0);
            add.tempt(4, 1.2, TemptItems::Item(ItemKind::CarrotOnAStick));
            add.tempt(4, 1.2, TemptItems::Tag("minecraft:pig_food"));
            add.stroll(6, 1.0);
            add.look(7, 6.0);
            add.look_around(8);
        }
        EntityKind::Sheep => {
            add.float(0);
            add.tempt(3, 1.1, TemptItems::Tag("minecraft:sheep_food"));
            add.stroll(6, 1.0);
            add.look(7, 6.0);
            add.look_around(8);
        }
        EntityKind::Chicken => {
            add.float(0);
            add.tempt(3, 1.0, TemptItems::Tag("minecraft:chicken_food"));
            add.stroll(5, 1.0);
            add.look(6, 6.0);
            add.look_around(7);
        }
        EntityKind::Zombie => {
            add.look(8, 8.0);
            add.look_around(8);
            add.stroll(7, 1.0);
            add.target(
                2,
                NearestAttackableTargetGoal::new(TargetKind::Player, true),
            );
            if ZOMBIE_AGGRESSIVE_TOWARDS_VILLAGER {
                add.target(
                    3,
                    NearestAttackableTargetGoal::new(TargetKind::AbstractVillager, false),
                );
            }
            add.target(
                3,
                NearestAttackableTargetGoal::new(TargetKind::IronGolem, true),
            );
            add.target(
                5,
                NearestAttackableTargetGoal::with(
                    TargetKind::Turtle,
                    10,
                    true,
                    TargetFilter::BabyOnLand,
                ),
            );
        }
        EntityKind::Skeleton => {
            add.stroll(5, 1.0);
            add.look(6, 8.0);
            add.look_around(6);
            add.target(
                2,
                NearestAttackableTargetGoal::new(TargetKind::Player, true),
            );
            add.target(
                3,
                NearestAttackableTargetGoal::new(TargetKind::IronGolem, true),
            );
            add.target(
                3,
                NearestAttackableTargetGoal::with(
                    TargetKind::Turtle,
                    10,
                    true,
                    TargetFilter::BabyOnLand,
                ),
            );
        }
        EntityKind::Creeper => {
            add.float(1);
            add.stroll(5, 0.8);
            add.look(6, 8.0);
            add.look_around(6);
            add.target(
                1,
                NearestAttackableTargetGoal::new(TargetKind::Player, true),
            );
        }
        EntityKind::Spider => {
            add.brain.navigation.set_climber(true);
            add.float(1);
            add.stroll(5, 0.8);
            add.look(6, 8.0);
            add.look_around(6);
            add.target(2, NearestAttackableTargetGoal::spider(TargetKind::Player));
            add.target(
                3,
                NearestAttackableTargetGoal::spider(TargetKind::IronGolem),
            );
        }
        _ => {}
    }
}
