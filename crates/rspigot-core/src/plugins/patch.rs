use crate::plugins::resolve::position_of;
use crate::plugins::{EventPatch, Location};

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub handler: u32,
    pub cancel: bool,
    pub patch: Option<EventPatch>,
}

pub(crate) fn location_refusal(location: &Location) -> Option<String> {
    position_of(location.position).err()
}

#[cfg(test)]
mod tests {
    use rspigot_domain::{Rotation, Vec3};

    use super::*;
    use crate::plugins::events::{
        EntityDamageByEntityPatch, EntityDamagePatch, InventoryClickPatch, PlayerMovePatch,
    };
    use crate::plugins::{EventKind, ItemSpec};

    fn at(x: f64) -> Location {
        Location {
            world: "minecraft:overworld".to_owned(),
            position: Vec3::new(x, 64.0, 0.0),
            rotation: Rotation::default(),
        }
    }

    fn moving(to: Option<Location>) -> EventPatch {
        EventPatch::PlayerMove(PlayerMovePatch { to })
    }

    #[test]
    fn a_later_patch_wins_each_field_it_sets_and_keeps_the_others() {
        assert_eq!(
            moving(Some(at(1.0))).then(moving(Some(at(9.0)))),
            moving(Some(at(9.0)))
        );
        assert_eq!(
            moving(Some(at(1.0))).then(moving(None)),
            moving(Some(at(1.0)))
        );
        assert_eq!(
            moving(None).then(moving(Some(at(9.0)))),
            moving(Some(at(9.0)))
        );
    }

    #[test]
    fn a_destination_outside_the_world_is_refused() {
        assert_eq!(moving(Some(at(1.0))).refusal(), None);
        assert_eq!(moving(None).refusal(), None);
        for nowhere in [
            Vec3::new(f64::NAN, 64.0, 0.0),
            Vec3::new(0.0, f64::INFINITY, 0.0),
            Vec3::new(0.0, 64.0, 4.0e7),
        ] {
            let mut location = at(0.0);
            location.position = nowhere;
            assert!(moving(Some(location)).refusal().is_some());
        }
    }

    #[test]
    fn a_patch_of_a_parent_event_lifts_into_the_fired_event() {
        let parent = EventPatch::EntityDamage(EntityDamagePatch { damage: Some(2.5) });
        assert_eq!(
            parent.clone().lift(EventKind::EntityDamageByEntity),
            Some(EventPatch::EntityDamageByEntity(
                EntityDamageByEntityPatch { damage: Some(2.5) }
            ))
        );
        assert_eq!(
            parent.clone().lift(EventKind::EntityDamage),
            Some(parent.clone())
        );
        assert_eq!(parent.lift(EventKind::PlayerMove), None);
    }

    #[test]
    fn only_a_direct_write_survives_a_cancel() {
        let stone = ItemSpec::of("minecraft:stone", 1);
        let both = EventPatch::InventoryClick(InventoryClickPatch {
            current_item: Some(None),
            cursor: Some(Some(stone.clone())),
            result: Some(crate::plugins::events::EventResult::Allow),
        });
        assert_eq!(
            both.when_cancelled(),
            Some(EventPatch::InventoryClick(InventoryClickPatch {
                current_item: Some(None),
                cursor: Some(Some(stone)),
                result: None,
            }))
        );
        assert_eq!(moving(Some(at(1.0))).when_cancelled(), None);
        let result_only = EventPatch::InventoryClick(InventoryClickPatch {
            result: Some(crate::plugins::events::EventResult::Deny),
            ..InventoryClickPatch::default()
        });
        assert_eq!(result_only.when_cancelled(), None);
    }
}
