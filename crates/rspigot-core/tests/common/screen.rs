#![allow(dead_code)]

use std::num::NonZeroU8;

use rspigot_core::events::{
    ClickInput, ClientEvent, ContainerClick, Direction, Hand, HashedSlot, Inbound, PlayerInput,
};
use rspigot_core::plugins::{BlockOrder, InventoryOrder, ItemSpec, Order};
use rspigot_domain::item::{ItemStack, Slot};
use rspigot_domain::{BlockPos, Vec3};
use rspigot_items::ItemKind;

use super::{Player, Scene};

pub fn stack(key: &str, count: u8) -> Slot {
    let kind = ItemKind::from_name(key).expect("a known item");
    Some(ItemStack::new(
        kind.id(),
        NonZeroU8::new(count).expect("a count"),
    ))
}

pub fn shown(slot: &Slot) -> Option<(&'static str, u8)> {
    slot.as_ref().map(|stack| {
        (
            stack.item.kind().map_or("?", ItemKind::name),
            stack.count.get(),
        )
    })
}

pub fn spec(key: &str, count: i32) -> ItemSpec {
    ItemSpec::of(key, count)
}

pub fn set_block(s: &mut Scene, pos: BlockPos, key: &str) {
    s.act(Order::Block {
        pos,
        order: BlockOrder::Set(key.to_owned()),
    });
}

pub fn give(s: &mut Scene, player: &Player, slot: i32, key: &str, count: i32) {
    s.act(Order::Inventory {
        player: player.id,
        order: InventoryOrder::Set {
            slot,
            item: Some(spec(key, count)),
        },
    });
}

#[derive(Debug, Clone)]
pub struct View {
    pub window: i32,
    pub state: i32,
    pub slots: Vec<Slot>,
    pub cursor: Slot,
    pub opened: bool,
    pub data: Vec<(i16, i16)>,
}

impl View {
    pub fn inventory() -> Self {
        Self {
            window: 0,
            state: 0,
            slots: vec![None; 46],
            cursor: None,
            opened: true,
            data: Vec::new(),
        }
    }

    pub fn apply(&mut self, events: &[ClientEvent]) {
        for event in events {
            match event {
                ClientEvent::ContainerContent {
                    window,
                    state_id,
                    slots,
                    carried,
                } if *window == self.window => {
                    self.state = *state_id;
                    self.slots.clone_from(slots);
                    self.cursor.clone_from(carried);
                }
                ClientEvent::ContainerSlot {
                    window,
                    state_id,
                    slot,
                    stack,
                } if *window == self.window => {
                    self.state = *state_id;
                    self.put(*slot, stack.clone());
                }
                ClientEvent::InventoryContent {
                    state_id,
                    slots,
                    carried,
                } if self.window == 0 => {
                    self.state = *state_id;
                    self.slots.clone_from(slots);
                    self.cursor.clone_from(carried);
                }
                ClientEvent::InventorySlot {
                    state_id,
                    slot,
                    stack,
                } if self.window == 0 => {
                    self.state = *state_id;
                    self.put(*slot, stack.clone());
                }
                ClientEvent::CursorItem(stack) => self.cursor.clone_from(stack),
                ClientEvent::ContainerData { window, id, value } if *window == self.window => {
                    self.data.push((*id, *value));
                }
                ClientEvent::ContainerClose(window) if *window == self.window => {
                    self.opened = false;
                }
                _ => {}
            }
        }
    }

    fn put(&mut self, slot: i16, stack: Slot) {
        if let Some(entry) = usize::try_from(slot)
            .ok()
            .and_then(|index| self.slots.get_mut(index))
        {
            *entry = stack;
        }
    }

    pub fn at(&self, slot: usize) -> Option<(&'static str, u8)> {
        shown(&self.slots[slot])
    }

    pub fn held(&self) -> Option<(&'static str, u8)> {
        shown(&self.cursor)
    }
}

pub fn press(slot: i16, button: i8, input: ClickInput) -> ContainerClick {
    ContainerClick {
        container: 0,
        state_id: 0,
        slot,
        button,
        input,
        changed: Vec::new(),
        carried: HashedSlot::EMPTY,
    }
}

pub fn predict(
    view: &mut View,
    click: ContainerClick,
    changed: &[(i16, Slot)],
    cursor: Slot,
) -> ContainerClick {
    for (slot, stack) in changed {
        view.put(*slot, stack.clone());
    }
    let carried = HashedSlot::of(&cursor);
    view.cursor = cursor;
    ContainerClick {
        changed: changed
            .iter()
            .map(|(slot, stack)| (*slot, HashedSlot::of(stack)))
            .collect(),
        carried,
        ..click
    }
}

pub fn send_click(s: &mut Scene, player: &Player, view: &mut View, click: ContainerClick) {
    s.server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::ContainerClick(ContainerClick {
            container: view.window,
            state_id: view.state,
            ..click
        }),
    });
    s.server.deliver();
    view.apply(&player.link.take());
}

pub fn refresh(player: &Player, view: &mut View) {
    view.apply(&player.link.take());
}

pub fn use_block(s: &mut Scene, player: &Player, pos: BlockPos) {
    s.server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::UseItemOn {
            hand: Hand::Main,
            pos,
            face: Direction::Up,
            cursor: Vec3::new(0.5, 1.0, 0.5),
            inside_block: false,
            sequence: 1,
        },
    });
    s.server.deliver();
}

pub fn opened_view(player: &Player) -> Option<View> {
    let events = player.link.take();
    let window = events.iter().find_map(|event| match event {
        ClientEvent::OpenScreen { window, .. } => Some(*window),
        _ => None,
    })?;
    let mut view = View {
        window,
        state: 0,
        slots: Vec::new(),
        cursor: None,
        opened: true,
        data: Vec::new(),
    };
    view.apply(&events);
    Some(view)
}

pub fn open_block(s: &mut Scene, player: &Player, pos: BlockPos) -> View {
    use_block(s, player, pos);
    opened_view(player).expect("the block opens a menu")
}

pub fn close_window(s: &mut Scene, player: &Player, view: &mut View) {
    s.server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::ContainerClose(view.window),
    });
    s.server.deliver();
    view.apply(&player.link.take());
}
