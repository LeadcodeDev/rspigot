use rspigot_core::plugins::{
    ClickKind, ClickedSlot, CustomInventoryOrder, CustomInventoryState, ItemSpec, Order,
    PlayerSnapshot,
};
use rspigot_plugin_protocol::v1::{
    self, clicked_slot, custom_inventory_order::Order as Wire, event,
};

use super::Malformed;
use super::player::wire_player;
use super::shared::{self, required};

pub(super) fn order(order: v1::CustomInventoryOrder) -> Result<Order, Malformed> {
    let inventory = order.inventory;
    let order = match required(order.order)? {
        Wire::Set(v1::SetSlot { slot, item }) => CustomInventoryOrder::Set {
            slot,
            item: item.map(shared::item),
        },
        Wire::Clear(_) => CustomInventoryOrder::Clear,
        Wire::Close(_) => CustomInventoryOrder::Close,
        Wire::Discard(_) => CustomInventoryOrder::Discard,
    };
    Ok(Order::CustomInventory { inventory, order })
}

pub(super) fn wire_state(state: CustomInventoryState) -> v1::CustomInventoryState {
    v1::CustomInventoryState {
        slots: (0..)
            .zip(state.items)
            .map(|(slot, item)| v1::Slot {
                slot,
                item: item.map(shared::wire_item),
            })
            .collect(),
        viewers: state.viewers.into_iter().map(wire_player).collect(),
    }
}

pub(super) fn wire_click(
    player: PlayerSnapshot,
    inventory: u64,
    slot: ClickedSlot,
    kind: ClickKind,
    item: Option<ItemSpec>,
) -> event::Payload {
    let (kind, hotbar_key) = wire_kind(kind);
    event::Payload::InventoryClick(v1::InventoryClick {
        player: Some(wire_player(player)),
        inventory,
        slot: Some(wire_slot(slot)),
        kind: kind as i32,
        hotbar_key,
        item: item.map(shared::wire_item),
    })
}

pub(super) fn wire_close(
    player: PlayerSnapshot,
    inventory: u64,
    by_player: bool,
) -> event::Payload {
    event::Payload::InventoryClose(v1::InventoryClose {
        player: Some(wire_player(player)),
        inventory,
        by_player,
    })
}

fn wire_slot(slot: ClickedSlot) -> v1::ClickedSlot {
    let slot = match slot {
        ClickedSlot::Menu(index) => clicked_slot::Slot::Menu(i32::from(index)),
        ClickedSlot::Player(spigot) => clicked_slot::Slot::Player(spigot),
        ClickedSlot::Outside => clicked_slot::Slot::Outside(v1::Empty {}),
    };
    v1::ClickedSlot { slot: Some(slot) }
}

const fn wire_kind(kind: ClickKind) -> (v1::ClickKind, i32) {
    match kind {
        ClickKind::Left => (v1::ClickKind::Left, 0),
        ClickKind::Right => (v1::ClickKind::Right, 0),
        ClickKind::ShiftLeft => (v1::ClickKind::ShiftLeft, 0),
        ClickKind::ShiftRight => (v1::ClickKind::ShiftRight, 0),
        ClickKind::NumberKey(key) => (v1::ClickKind::NumberKey, key as i32),
        ClickKind::SwapOffhand => (v1::ClickKind::SwapOffhand, 0),
        ClickKind::Drop => (v1::ClickKind::Drop, 0),
        ClickKind::ControlDrop => (v1::ClickKind::ControlDrop, 0),
        ClickKind::DoubleClick => (v1::ClickKind::DoubleClick, 0),
        ClickKind::Middle => (v1::ClickKind::Middle, 0),
    }
}

#[cfg(test)]
mod tests {
    use rspigot_core::plugins::{
        Answer, ClickKind, ClickedSlot, CustomInventoryOrder, CustomInventoryState, EventKind,
        EventPayload, FromPlugin, ItemSpec, Order, Phase, PlayerOrder, PlayerSnapshot, Question,
        ServerOrder, ToPlugin,
    };
    use rspigot_domain::player::{GameMode, PlayerId};
    use rspigot_domain::{Rotation, Vec3};
    use rspigot_plugin_protocol::v1::{
        self, answer, clicked_slot, custom_inventory_order, event, player_order, server_order,
    };
    use uuid::Uuid;

