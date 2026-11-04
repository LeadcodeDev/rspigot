use crossbeam_channel::Sender;
use rspigot_domain::entity::EntityBase;
use rspigot_domain::item::ItemStack;
use rspigot_domain::player::{ClientSettings, GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::{BlockPos, BlockStateId, EntityId, Rotation, Vec3};

use crate::commands::CommandSender;
use crate::events::{Direction, Hand, InputState, PreLoginVerdict};
use crate::plugins::EventKind;
use crate::plugins::events::PlayerTeleportCause;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Held {
    pub(crate) position: Option<Vec3>,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MoveStep {
    pub(crate) player: PlayerId,
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) on_ground: bool,
}

#[derive(Debug)]
pub(crate) struct LeaveStep {
    pub(crate) name: String,
    pub(crate) default: String,
    pub(crate) vanilla: bool,
}

#[derive(Debug)]
pub(crate) struct KickStep {
    pub(crate) player: PlayerId,
    pub(crate) reason: Component,
    pub(crate) text: String,
    pub(crate) leave: String,
}

#[derive(Debug)]
pub(crate) struct SettingsStep {
    pub(crate) kind: EventKind,
    pub(crate) player: PlayerId,
    pub(crate) settings: Option<ClientSettings>,
}

#[derive(Debug)]
pub(crate) struct ChatStep {
    pub(crate) player: PlayerId,
    pub(crate) message: String,
}

#[derive(Debug)]
pub(crate) struct TeleportStep {
    pub(crate) player: PlayerId,
    pub(crate) position: Vec3,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) cause: PlayerTeleportCause,
    pub(crate) fallback: Option<MoveStep>,
}

#[derive(Debug)]
pub(crate) struct VelocityStep {
    pub(crate) player: PlayerId,
    pub(crate) restore: Option<Vec3>,
}

#[derive(Debug)]
pub(crate) struct RespawnStep {
    pub(crate) player: PlayerId,
}

#[derive(Debug)]
pub(crate) struct DeathStep {
    pub(crate) player: PlayerId,
    pub(crate) entity: EntityId,
    pub(crate) message: Component,
    pub(crate) text: String,
    pub(crate) show: bool,
    pub(crate) keep: bool,
    pub(crate) taken: Vec<(usize, ItemStack)>,
    pub(crate) origin: Vec3,
    pub(crate) feet: Vec3,
    pub(crate) experience: i32,
}

#[derive(Debug)]
pub(crate) struct InputStep {
    pub(crate) player: PlayerId,
    pub(crate) state: InputState,
}

#[derive(Debug)]
pub(crate) struct ToggleStep {
    pub(crate) player: PlayerId,
    pub(crate) value: bool,
}

#[derive(Debug)]
pub(crate) struct GameModeStep {
    pub(crate) player: PlayerId,
    pub(crate) mode: GameMode,
    pub(crate) replies: Vec<(CommandSender, Component)>,
}

#[derive(Debug)]
pub(crate) struct HeldStep {
    pub(crate) player: PlayerId,
    pub(crate) slot: u8,
    pub(crate) previous: Option<u8>,
}

#[derive(Debug)]
pub(crate) struct SwingStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
}

#[derive(Debug)]
pub(crate) struct SwapStep {
    pub(crate) player: PlayerId,
    pub(crate) selected: u8,
    pub(crate) main: Option<ItemStack>,
    pub(crate) off: Option<ItemStack>,
}

#[derive(Debug)]
pub(crate) struct UseOnStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) slot: usize,
    pub(crate) pos: BlockPos,
    pub(crate) face: Direction,
    pub(crate) cursor: Vec3,
    pub(crate) sequence: i32,
    pub(crate) stack: Option<ItemStack>,
    pub(crate) clicked: BlockStateId,
    pub(crate) game_mode: GameMode,
    pub(crate) block_denied: bool,
}

#[derive(Debug)]
pub(crate) struct UseStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) sequence: i32,
}

#[derive(Debug)]
pub(crate) struct DigStep {
    pub(crate) player: PlayerId,
    pub(crate) pos: BlockPos,
    pub(crate) sequence: i32,
    pub(crate) clicked: BlockStateId,
    pub(crate) game_mode: GameMode,
}

#[derive(Debug)]
pub(crate) struct InteractStep {
    pub(crate) player: PlayerId,
    pub(crate) target: EntityId,
    pub(crate) hand: Hand,
    pub(crate) stack: Option<ItemStack>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DropSource {
    Selected { slot: usize },
    Creative,
    Released,
}

#[derive(Debug)]
pub(crate) struct Launch {
    pub(crate) base: EntityBase,
    pub(crate) velocity: Vec3,
    pub(crate) yaw: f32,
}

#[derive(Debug)]
pub(crate) struct DropStep {
    pub(crate) player: PlayerId,
    pub(crate) stack: ItemStack,
    pub(crate) source: DropSource,
    pub(crate) launch: Launch,
    pub(crate) thrower: bool,
    pub(crate) predicted: bool,
}

#[derive(Debug)]
pub(crate) struct WearStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) slot: usize,
    pub(crate) stack: ItemStack,
    pub(crate) damage: i32,
}

