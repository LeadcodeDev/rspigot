use rspigot_core::plugins::{EventPatch, FromPlugin, PlayerMovePatch, Verdict};
use rspigot_plugin_protocol::v1::{self, patch};

use super::shared::vec3;

pub(super) fn decision(decision: v1::Decision) -> FromPlugin {
    FromPlugin::Decision {
        event: decision.event,
        verdicts: decision.verdicts.into_iter().map(verdict).collect(),
    }
}

fn verdict(verdict: v1::Verdict) -> Verdict {
    Verdict {
        handler: verdict.handler,
        cancel: verdict.cancel,
        patch: verdict.patch.and_then(|patch| patch.change).map(change),
    }
}

fn change(change: patch::Change) -> EventPatch {
    match change {
        patch::Change::PlayerMove(moved) => EventPatch::PlayerMove(PlayerMovePatch {
            to: moved.to.map(vec3),
        }),
    }
}

#[cfg(test)]
mod tests {
    use rspigot_domain::Vec3;
    use rspigot_plugin_protocol::v1::from_plugin;

    use super::*;
    use crate::convert;

    fn moving(to: Option<v1::Vec3>) -> Option<v1::Patch> {
        Some(v1::Patch {
            change: Some(patch::Change::PlayerMove(v1::PlayerMovePatch { to })),
        })
    }

    #[test]
    fn a_decision_converts_every_verdict_with_its_patch() {
        let decision = v1::FromPlugin {
            body: Some(from_plugin::Body::Decision(v1::Decision {
                event: 7,
                verdicts: vec![
                    v1::Verdict {
                        handler: 2,
                        cancel: false,
                        patch: moving(Some(v1::Vec3 {
                            x: 4.5,
                            y: 70.0,
                            z: -1.5,
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
                ],
            })),
        };
        assert_eq!(
            convert::from_plugin(decision).unwrap(),
            FromPlugin::Decision {
                event: 7,
                verdicts: vec![
                    Verdict {
                        handler: 2,
                        cancel: false,
                        patch: Some(EventPatch::PlayerMove(PlayerMovePatch {
                            to: Some(Vec3::new(4.5, 70.0, -1.5)),
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
}