    use crate::convert::player::wire_player;
    use crate::convert::{from_plugin, intro, question, to_plugin};

    const STEVE: &str = "00000000-0000-0000-0000-000000000001";

    fn steve() -> PlayerSnapshot {
        PlayerSnapshot {
            id: PlayerId(Uuid::from_u128(1)),
            name: "Steve".to_owned(),
            entity_id: 12,
            world: "minecraft:overworld".to_owned(),
            position: Vec3::new(1.0, 64.0, -3.5),
            rotation: Rotation::new(90.0, -30.0),
            game_mode: GameMode::Survival,
        }
    }

    fn convert(body: from_plugin::Body) -> FromPlugin {
        from_plugin(v1::FromPlugin { body: Some(body) }).expect("the message converts")
    }

    fn payload(payload: EventPayload) -> event::Payload {
        let Some(to_plugin::Body::Event(v1::Event {
            payload: Some(payload),
            ..
        })) = to_plugin(ToPlugin::Event {
            id: 4,
            handlers: vec![1],
            phase: Phase::Settled,
            cancelled: false,
            payload,
        })
        .body
        else {
            panic!("expected an event");
        };
        payload
    }

    fn wire_diamond() -> v1::Item {
        v1::Item {
            key: "minecraft:diamond".into(),
            count: 1,
            ..v1::Item::default()
        }
    }

    #[test]
    fn inventory_events_convert_to_the_wire() {
        let clicked = payload(EventPayload::InventoryClick {
            player: steve(),
            inventory: 7,
            slot: ClickedSlot::Player(39),
            kind: ClickKind::NumberKey(3),
            item: Some(ItemSpec::of("minecraft:diamond", 1)),
        });
        assert_eq!(
            clicked,
            event::Payload::InventoryClick(v1::InventoryClick {
                player: Some(wire_player(steve())),
                inventory: 7,
                slot: Some(v1::ClickedSlot {
                    slot: Some(clicked_slot::Slot::Player(39)),
                }),
                kind: v1::ClickKind::NumberKey as i32,
                hotbar_key: 3,
                item: Some(wire_diamond()),
            })
        );
        for (slot, wire) in [
            (ClickedSlot::Menu(53), clicked_slot::Slot::Menu(53)),
            (
                ClickedSlot::Outside,
                clicked_slot::Slot::Outside(v1::Empty {}),
            ),
        ] {
            let event::Payload::InventoryClick(click) = payload(EventPayload::InventoryClick {
                player: steve(),
                inventory: 7,
                slot,
                kind: ClickKind::ShiftRight,
                item: None,
            }) else {
                panic!("expected a click");
            };
            assert_eq!(click.slot, Some(v1::ClickedSlot { slot: Some(wire) }));
            assert_eq!(click.kind, v1::ClickKind::ShiftRight as i32);
            assert_eq!(click.hotbar_key, 0);
            assert_eq!(click.item, None);
        }
        let kinds = [
            (ClickKind::Left, v1::ClickKind::Left),
            (ClickKind::Right, v1::ClickKind::Right),
            (ClickKind::ShiftLeft, v1::ClickKind::ShiftLeft),
            (ClickKind::SwapOffhand, v1::ClickKind::SwapOffhand),
            (ClickKind::Drop, v1::ClickKind::Drop),
            (ClickKind::ControlDrop, v1::ClickKind::ControlDrop),
            (ClickKind::DoubleClick, v1::ClickKind::DoubleClick),
            (ClickKind::Middle, v1::ClickKind::Middle),
        ];
        for (kind, wire) in kinds {
            let event::Payload::InventoryClick(click) = payload(EventPayload::InventoryClick {
                player: steve(),
                inventory: 7,
                slot: ClickedSlot::Menu(0),
                kind,
                item: None,
            }) else {
                panic!("expected a click");
            };
            assert_eq!(click.kind, wire as i32);
        }

        assert_eq!(
            payload(EventPayload::InventoryClose {
                player: steve(),
                inventory: 7,
                by_player: true,
            }),
            event::Payload::InventoryClose(v1::InventoryClose {
                player: Some(wire_player(steve())),
                inventory: 7,
                by_player: true,
            })
        );
    }

