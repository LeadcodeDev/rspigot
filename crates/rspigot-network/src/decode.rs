use std::num::NonZeroU8;

use rspigot_core::events::{DigAction, Direction, Hand, InputState, MovementAction, PlayerInput};
use rspigot_domain::item::ItemStack;
use rspigot_domain::player::GameMode;
use rspigot_domain::{BlockPos, EntityId, ItemId, Rotation, Vec3};
use rspigot_nbt::Compound;
use rspigot_protocol::codec::Position;
use rspigot_protocol::play::ServerboundPlay;
use rspigot_protocol::play::serverbound::{
    ClientCommandAction, PlayerActionKind, PlayerCommandAction,
};
use rspigot_protocol::play::slot::ItemStack as WireStack;
use rspigot_protocol::play::types::{
    Direction as WireDirection, GameMode as WireGameMode, Hand as WireHand,
};

use crate::connection::client_settings;

pub enum Decoded {
    Input(PlayerInput),
    KeepAlive(i64),
    Ping(i64),
    Ignored,
}

fn block_pos(pos: Position) -> BlockPos {
    BlockPos::new(pos.x, pos.y, pos.z)
}

fn hand(hand: WireHand) -> Hand {
    match hand {
        WireHand::MainHand => Hand::Main,
        WireHand::OffHand => Hand::Off,
    }
}

fn direction(direction: WireDirection) -> Direction {
    match direction {
        WireDirection::Down => Direction::Down,
        WireDirection::Up => Direction::Up,
        WireDirection::North => Direction::North,
        WireDirection::South => Direction::South,
        WireDirection::West => Direction::West,
        WireDirection::East => Direction::East,
    }
}

fn game_mode(mode: WireGameMode) -> GameMode {
    match mode {
        WireGameMode::Survival => GameMode::Survival,
        WireGameMode::Creative => GameMode::Creative,
        WireGameMode::Adventure => GameMode::Adventure,
        WireGameMode::Spectator => GameMode::Spectator,
    }
}

pub fn item_stack(stack: &WireStack) -> Option<ItemStack> {
    if stack.is_empty() {
        return None;
    }
    let item = u16::try_from(stack.item).ok()?;
    let count = NonZeroU8::new(u8::try_from(stack.count.clamp(1, 99)).ok()?)?;
    Some(ItemStack {
        item: ItemId(item),
        count,
        components: Compound::new(),
    })
}

fn finite(value: f64) -> bool {
    value.is_finite()
}

fn input(value: PlayerInput) -> Decoded {
    Decoded::Input(value)
}