#[derive(Debug)]
pub(crate) struct BookStep {
    pub(crate) player: PlayerId,
    pub(crate) pos: BlockPos,
    pub(crate) window: i32,
    pub(crate) book: ItemStack,
}

#[derive(Debug)]
pub(crate) struct CooldownStep {
    pub(crate) player: PlayerId,
    pub(crate) value: i32,
}

#[derive(Debug)]
pub(crate) struct OrbStep {
    pub(crate) player: PlayerId,
    pub(crate) collector: EntityId,
    pub(crate) orb: EntityId,
    pub(crate) amount: i32,
}

#[derive(Debug)]
pub(crate) enum Continuation {
    PlayerMove(MoveStep),
    Notice(EventKind),
    PreLogin(Sender<PreLoginVerdict>),
    Login(u64),
    Join(PlayerId),
    Leave(LeaveStep),
    Kick(KickStep),
    Settings(SettingsStep),
    CommandSend(PlayerId),
    Chat(ChatStep),
    Command(ChatStep),
    Teleport(TeleportStep),
    Velocity(VelocityStep),
    Respawn(RespawnStep),
    Death(Box<DeathStep>),
    Input(InputStep),
    Sneak(ToggleStep),
    Sprint(ToggleStep),
    Flight(ToggleStep),
    GameMode(GameModeStep),
    HeldSlot(HeldStep),
    Swing(SwingStep),
    Swap(Box<SwapStep>),
    UseItemOn(Box<UseOnStep>),
    UseItem(UseStep),
    Dig(DigStep),
    InteractEntity(Box<InteractStep>),
    Drop(Box<DropStep>),
    ItemDamage(Box<WearStep>),
    TakeBook(Box<BookStep>),
    ExpCooldown(CooldownStep),
    ExpChange(OrbStep),
}

impl Continuation {
    pub(crate) const fn kind(&self) -> EventKind {
        match self {
            Self::PlayerMove(_) => EventKind::PlayerMove,
            Self::Notice(kind) => *kind,
            Self::PreLogin(_) => EventKind::AsyncPlayerPreLogin,
            Self::Login(_) => EventKind::PlayerLogin,
            Self::Join(_) => EventKind::PlayerJoin,
            Self::Leave(_) => EventKind::PlayerLeave,
            Self::Kick(_) => EventKind::PlayerKick,
            Self::Settings(step) => step.kind,
            Self::CommandSend(_) => EventKind::PlayerCommandSend,
            Self::Chat(_) => EventKind::AsyncPlayerChat,
            Self::Command(_) => EventKind::PlayerCommandPreprocess,
            Self::Teleport(_) => EventKind::PlayerTeleport,
            Self::Velocity(_) => EventKind::PlayerVelocity,
            Self::Respawn(_) => EventKind::PlayerRespawn,
            Self::Death(_) => EventKind::PlayerDeath,
            Self::Input(_) => EventKind::PlayerInput,
            Self::Sneak(_) => EventKind::PlayerToggleSneak,
            Self::Sprint(_) => EventKind::PlayerToggleSprint,
            Self::Flight(_) => EventKind::PlayerToggleFlight,
            Self::GameMode(_) => EventKind::PlayerGameModeChange,
            Self::HeldSlot(_) => EventKind::PlayerItemHeld,
            Self::Swing(_) => EventKind::PlayerAnimation,
            Self::Swap(_) => EventKind::PlayerSwapHandItems,
            Self::UseItemOn(_) | Self::UseItem(_) | Self::Dig(_) => EventKind::PlayerInteract,
            Self::InteractEntity(_) => EventKind::PlayerInteractAtEntity,
            Self::Drop(_) => EventKind::PlayerDropItem,
            Self::ItemDamage(_) => EventKind::PlayerItemDamage,
            Self::TakeBook(_) => EventKind::PlayerTakeLecternBook,
            Self::ExpCooldown(_) => EventKind::PlayerExpCooldownChange,
            Self::ExpChange(_) => EventKind::PlayerExpChange,
        }
    }

    pub(crate) const fn destination(&self) -> Option<Vec3> {
        match self {
            Self::PlayerMove(step) => Some(step.to),
            _ => None,
        }
    }

    pub(crate) const fn undo(&self) -> Option<Undo> {
        match self {
            Self::PlayerMove(step) => Some(Undo::SendBack(step.from)),
            _ => None,
        }
    }

    pub(crate) const fn is_move(&self) -> bool {
        matches!(self, Self::PlayerMove(_))
    }

    pub(crate) fn same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Velocity(one), Self::Velocity(other)) => one.player == other.player,
            (Self::ItemDamage(one), Self::ItemDamage(other)) => {
                one.player == other.player && one.slot == other.slot
            }
            _ => false,
        }
    }

    pub(crate) const fn hot(&self) -> bool {
        matches!(self, Self::Velocity(_) | Self::ItemDamage(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Undo {
    SendBack(Vec3),
}