    #[test]
    fn custom_inventory_calls_convert() {
        assert_eq!(
            convert(from_plugin::Body::Server(v1::ServerOrder {
                order: Some(server_order::Order::CreateInventory(v1::CreateInventory {
                    id: 3,
                    rows: 2,
                    title: "§6Menu".into(),
                    locked: true,
                })),
            })),
            FromPlugin::Order(Order::Server(ServerOrder::CreateInventory {
                id: 3,
                rows: 2,
                title: "§6Menu".into(),
                locked: true,
            }))
        );
        assert_eq!(
            convert(from_plugin::Body::Player(v1::PlayerOrder {
                player: STEVE.into(),
                order: Some(player_order::Order::OpenInventory(3)),
            })),
            FromPlugin::Order(Order::Player {
                player: PlayerId(Uuid::from_u128(1)),
                order: PlayerOrder::OpenInventory(3),
            })
        );
        let custom = |order| {
            convert(from_plugin::Body::CustomInventory(
                v1::CustomInventoryOrder {
                    inventory: 3,
                    order: Some(order),
                },
            ))
        };
        let on_three = |order| {
            FromPlugin::Order(Order::CustomInventory {
                inventory: 3,
                order,
            })
        };
        assert_eq!(
            custom(custom_inventory_order::Order::Set(v1::SetSlot {
                slot: 4,
                item: Some(wire_diamond()),
            })),
            on_three(CustomInventoryOrder::Set {
                slot: 4,
                item: Some(ItemSpec::of("minecraft:diamond", 1)),
            })
        );
        assert_eq!(
            custom(custom_inventory_order::Order::Set(v1::SetSlot {
                slot: 4,
                item: None,
            })),
            on_three(CustomInventoryOrder::Set {
                slot: 4,
                item: None
            })
        );
        assert_eq!(
            custom(custom_inventory_order::Order::Clear(v1::Empty {})),
            on_three(CustomInventoryOrder::Clear)
        );
        assert_eq!(
            custom(custom_inventory_order::Order::Close(v1::Empty {})),
            on_three(CustomInventoryOrder::Close)
        );
        assert_eq!(
            custom(custom_inventory_order::Order::Discard(v1::Empty {})),
            on_three(CustomInventoryOrder::Discard)
        );
        assert!(
            from_plugin(v1::FromPlugin {
                body: Some(from_plugin::Body::CustomInventory(
                    v1::CustomInventoryOrder {
                        inventory: 3,
                        order: None,
                    }
                )),
            })
            .is_err()
        );

        assert_eq!(
            convert(from_plugin::Body::Question(v1::Question {
                request: 2,
                question: Some(question::Question::CustomInventory(3)),
            })),
            FromPlugin::Question {
                request: 2,
                question: Question::CustomInventory(3),
            }
        );
        let Some(to_plugin::Body::Answer(v1::Answer {
            request: 2,
            body: Some(body),
        })) = to_plugin(ToPlugin::Answer {
            request: 2,
            answer: Answer::CustomInventory(CustomInventoryState {
                items: vec![None, Some(ItemSpec::of("minecraft:diamond", 1))],
                viewers: vec![steve()],
            }),
        })
        .body
        else {
            panic!("expected an answer");
        };
        assert_eq!(
            body,
            answer::Body::CustomInventory(v1::CustomInventoryState {
                slots: vec![
                    v1::Slot {
                        slot: 0,
                        item: None
                    },
                    v1::Slot {
                        slot: 1,
                        item: Some(wire_diamond())
                    },
                ],
                viewers: vec![wire_player(steve())],
            })
        );
    }

    #[test]
    fn inventory_handlers_are_accepted() {
        let handler = |id, event: v1::EventKind| v1::Handler {
            id,
            event: event as i32,
            access: v1::Access::Observe as i32,
            priority: v1::Priority::Normal as i32,
            ignore_cancelled: false,
        };
        let introduced = intro(v1::Hello {
            protocol_version: 2,
            name: "menus".into(),
            version: "1.0.0".into(),
            token: String::new(),
            handlers: vec![
                handler(1, v1::EventKind::InventoryClick),
                handler(2, v1::EventKind::InventoryClose),
            ],
        })
        .expect("the introduction converts");
        let events: Vec<EventKind> = introduced
            .handlers
            .iter()
            .map(|handler| handler.event)
            .collect();
        assert_eq!(
            events,
            [EventKind::InventoryClick, EventKind::InventoryClose]
        );
    }
}