pub fn decode(packet: ServerboundPlay) -> Decoded {
    match packet {
        ServerboundPlay::AcceptTeleportation(p) => {
            input(PlayerInput::TeleportConfirm(p.teleport_id))
        }
        ServerboundPlay::MovePlayerPos(p) if finite(p.x) && finite(p.y) && finite(p.z) => {
            input(PlayerInput::Move {
                position: Some(Vec3::new(p.x, p.y, p.z)),
                rotation: None,
                on_ground: p.flags.on_ground,
            })
        }
        ServerboundPlay::MovePlayerPosRot(p) if finite(p.x) && finite(p.y) && finite(p.z) => {
            input(PlayerInput::Move {
                position: Some(Vec3::new(p.x, p.y, p.z)),
                rotation: Some(Rotation::new(p.y_rot, p.x_rot)),
                on_ground: p.flags.on_ground,
            })
        }
        ServerboundPlay::MovePlayerRot(p) => input(PlayerInput::Move {
            position: None,
            rotation: Some(Rotation::new(p.y_rot, p.x_rot)),
            on_ground: p.flags.on_ground,
        }),
        ServerboundPlay::MovePlayerStatusOnly(p) => input(PlayerInput::Move {
            position: None,
            rotation: None,
            on_ground: p.flags.on_ground,
        }),
        ServerboundPlay::Chat(p) => input(PlayerInput::Chat(p.message)),
        ServerboundPlay::ChatCommand(p) => input(PlayerInput::Command(p.command)),
        ServerboundPlay::ChatCommandSigned(p) => input(PlayerInput::Command(p.command)),
        ServerboundPlay::ChunkBatchReceived(p) => input(PlayerInput::ChunkBatchReceived {
            chunks_per_tick: p.desired_chunks_per_tick,
        }),
        ServerboundPlay::ClientCommand(p) => match p.action {
            ClientCommandAction::PerformRespawn => input(PlayerInput::Respawn),
            _ => Decoded::Ignored,
        },
        ServerboundPlay::ClientInformation(info) => {
            input(PlayerInput::Settings(client_settings(&info)))
        }
        ServerboundPlay::CommandSuggestion(p) => input(PlayerInput::CommandSuggestion {
            id: p.id,
            text: p.command,
        }),
        ServerboundPlay::PlayerAction(p) => {
            let action = match p.action {
                PlayerActionKind::StartDestroyBlock => DigAction::Start,
                PlayerActionKind::AbortDestroyBlock => DigAction::Cancel,
                PlayerActionKind::StopDestroyBlock => DigAction::Finish,
                PlayerActionKind::DropAllItems => DigAction::DropStack,
                PlayerActionKind::DropItem => DigAction::DropItem,
                PlayerActionKind::ReleaseUseItem => DigAction::ReleaseUseItem,
                PlayerActionKind::SwapItemWithOffhand => DigAction::SwapHands,
                PlayerActionKind::Stab => DigAction::Stab,
                PlayerActionKind::ChangeDestroyDirection => return Decoded::Ignored,
            };
            input(PlayerInput::Dig {
                action,
                pos: block_pos(p.pos),
                face: direction(p.direction),
                sequence: p.sequence,
            })
        }
        ServerboundPlay::UseItemOn(p) => input(PlayerInput::UseItemOn {
            hand: hand(p.hand),
            pos: block_pos(p.hit.pos),
            face: direction(p.hit.direction),
            cursor: Vec3::new(
                f64::from(p.hit.cursor_x),
                f64::from(p.hit.cursor_y),
                f64::from(p.hit.cursor_z),
            ),
            inside_block: p.hit.inside,
            sequence: p.sequence,
        }),
        ServerboundPlay::UseItem(p) => input(PlayerInput::UseItem {
            hand: hand(p.hand),
            sequence: p.sequence,
            rotation: Rotation::new(p.y_rot, p.x_rot),
        }),
        ServerboundPlay::SetCarriedItem(p) => match u8::try_from(p.slot) {
            Ok(slot) if slot < 9 => input(PlayerInput::HeldSlot(slot)),
            _ => Decoded::Ignored,
        },
        ServerboundPlay::SetCreativeModeSlot(p) => input(PlayerInput::CreativeSlot {
            slot: p.slot,
            stack: item_stack(&p.item),
        }),
        ServerboundPlay::Punch(_) => input(PlayerInput::Swing(Hand::Main)),
        ServerboundPlay::PlayerCommand(p) => match p.action {
            PlayerCommandAction::StartSprinting => {
                input(PlayerInput::Movement(MovementAction::StartSprinting))
            }
            PlayerCommandAction::StopSprinting => {
                input(PlayerInput::Movement(MovementAction::StopSprinting))
            }
            PlayerCommandAction::StopSleeping => {
                input(PlayerInput::Movement(MovementAction::LeaveBed))
            }
            PlayerCommandAction::StartFallFlying => {
                input(PlayerInput::Movement(MovementAction::StartFallFlying))
            }
            _ => Decoded::Ignored,
        },
        ServerboundPlay::PlayerInput(p) => input(PlayerInput::Input(InputState {
            forward: p.forward,
            backward: p.backward,
            left: p.left,
            right: p.right,
            jump: p.jump,
            shift: p.shift,
            sprint: p.sprint,
        })),
        ServerboundPlay::PlayerLoaded(_) => input(PlayerInput::Loaded),
        ServerboundPlay::PlayerAbilities(p) => input(PlayerInput::Abilities { flying: p.flying }),
        ServerboundPlay::ChangeGameMode(p) => input(PlayerInput::ChangeGameMode(game_mode(p.mode))),
        ServerboundPlay::PickItemFromBlock(p) => input(PlayerInput::PickBlock {
            pos: block_pos(p.pos),
            include_data: p.include_data,
        }),
        ServerboundPlay::Attack(p) => input(PlayerInput::Attack(EntityId(p.entity_id))),
        ServerboundPlay::Interact(p) => input(PlayerInput::Interact {
            target: EntityId(p.entity_id),
            hand: hand(p.hand),
            sneaking: p.using_secondary_action,
        }),
        ServerboundPlay::KeepAlive(p) => Decoded::KeepAlive(p.id),
        ServerboundPlay::PingRequest(p) => Decoded::Ping(p.time),
        _ => Decoded::Ignored,
    }
}
