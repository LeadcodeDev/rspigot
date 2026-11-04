use rspigot_domain::Vec3;

use crate::plugins::resolve::position_of;
use crate::plugins::{EntityView, EventPatch, Location};

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub handler: u32,
    pub cancel: bool,
    pub patch: Option<EventPatch>,
}

pub(crate) fn location_refusal(location: &Location) -> Option<String> {
    let rotation = location.rotation;
    position_of(location.position).err().or_else(|| {
        (!(rotation.yaw.is_finite() && rotation.pitch.is_finite()))
            .then(|| format!("rotation {} {} is not finite", rotation.yaw, rotation.pitch))
    })
}

pub(crate) fn vector_refusal(name: &str, vector: Vec3) -> Option<String> {
    (!vector.is_finite()).then(|| {
        format!(
            "{name} {} {} {} is not finite",
            vector.x, vector.y, vector.z
        )
    })
}

pub(crate) fn merge_indexed<V>(
    mut earlier: Vec<(EntityView, V)>,
    later: Vec<(EntityView, V)>,
) -> Vec<(EntityView, V)> {
    for (entity, value) in later {
        match earlier.iter_mut().find(|(known, _)| known.id == entity.id) {
            Some(slot) => *slot = (entity, value),
            None => earlier.push((entity, value)),
        }
    }
    earlier
}

pub(crate) fn number_refusal(name: &str, value: f64) -> Option<String> {
    (!value.is_finite()).then(|| format!("{name} {value} is not finite"))
}

#[cfg(test)]
mod tests {
    use rspigot_domain::{EntityId, Rotation};
    use uuid::Uuid;

    use super::*;
    use crate::plugins::events::{
        EntityDamageByEntityPatch, EntityDamagePatch, EntityExhaustionPatch, EntityTeleportPatch,
        EventResult, InventoryClickPatch, InventoryDragPatch, PlayerMovePatch, PlayerVelocityPatch,
        PotionSplashPatch,
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
    fn every_patched_floating_value_must_be_finite() {
        let damage = |damage| {
            EventPatch::EntityDamage(EntityDamagePatch {
                damage: Some(damage),
            })
        };
        assert_eq!(damage(2.5).refusal(), None);
        assert_eq!(
            damage(f64::NAN).refusal(),
            Some("damage NaN is not finite".to_owned())
        );
        let tired = |exhaustion| {
            EventPatch::EntityExhaustion(EntityExhaustionPatch {
                exhaustion: Some(exhaustion),
            })
        };
        assert_eq!(tired(0.5).refusal(), None);
        assert_eq!(
            tired(f32::INFINITY).refusal(),
            Some("exhaustion inf is not finite".to_owned())
        );
        let pushed = |velocity| {
            EventPatch::PlayerVelocity(PlayerVelocityPatch {
                velocity: Some(velocity),
            })
        };
        assert_eq!(pushed(Vec3::new(0.0, 1.0, 0.0)).refusal(), None);
        assert_eq!(
            pushed(Vec3::new(0.0, f64::NAN, 0.0)).refusal(),
            Some("velocity 0 NaN 0 is not finite".to_owned())
        );
        let mut turned = at(1.0);
        turned.rotation = Rotation::new(f32::NAN, 0.0);
        let teleported = |to| EventPatch::EntityTeleport(EntityTeleportPatch { to: Some(to) });
        assert_eq!(teleported(None).refusal(), None);
        assert_eq!(teleported(Some(at(1.0))).refusal(), None);
        assert_eq!(
            teleported(Some(turned)).refusal(),
            Some("rotation NaN 0 is not finite".to_owned())
        );
    }

    fn splashed(id: i32) -> EntityView {
        EntityView {
            id: EntityId(id),
            uuid: Uuid::from_u128(u128::try_from(id).unwrap_or_default()),
            kind: "minecraft:zombie".to_owned(),
            position: Vec3::new(0.0, 64.0, 0.0),
        }
    }

    fn intensities(values: &[(i32, f64)]) -> EventPatch {
        EventPatch::PotionSplash(PotionSplashPatch {
            intensities: values
                .iter()
                .map(|&(id, value)| (splashed(id), value))
                .collect(),
        })
    }

    #[test]
    fn an_indexed_patch_keeps_the_last_value_written_for_each_entity() {
        assert_eq!(
            intensities(&[(1, 0.5), (2, 0.2)]).then(intensities(&[(1, 0.9), (3, 1.0)])),
            intensities(&[(1, 0.9), (2, 0.2), (3, 1.0)])
        );
        assert_eq!(intensities(&[(1, 0.5)]).refusal(), None);
        assert_eq!(
            intensities(&[(1, 0.5), (2, f64::NAN)]).refusal(),
            Some("intensities NaN is not finite".to_owned())
        );
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
    fn only_direct_writes_and_results_survive_a_cancel() {
        let stone = ItemSpec::of("minecraft:stone", 1);
        let both = EventPatch::InventoryClick(InventoryClickPatch {
            current_item: Some(None),
            cursor: Some(Some(stone.clone())),
            result: Some(EventResult::Allow),
        });
        assert_eq!(both.clone().when_cancelled(), Some(both));
        assert_eq!(moving(Some(at(1.0))).when_cancelled(), None);
        let dragged = EventPatch::InventoryDrag(InventoryDragPatch {
            cursor: Some(Some(stone)),
            result: Some(EventResult::Deny),
        });
        assert_eq!(
            dragged.when_cancelled(),
            Some(EventPatch::InventoryDrag(InventoryDragPatch {
                cursor: None,
                result: Some(EventResult::Deny),
            }))
        );
        let cursor_only = EventPatch::InventoryDrag(InventoryDragPatch {
            cursor: Some(None),
            ..InventoryDragPatch::default()
        });
        assert_eq!(cursor_only.when_cancelled(), None);
    }
}
