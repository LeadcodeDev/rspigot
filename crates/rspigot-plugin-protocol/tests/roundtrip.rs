use prost::Message;
use rspigot_plugin_protocol::PROTOCOL_VERSION;
use rspigot_plugin_protocol::v1::{
    Access, Decision, Event, EventKind, FromPlugin, GameMode, Handler, Hello, Patch, Phase,
    PlayerMove, PlayerMovePatch, PlayerSnapshot, Priority, Rotation, ToPlugin, Vec3, Verdict,
    event, from_plugin, patch, to_plugin,
};

fn steve() -> PlayerSnapshot {
    PlayerSnapshot {
        uuid: "00000000-0000-0000-0000-000000000001".to_owned(),
        name: "Steve".to_owned(),
        entity_id: 12,
        world: "world".to_owned(),
        position: Some(Vec3 {
            x: 1.0,
            y: 64.0,
            z: -3.5,
        }),
        game_mode: GameMode::Survival as i32,
        rotation: Some(Rotation {
            yaw: 90.0,
            pitch: -30.0,
        }),
    }
}

#[test]
fn a_hello_and_an_event_survive_encoding() {
    let hello = FromPlugin {
        body: Some(from_plugin::Body::Hello(Hello {
            protocol_version: PROTOCOL_VERSION,
            name: "welcome".to_owned(),
            version: "0.1.0".to_owned(),
            token: String::new(),
            handlers: vec![Handler {
                id: 1,
                event: EventKind::PlayerMove as i32,
                access: Access::Decide as i32,
                priority: Priority::Normal as i32,
                ignore_cancelled: false,
            }],
        })),
    };
    let decoded = FromPlugin::decode(hello.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded, hello);

    let event = ToPlugin {
        body: Some(to_plugin::Body::Event(Event {
            id: 7,
            handlers: vec![1],
            phase: Phase::Decide as i32,
            cancelled: false,
            payload: Some(event::Payload::PlayerMove(PlayerMove {
                player: Some(steve()),
                from: Some(Vec3 {
                    x: 1.0,
                    y: 64.0,
                    z: -3.5,
                }),
                to: Some(Vec3 {
                    x: 2.0,
                    y: 64.0,
                    z: -3.5,
                }),
            })),
        })),
    };
    let decoded = ToPlugin::decode(event.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded, event);
}

#[test]
fn the_protocol_is_version_three() {
    assert_eq!(PROTOCOL_VERSION, 3);
}

#[test]
fn a_decision_carries_one_verdict_per_handler_with_its_patch() {
    let decision = FromPlugin {
        body: Some(from_plugin::Body::Decision(Decision {
            event: 7,
            verdicts: vec![
                Verdict {
                    handler: 2,
                    cancel: false,
                    patch: Some(Patch {
                        change: Some(patch::Change::PlayerMove(PlayerMovePatch {
                            to: Some(Vec3 {
                                x: 4.5,
                                y: 70.0,
                                z: -1.5,
                            }),
                        })),
                    }),
                },
                Verdict {
                    handler: 4,
                    cancel: true,
                    patch: None,
                },
                Verdict {
                    handler: 5,
                    cancel: false,
                    patch: Some(Patch {
                        change: Some(patch::Change::PlayerMove(PlayerMovePatch { to: None })),
                    }),
                },
            ],
        })),
    };
    let decoded = FromPlugin::decode(decision.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded, decision);
}

#[test]
fn a_patch_shares_its_field_number_with_the_event_payload() {
    let moved = Event {
        payload: Some(event::Payload::PlayerMove(PlayerMove::default())),
        ..Event::default()
    };
    let patched = Patch {
        change: Some(patch::Change::PlayerMove(PlayerMovePatch::default())),
    };
    assert_eq!(moved.encode_to_vec()[0] >> 3, 12);
    assert_eq!(patched.encode_to_vec()[0] >> 3, 12);
}
