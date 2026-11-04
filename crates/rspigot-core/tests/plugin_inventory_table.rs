mod common;
#[path = "common/screen.rs"]
mod screen;

use common::*;
use rspigot_core::events::ClickInput;
use rspigot_core::plugins::events::{ClickKind, InventoryAction, InventoryTypeSlotType};
use rspigot_core::plugins::{
    CustomInventoryOrder, EventKind, EventPayload, Order, Phase, PlayerOrder, ServerOrder,
};
use rspigot_domain::BlockPos;
use rspigot_domain::player::GameMode;
use screen::*;

const CHEST: BlockPos = BlockPos::new(2, 64, 0);
const TABLE: BlockPos = BlockPos::new(-2, 64, 0);
const MENU: u64 = 3;
const STONE: &str = "minecraft:stone";
const DIRT: &str = "minecraft:dirt";
const LOG: &str = "minecraft:oak_log";
const PLANKS: &str = "minecraft:oak_planks";
const SEED_HOTBAR: i32 = 8;
const CURSOR_HOTBAR: i32 = 7;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Place {
    Chest,
    Table,
    Plugin,
}

impl Place {
    const fn top(self) -> i16 {
        match self {
            Self::Chest | Self::Plugin => 27,
            Self::Table => 10,
        }
    }

    const fn hotbar(self, key: i32) -> i16 {
        self.top() + 27 + key as i16
    }
}

struct Row {
    place: Place,
    seed: &'static [(i16, &'static str, u8)],
    hotbar: &'static [(i32, &'static str, i32)],
    cursor: Option<(&'static str, u8)>,
    creative: bool,
    slot: i16,
    button: i8,
    input: ClickInput,
    expected: (ClickKind, InventoryAction, InventoryTypeSlotType),
}

const fn row(
    place: Place,
    seed: &'static [(i16, &'static str, u8)],
    cursor: Option<(&'static str, u8)>,
    slot: i16,
    button: i8,
    input: ClickInput,
    expected: (ClickKind, InventoryAction, InventoryTypeSlotType),
) -> Row {
    Row {
        place,
        seed,
        hotbar: &[],
        cursor,
        creative: false,
        slot,
        button,
        input,
        expected,
    }
}

fn heard(s: &Scene) -> Vec<(ClickKind, InventoryAction, InventoryTypeSlotType)> {
    fired(&s.plugin)
        .into_iter()
        .filter(|event| event.phase == Phase::Settled)
        .filter_map(|event| match event.payload {
            EventPayload::InventoryClick(click) => {
                Some((click.click, click.action, click.slot_type))
            }
            EventPayload::CraftItem(craft) => Some((craft.click, craft.action, craft.slot_type)),
            _ => None,
        })
        .collect()
}

fn carry(s: &mut Scene, alice: &Player, view: &mut View, from: i16, item: (&str, u8)) {
    let take = predict(
        view,
        press(from, 0, ClickInput::Pickup),
        &[(from, None)],
        stack(item.0, item.1),
    );
    send_click(s, alice, view, take);
}

fn play(row: &Row) -> Vec<(ClickKind, InventoryAction, InventoryTypeSlotType)> {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::InventoryClick)]);
    let alice = s.players[0].clone();
    if row.creative {
        s.act(to(&alice, PlayerOrder::SetGameMode(GameMode::Creative)));
    }
    for (slot, key, count) in row.hotbar {
        give(&mut s, &alice, *slot, key, *count);
    }
    let mut view = match row.place {
        Place::Chest => {
            set_block(&mut s, CHEST, "minecraft:chest");
            alice.link.take();
            open_block(&mut s, &alice, CHEST)
        }
        Place::Table => {
            set_block(&mut s, TABLE, "minecraft:crafting_table");
            alice.link.take();
            open_block(&mut s, &alice, TABLE)
        }
        Place::Plugin => {
            s.act(Order::Server(ServerOrder::CreateInventory {
                id: MENU,
                rows: 3,
                title: "Box".to_owned(),
                locked: false,
            }));
            for (slot, key, count) in row.seed {
                s.act(Order::CustomInventory {
                    inventory: MENU,
                    order: CustomInventoryOrder::Set {
                        slot: i32::from(*slot),
                        item: Some(spec(key, i32::from(*count))),
                    },
                });
            }
            alice.link.take();
            s.act(to(&alice, PlayerOrder::OpenInventory(MENU)));
            opened_view(&alice).expect("the plugin menu opens")
        }
    };
    if row.place != Place::Plugin {
        for (slot, key, count) in row.seed {
            give(&mut s, &alice, SEED_HOTBAR, key, i32::from(*count));
            refresh(&alice, &mut view);
            carry(
                &mut s,
                &alice,
                &mut view,
                row.place.hotbar(SEED_HOTBAR),
                (key, *count),
            );
            let put = predict(
                &mut view,
                press(*slot, 0, ClickInput::Pickup),
                &[(*slot, stack(key, *count))],
                None,
            );
            send_click(&mut s, &alice, &mut view, put);
        }
    }
    if let Some((key, count)) = row.cursor {
        give(&mut s, &alice, CURSOR_HOTBAR, key, i32::from(count));
        refresh(&alice, &mut view);
        carry(
            &mut s,
            &alice,
            &mut view,
            row.place.hotbar(CURSOR_HOTBAR),
            (key, count),
        );
    }
    let before = heard(&s).len();
    refresh(&alice, &mut view);
    send_click(
        &mut s,
        &alice,
        &mut view,
        press(row.slot, row.button, row.input),
    );
    heard(&s).split_off(before)
}

