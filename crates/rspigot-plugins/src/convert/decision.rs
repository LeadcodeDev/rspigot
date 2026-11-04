use rspigot_core::plugins::{FromPlugin, Verdict};
use rspigot_plugin_protocol::v1;

use super::Malformed;
use super::events::change;

pub(super) fn decision(decision: v1::Decision) -> Result<FromPlugin, Malformed> {
    Ok(FromPlugin::Decision {
        event: decision.event,
        verdicts: decision
            .verdicts
            .into_iter()
            .map(verdict)
            .collect::<Result<_, _>>()?,
    })
}

fn verdict(verdict: v1::Verdict) -> Result<Verdict, Malformed> {
    Ok(Verdict {
        handler: verdict.handler,
        cancel: verdict.cancel,
        patch: verdict
            .patch
            .and_then(|patch| patch.change)
            .map(change)
            .transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use rspigot_core::plugins::events::{EntityTargetPatch, PlayerMovePatch};
    use rspigot_core::plugins::{EntityView, EventPatch, Location};
    use rspigot_domain::{EntityId, Rotation, Vec3};
    use rspigot_plugin_protocol::v1::{from_plugin, patch};
    use uuid::Uuid;

    use super::*;
    use crate::convert;

    fn moving(to: Option<v1::Location>) -> Option<v1::Patch> {
        Some(v1::Patch {
            change: Some(patch::Change::PlayerMove(v1::PlayerMovePatch { to })),
        })
    }

    fn targeting(target: Option<v1::NullableEntitySnapshot>) -> Option<v1::Patch> {
        Some(v1::Patch {
            change: Some(patch::Change::EntityTarget(v1::EntityTargetPatch {
                target,
            })),
        })
    }

    fn decide(verdicts: Vec<v1::Verdict>) -> Result<FromPlugin, Malformed> {
        convert::from_plugin(v1::FromPlugin {
            body: Some(from_plugin::Body::Decision(v1::Decision {
                event: 7,
                verdicts,
            })),
        })
    }

    fn zombie() -> v1::EntitySnapshot {
        v1::EntitySnapshot {
            entity_id: 4,
            uuid: "00000000-0000-0000-0000-000000000004".to_owned(),
            kind: "minecraft:zombie".to_owned(),
            position: Some(v1::Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            }),
        }
    }

    #[test]
    fn a_decision_converts_every_verdict_with_its_patch() {
        let verdicts = vec![
            v1::Verdict {
                handler: 2,
                cancel: false,
                patch: moving(Some(v1::Location {
                    world: "minecraft:overworld".to_owned(),
                    position: Some(v1::Vec3 {
                        x: 4.5,
                        y: 70.0,
                        z: -1.5,
                    }),
                    rotation: Some(v1::Rotation {
                        yaw: 0.0,
                        pitch: 0.0,
                    }),
                })),
            },
            v1::Verdict {
                handler: 4,
                cancel: true,
                patch: None,
            },
            v1::Verdict {
                handler: 5,
                cancel: false,
                patch: moving(None),
            },
            v1::Verdict {
                handler: 6,
                cancel: false,
                patch: Some(v1::Patch { change: None }),
            },
        ];
        assert_eq!(
            decide(verdicts).unwrap(),
            FromPlugin::Decision {
                event: 7,
                verdicts: vec![
                    Verdict {
                        handler: 2,
                        cancel: false,
                        patch: Some(EventPatch::PlayerMove(PlayerMovePatch {
                            to: Some(Location {
                                world: "minecraft:overworld".to_owned(),
                                position: Vec3::new(4.5, 70.0, -1.5),
                                rotation: Rotation::default(),
                            }),
                        })),
                    },
                    Verdict {
                        handler: 4,
                        cancel: true,
                        patch: None,
                    },
                    Verdict {
                        handler: 5,
                        cancel: false,
                        patch: Some(EventPatch::PlayerMove(PlayerMovePatch { to: None })),
                    },
                    Verdict {
                        handler: 6,
                        cancel: false,
                        patch: None,
                    },
                ],
            }
        );
    }

    #[test]
    fn a_nullable_patch_tells_a_cleared_value_from_an_untouched_one() {
        let target = |patch| {
            let Ok(FromPlugin::Decision { verdicts, .. }) = decide(vec![v1::Verdict {
                handler: 1,
                cancel: false,
                patch,
            }]) else {
                panic!("expected a decision");
            };
            verdicts[0].patch.clone()
        };
        assert_eq!(
            target(targeting(None)),
            Some(EventPatch::EntityTarget(EntityTargetPatch { target: None }))
        );
        assert_eq!(
            target(targeting(Some(v1::NullableEntitySnapshot { value: None }))),
            Some(EventPatch::EntityTarget(EntityTargetPatch {
                target: Some(None)
            }))
        );
        assert_eq!(
            target(targeting(Some(v1::NullableEntitySnapshot {
                value: Some(zombie())
            }))),
            Some(EventPatch::EntityTarget(EntityTargetPatch {
                target: Some(Some(EntityView {
                    id: EntityId(4),
                    uuid: Uuid::from_u128(4),
                    kind: "minecraft:zombie".to_owned(),
                    position: Vec3::new(0.0, 64.0, 0.0),
                }))
            }))
        );
    }

    #[test]
    fn a_malformed_patch_drops_the_whole_decision() {
        let mut stranger = zombie();
        stranger.uuid = "nobody".to_owned();
        let refused = decide(vec![v1::Verdict {
            handler: 1,
            cancel: true,
            patch: targeting(Some(v1::NullableEntitySnapshot {
                value: Some(stranger),
            })),
        }]);
        assert!(matches!(refused, Err(Malformed::Entity(uuid)) if uuid == "nobody"));
    }
}
