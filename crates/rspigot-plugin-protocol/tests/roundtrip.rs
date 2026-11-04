use prost::Message;
use rspigot_plugin_protocol::PROTOCOL_VERSION;
use rspigot_plugin_protocol::v1::{
    Access, BlockPos, BlockSnapshot, BlockState, ClickKind, DamageSource, Decision,
    EntityDamageByEntity, EntityDamageCause, EntitySnapshot, EntityTargetPatch, Event, EventKind,
    EventResult, ExplosionResult, FromPlugin, GameMode, Handler, Hello, InventoryAction,
    InventoryClick, InventoryRef, InventoryTypeSlotType, InventoryView, Item, ItemList, Location,
    NullableEntitySnapshot, NullableString, Patch, Phase, PlayerDeath, PlayerDeathPatch,
    PlayerMove, PlayerMovePatch, PlayerSnapshot, Priority, Property, Rotation, ToPlugin, Vec3,
    Verdict, event, from_plugin, inventory_ref, patch, to_plugin,
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

fn zombie() -> EntitySnapshot {
    EntitySnapshot {
        entity_id: 40,
        uuid: "00000000-0000-0000-0000-000000000028".to_owned(),
        kind: "minecraft:zombie".to_owned(),
        position: Some(Vec3 {
            x: 2.0,
            y: 64.0,
            z: -3.5,
        }),
    }
}

fn at(x: f64) -> Location {
    Location {
        world: "minecraft:overworld".to_owned(),
        position: Some(Vec3 {
            x,
            y: 64.0,
            z: -3.5,
        }),
        rotation: Some(Rotation {
            yaw: 0.0,
            pitch: 0.0,
        }),
    }
}

fn diamond(count: i32) -> Item {
    Item {
        key: "minecraft:diamond".to_owned(),
        count,
        name: Some("Shiny".to_owned()),
        ..Item::default()
    }
}

fn custom(id: u64) -> InventoryRef {
    InventoryRef {
        owner: Some(inventory_ref::Owner::Custom(id)),
        size: 27,
        kind: "CHEST".to_owned(),
    }
}

fn event(payload: event::Payload) -> ToPlugin {
    ToPlugin {
        body: Some(to_plugin::Body::Event(Event {
            id: 7,
            handlers: vec![1, 2],
            phase: Phase::Decide as i32,
            cancelled: false,
            payload: Some(payload),
        })),
    }
}

fn survives(message: &ToPlugin) {
    let decoded = ToPlugin::decode(message.encode_to_vec().as_slice()).unwrap();
    assert_eq!(&decoded, message);
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
                event: EventKind::EntityDamageByEntity as i32,
                access: Access::Decide as i32,
                priority: Priority::Normal as i32,
                ignore_cancelled: false,
            }],
        })),
    };
    let decoded = FromPlugin::decode(hello.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded, hello);

    survives(&event(event::Payload::PlayerMove(PlayerMove {
        player: Some(steve()),
        from: Some(at(1.0)),
        to: Some(at(2.0)),
    })));
}

#[test]
fn every_kind_of_field_survives_encoding() {
    let block = BlockPos { x: 3, y: 64, z: -2 };
    let samples = [
        event::Payload::InventoryClick(InventoryClick {
            action: InventoryAction::PickupAll as i32,
            click: ClickKind::Creative as i32,
            clicked_inventory: Some(InventoryRef {
                owner: Some(inventory_ref::Owner::Player(steve().uuid)),
                size: 41,
                kind: "PLAYER".to_owned(),
            }),
            current_item: Some(diamond(3)),
            cursor: None,
            hotbar_button: -1,
            inventory: Some(custom(5)),
            left_click: true,
            raw_slot: 30,
            result: EventResult::Deny as i32,
            right_click: false,
            shift_click: false,
            slot: 3,
            slot_type: InventoryTypeSlotType::Quickbar as i32,
            view: Some(InventoryView {
                top: Some(custom(5)),
                bottom: Some(InventoryRef {
                    owner: Some(inventory_ref::Owner::Block(block)),
                    size: 9,
                    kind: "DISPENSER".to_owned(),
                }),
                title: "§6Menu".to_owned(),
                player: Some(steve()),
            }),
            viewers: vec![steve(), steve()],
            who_clicked: Some(steve()),
        }),
        event::Payload::EntityDamageByEntity(EntityDamageByEntity {
            cause: EntityDamageCause::EntityAttack as i32,
            damage: 4.5,
            damage_source: Some(DamageSource {
                kind: "minecraft:player_attack".to_owned(),
                causing: Some(zombie()),
                direct: Some(zombie()),
                location: None,
            }),
            damager: Some(zombie()),
            entity: Some(zombie()),
            entity_type: "minecraft:zombie".to_owned(),
            final_damage: 3.0,
        }),
        event::Payload::BlockExplode(rspigot_plugin_protocol::v1::BlockExplode {
            block: Some(block),
            block_list: vec![block, BlockPos { x: 4, y: 64, z: -2 }],
            exploded_block_state: Some(BlockSnapshot {
                pos: Some(block),
                state: Some(BlockState {
                    block: "minecraft:tnt".to_owned(),
                    properties: vec![Property {
                        name: "unstable".to_owned(),
                        value: "false".to_owned(),
                    }],
                }),
            }),
            explosion_result: ExplosionResult::DestroyWithDecay as i32,
            explosion_yield: 0.25,
        }),
        event::Payload::PlayerDeath(PlayerDeath {
            damage_source: None,
            death_message: Some("Steve fell".to_owned()),
            dropped_exp: 7,
            drops: vec![diamond(1), diamond(64)],
            entity: Some(steve()),
            entity_type: "minecraft:player".to_owned(),
            keep_inventory: true,
            keep_level: false,
            new_exp: 0,
            new_level: 3,
            new_total_exp: 30,
        }),
    ];
    for payload in samples {
        survives(&event(payload));
    }
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
                            to: Some(at(4.5)),
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
                        change: Some(patch::Change::PlayerDeath(PlayerDeathPatch {
                            death_message: Some(NullableString { value: None }),
                            drops: Some(ItemList { values: Vec::new() }),
                            keep_inventory: Some(true),
                            ..PlayerDeathPatch::default()
                        })),
                    }),
                },
                Verdict {
                    handler: 6,
                    cancel: false,
                    patch: Some(Patch {
                        change: Some(patch::Change::EntityTarget(EntityTargetPatch {
                            target: Some(NullableEntitySnapshot {
                                value: Some(zombie()),
                            }),
                        })),
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
    assert_eq!(EventKind::PlayerJoin as i32, 1);
    assert_eq!(EventKind::InventoryClose as i32, 5);
}
