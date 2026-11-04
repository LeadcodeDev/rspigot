use rspigot_items::{ItemId, ItemKind};

use crate::item::{ItemStack, Slot, count, max_stack, same_item_same_components, with_count};
use crate::menu::{ClickInput, OUTSIDE};

const OFFHAND_BUTTON: i32 = 40;
const HOTBAR_KEYS: std::ops::Range<i32> = 0..9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    Left,
    Right,
    ShiftLeft,
    ShiftRight,
    NumberKey,
    SwapOffhand,
    Drop,
    ControlDrop,
    DoubleClick,
    Middle,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    Nothing,
    PickupAll,
    PickupSome,
    PickupHalf,
    PickupOne,
    PlaceAll,
    PlaceSome,
    PlaceOne,
    SwapWithCursor,
    DropAllCursor,
    DropOneCursor,
    DropAllSlot,
    DropOneSlot,
    MoveToOtherInventory,
    HotbarMoveAndReadd,
    HotbarSwap,
    CloneStack,
    CollectToCursor,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Classified {
    pub click: ClickType,
    pub action: ClickAction,
}

impl Classified {
    const fn of(click: ClickType, action: ClickAction) -> Self {
        Self { click, action }
    }

    const UNKNOWN: Self = Self::of(ClickType::Unknown, ClickAction::Unknown);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Drag {
    pub single: bool,
    pub cursor: ItemStack,
    pub placed: Vec<(usize, ItemStack)>,
    pub before: Vec<(usize, Slot)>,
    pub remaining: Slot,
}

pub(crate) trait ClickView {
    fn slot(&self, raw: i32) -> Option<usize>;
    fn item(&self, slot: usize) -> Slot;
    fn carried(&self) -> &Slot;
    fn may_pickup(&self, slot: usize) -> bool;
    fn may_place(&self, slot: usize, stack: &ItemStack) -> bool;
    fn slot_limit(&self, slot: usize) -> u32;
    fn stack_limit(&self, slot: usize, stack: &ItemStack) -> u32;
    fn container_limit(&self, slot: usize) -> u32;
    fn hotbar(&self, button: i32) -> Slot;
    fn in_player(&self, slot: usize) -> bool;
    fn holds(&self, item: ItemId) -> bool;
    fn instabuild(&self) -> bool;
}

pub(crate) fn classify(
    view: &impl ClickView,
    slot: i16,
    button: i8,
    input: ClickInput,
) -> Classified {
    let raw = i32::from(slot);
    let button = i32::from(button);
    match input {
        ClickInput::Pickup => pickup(view, raw, button),
        ClickInput::QuickMove => quick_move(view, raw, button),
        ClickInput::Swap => swap(view, raw, button),
        ClickInput::Clone => clone_stack(view, raw, button),
        ClickInput::Throw => throw(view, raw, button),
        ClickInput::PickupAll => pickup_all(view, raw),
        ClickInput::QuickCraft => Classified::UNKNOWN,
    }
}

fn pickup(view: &impl ClickView, raw: i32, button: i32) -> Classified {
    let click = match button {
        0 => ClickType::Left,
        1 => ClickType::Right,
        _ => return Classified::UNKNOWN,
    };
    let primary = button == 0;
    let carried = view.carried();
    let action = if raw == i32::from(OUTSIDE) {
        match carried {
            Some(_) if primary => ClickAction::DropAllCursor,
            Some(_) => ClickAction::DropOneCursor,
            None => ClickAction::Nothing,
        }
    } else {
        match view.slot(raw) {
            Some(slot) => pickup_action(view, slot, primary),
            None => ClickAction::Nothing,
        }
    };
    Classified::of(click, action)
}

fn pickup_action(view: &impl ClickView, slot: usize, primary: bool) -> ClickAction {
    let clicked = view.item(slot);
    let Some(clicked) = clicked else {
        return match view.carried() {
            Some(_) if primary => ClickAction::PlaceAll,
            Some(_) => ClickAction::PlaceOne,
            None => ClickAction::Nothing,
        };
    };
    if !view.may_pickup(slot) {
        return ClickAction::Nothing;
    }
    let Some(cursor) = view.carried() else {
        return if primary {
            ClickAction::PickupAll
        } else {
            ClickAction::PickupHalf
        };
    };
    let held = i64::from(cursor.count.get());
    let inside = i64::from(clicked.count.get());
    if view.may_place(slot, cursor) {
        if !same_item_same_components(&clicked, cursor) {
            return if u32::from(cursor.count.get()) <= view.slot_limit(slot) {
                ClickAction::SwapWithCursor
            } else {
                ClickAction::Nothing
            };
        }
        let wanted = if primary { held } else { 1 };
        let placed = wanted
            .min(i64::from(max_stack(&clicked)) - inside)
            .min(i64::from(view.container_limit(slot)) - inside);
        return match placed {
            1 => ClickAction::PlaceOne,
            n if n == held => ClickAction::PlaceAll,
            -1 => ClickAction::PickupOne,
            n if n < 0 => ClickAction::PickupSome,
            0 => ClickAction::Nothing,
            _ => ClickAction::PlaceSome,
        };
    }
    if same_item_same_components(cursor, &clicked) && inside + held <= i64::from(max_stack(cursor))
    {
        ClickAction::PickupAll
    } else {
        ClickAction::Nothing
    }
}

fn quick_move(view: &impl ClickView, raw: i32, button: i32) -> Classified {
    let click = match button {
        0 => ClickType::ShiftLeft,
        1 => ClickType::ShiftRight,
        _ => return Classified::UNKNOWN,
    };
    let movable = view
        .slot(raw)
        .is_some_and(|slot| view.may_pickup(slot) && view.item(slot).is_some());
    let action = if movable {
        ClickAction::MoveToOtherInventory
    } else {
        ClickAction::Nothing
    };
    Classified::of(click, action)
}

fn swap(view: &impl ClickView, raw: i32, button: i32) -> Classified {
    let click = if button == OFFHAND_BUTTON {
        ClickType::SwapOffhand
    } else if HOTBAR_KEYS.contains(&button) {
        ClickType::NumberKey
    } else {
        return Classified::UNKNOWN;
    };
    let Some(slot) = view.slot(raw).filter(|slot| view.may_pickup(*slot)) else {
        return Classified::of(click, ClickAction::Nothing);
    };
    let hotbar = view.hotbar(button);
    let fits = |stack: &ItemStack| view.may_place(slot, stack);
    let allowed = hotbar
        .as_ref()
        .is_none_or(|stack| view.in_player(slot) && fits(stack));
    let action = if view.item(slot).is_some() {
        if allowed {
            ClickAction::HotbarSwap
        } else {
            ClickAction::HotbarMoveAndReadd
        }
    } else if hotbar.as_ref().is_some_and(fits) {
        ClickAction::HotbarSwap
    } else {
        ClickAction::Nothing
    };
    Classified::of(click, action)
}

fn clone_stack(view: &impl ClickView, raw: i32, button: i32) -> Classified {
    if button != 2 {
        return Classified::UNKNOWN;
    }
    let clonable = view.slot(raw).is_some_and(|slot| view.item(slot).is_some())
        && view.instabuild()
        && view.carried().is_none();
    let action = if clonable {
        ClickAction::CloneStack
    } else {
        ClickAction::Nothing
    };
    Classified::of(ClickType::Middle, action)
}

fn throw(view: &impl ClickView, raw: i32, button: i32) -> Classified {
    if raw < 0 {
        let click = if button == 1 {
            ClickType::Right
        } else {
            ClickType::Left
        };
        return Classified::of(click, ClickAction::Nothing);
    }
    let (click, dropped) = match button {
        0 => (ClickType::Drop, ClickAction::DropOneSlot),
        1 => (ClickType::ControlDrop, ClickAction::DropAllSlot),
        _ => return Classified::UNKNOWN,
    };
    let droppable = view.slot(raw).is_some_and(|slot| {
        view.may_pickup(slot)
            && view
                .item(slot)
                .is_some_and(|stack| stack.item != ItemKind::Air.id())
    });
    let action = if droppable {
        dropped
    } else {
        ClickAction::Nothing
    };
    Classified::of(click, action)
}

fn pickup_all(view: &impl ClickView, raw: i32) -> Classified {
    let gathered = raw >= 0
        && view
            .carried()
            .as_ref()
            .is_some_and(|cursor| view.holds(cursor.item));
    let action = if gathered {
        ClickAction::CollectToCursor
    } else {
        ClickAction::Nothing
    };
    Classified::of(ClickType::DoubleClick, action)
}

fn quick_replaceable(view: &impl ClickView, slot: usize, stack: &ItemStack) -> bool {
    view.item(slot).is_none_or(|existing| {
        same_item_same_components(stack, &existing)
            && u32::from(existing.count.get()) <= max_stack(stack)
    })
}

fn place_count(slots: usize, kind: i32, stack: &ItemStack) -> u32 {
    match kind {
        0 => u32::from(stack.count.get()) / u32::try_from(slots).unwrap_or(u32::MAX).max(1),
        1 => 1,
        2 => max_stack(stack),
        _ => u32::from(stack.count.get()),
    }
}

pub(crate) fn plan_drag(
    view: &impl ClickView,
    slots: &[usize],
    kind: i32,
    copy: &ItemStack,
) -> Drag {
    let spread = slots.len();
    let mut remaining = i64::from(copy.count.get());
    let mut placed = Vec::new();
    let mut before = Vec::new();
    for &target in slots {
        let wide =
            kind == 2 || u32::from(copy.count.get()) >= u32::try_from(spread).unwrap_or(u32::MAX);
        if !quick_replaceable(view, target, copy) || !view.may_place(target, copy) || !wide {
            continue;
        }
        let existing = count(&view.item(target));
        let limit = max_stack(copy).min(view.stack_limit(target, copy));
        let total = (place_count(spread, kind, copy) + existing).min(limit);
        remaining -= i64::from(total) - i64::from(existing);
        before.push((target, view.item(target)));
        if let Some(stack) = with_count(copy, total) {
            placed.push((target, stack));
        }
    }
    Drag {
        single: kind == 1,
        cursor: copy.clone(),
        placed,
        before,
        remaining: with_count(copy, u32::try_from(remaining.max(0)).unwrap_or(0)),
    }
}