#[test]
fn every_click_row_is_classified_in_a_chest_a_crafting_table_and_a_plugin_inventory() {
    use ClickInput::{Clone, Pickup, PickupAll, QuickMove, Swap, Throw};
    use ClickKind as K;
    use InventoryAction as A;
    use InventoryTypeSlotType as T;
    use Place::{Chest, Plugin, Table};
    let stone5: &[(i16, &str, u8)] = &[(0, STONE, 5)];
    let dirt_key: &[(i32, &str, i32)] = &[(0, DIRT, 1)];
    let rows = [
        row(
            Chest,
            stone5,
            None,
            0,
            0,
            Pickup,
            (K::Left, A::PickupAll, T::Container),
        ),
        row(
            Chest,
            stone5,
            None,
            0,
            1,
            Pickup,
            (K::Right, A::PickupHalf, T::Container),
        ),
        row(
            Chest,
            &[],
            Some((STONE, 5)),
            0,
            0,
            Pickup,
            (K::Left, A::PlaceAll, T::Container),
        ),
        row(
            Chest,
            &[],
            Some((STONE, 5)),
            0,
            1,
            Pickup,
            (K::Right, A::PlaceOne, T::Container),
        ),
        row(
            Chest,
            stone5,
            Some((DIRT, 3)),
            0,
            0,
            Pickup,
            (K::Left, A::SwapWithCursor, T::Container),
        ),
        row(
            Chest,
            &[(0, STONE, 62)],
            Some((STONE, 5)),
            0,
            0,
            Pickup,
            (K::Left, A::PlaceSome, T::Container),
        ),
        row(
            Chest,
            stone5,
            None,
            0,
            0,
            QuickMove,
            (K::ShiftLeft, A::MoveToOtherInventory, T::Container),
        ),
        Row {
            hotbar: dirt_key,
            ..row(
                Chest,
                stone5,
                None,
                0,
                0,
                Swap,
                (K::NumberKey, A::HotbarMoveAndReadd, T::Container),
            )
        },
        Row {
            hotbar: dirt_key,
            ..row(
                Chest,
                &[],
                None,
                0,
                0,
                Swap,
                (K::NumberKey, A::HotbarSwap, T::Container),
            )
        },
        row(
            Chest,
            stone5,
            None,
            0,
            0,
            Throw,
            (K::Drop, A::DropOneSlot, T::Container),
        ),
        row(
            Chest,
            stone5,
            None,
            0,
            1,
            Throw,
            (K::ControlDrop, A::DropAllSlot, T::Container),
        ),
        row(
            Chest,
            stone5,
            Some((STONE, 2)),
            1,
            0,
            PickupAll,
            (K::DoubleClick, A::CollectToCursor, T::Container),
        ),
        row(
            Chest,
            &[],
            Some((STONE, 2)),
            -999,
            0,
            Pickup,
            (K::Left, A::DropAllCursor, T::Outside),
        ),
        row(
            Chest,
            stone5,
            None,
            54,
            0,
            QuickMove,
            (K::ShiftLeft, A::Nothing, T::Quickbar),
        ),
        Row {
            creative: true,
            ..row(
                Chest,
                stone5,
                None,
                0,
                2,
                Clone,
                (K::Middle, A::CloneStack, T::Container),
            )
        },
        row(
            Table,
            &[(1, LOG, 1)],
            None,
            0,
            0,
            Pickup,
            (K::Left, A::PickupAll, T::Result),
        ),
        row(
            Table,
            &[(1, LOG, 1)],
            Some((PLANKS, 2)),
            0,
            0,
            Pickup,
            (K::Left, A::PickupAll, T::Result),
        ),
        row(
            Table,
            &[(1, LOG, 1)],
            Some((STONE, 2)),
            0,
            0,
            Pickup,
            (K::Left, A::Nothing, T::Result),
        ),
        row(
            Table,
            &[(1, LOG, 1)],
            None,
            0,
            0,
            QuickMove,
            (K::ShiftLeft, A::MoveToOtherInventory, T::Result),
        ),
        row(
            Table,
            &[(1, LOG, 1)],
            None,
            0,
            0,
            Swap,
            (K::NumberKey, A::HotbarSwap, T::Result),
        ),
        row(
            Table,
            &[],
            Some((LOG, 1)),
            5,
            0,
            Pickup,
            (K::Left, A::PlaceAll, T::Crafting),
        ),
        row(
            Table,
            &[],
            None,
            10,
            0,
            Pickup,
            (K::Left, A::Nothing, T::Container),
        ),
        row(
            Plugin,
            stone5,
            None,
            0,
            0,
            Pickup,
            (K::Left, A::PickupAll, T::Container),
        ),
        row(
            Plugin,
            stone5,
            Some((DIRT, 3)),
            0,
            0,
            Pickup,
            (K::Left, A::SwapWithCursor, T::Container),
        ),
        row(
            Plugin,
            stone5,
            None,
            0,
            0,
            QuickMove,
            (K::ShiftLeft, A::MoveToOtherInventory, T::Container),
        ),
        Row {
            hotbar: dirt_key,
            ..row(
                Plugin,
                stone5,
                None,
                0,
                0,
                Swap,
                (K::NumberKey, A::HotbarMoveAndReadd, T::Container),
            )
        },
        row(
            Plugin,
            stone5,
            None,
            0,
            1,
            Throw,
            (K::ControlDrop, A::DropAllSlot, T::Container),
        ),
        row(
            Plugin,
            stone5,
            Some((STONE, 2)),
            1,
            0,
            PickupAll,
            (K::DoubleClick, A::CollectToCursor, T::Container),
        ),
        row(
            Plugin,
            &[],
            Some((STONE, 2)),
            -999,
            1,
            Pickup,
            (K::Right, A::DropOneCursor, T::Outside),
        ),
    ];
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(play(row), [row.expected], "row {index}");
    }
}
