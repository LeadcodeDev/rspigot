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
    pub(crate) message: Component,
    pub(crate) text: String,
    pub(crate) show: bool,
    pub(crate) keep: bool,
    pub(crate) inventory: Vec<(usize, ItemStack)>,
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
pub(crate) struct ClickStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) pos: BlockPos,
    pub(crate) face: Direction,
    pub(crate) cursor: Vec3,
    pub(crate) sequence: i32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Aim {
    pub(crate) pos: BlockPos,
    pub(crate) face: Direction,
    pub(crate) clicked: Vec3,
}

#[derive(Debug)]
pub(crate) struct UseStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) sequence: i32,
    pub(crate) rotation: Rotation,
    pub(crate) aim: Option<Aim>,
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
    pub(crate) position: Vec3,
    pub(crate) velocity: Vec3,
    pub(crate) yaw: f32,
}

#[derive(Debug)]
pub(crate) struct Announced {
    pub(crate) base: EntityBase,
    pub(crate) velocity: Vec3,
    pub(crate) yaw: f32,
}

#[derive(Debug)]
pub(crate) struct SelectedDrop {
    pub(crate) player: PlayerId,
    pub(crate) all: bool,
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
pub(crate) struct WearRequest {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) slot: usize,
    pub(crate) amount: i32,
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
pub(crate) struct ArrowStep {
    pub(crate) player: PlayerId,
    pub(crate) collector: EntityId,
    pub(crate) arrow: EntityId,
}

#[derive(Debug)]
pub(crate) struct ShearStep {
    pub(crate) player: PlayerId,
    pub(crate) target: EntityId,
    pub(crate) hand: Hand,
    pub(crate) stack: ItemStack,
    pub(crate) parked: bool,
}

#[derive(Debug)]
pub(crate) struct DamageStep {
    pub(crate) player: PlayerId,
    pub(crate) pos: BlockPos,
    pub(crate) sequence: i32,
    pub(crate) state: BlockStateId,
    pub(crate) held: Option<ItemStack>,
    pub(crate) insta: bool,
}

#[derive(Debug)]
pub(crate) struct BreakStep {
    pub(crate) player: PlayerId,
    pub(crate) pos: BlockPos,
    pub(crate) sequence: i32,
    pub(crate) state: BlockStateId,
    pub(crate) game_mode: GameMode,
    pub(crate) held: Option<ItemStack>,
    pub(crate) refused: bool,
    pub(crate) exp: i32,
}

#[derive(Debug)]
pub(crate) struct FinishStep {
    pub(crate) player: PlayerId,
    pub(crate) pos: BlockPos,
    pub(crate) sequence: i32,
}

#[derive(Debug)]
pub(crate) struct AbortStep {
    pub(crate) player: PlayerId,
    pub(crate) pos: BlockPos,
    pub(crate) sequence: i32,
}

#[derive(Debug)]
pub(crate) struct DropItemsStep {
    pub(crate) pos: BlockPos,
    pub(crate) drops: Vec<(Announced, ItemStack)>,
    pub(crate) exp: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Change {
    pub(crate) pos: BlockPos,
    pub(crate) old: BlockStateId,
    pub(crate) new: BlockStateId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolEffects {
    pub(crate) sound: &'static str,
    pub(crate) particle: Option<i32>,
    pub(crate) damage: i32,
}

#[derive(Debug)]
pub(crate) struct UseItem {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) slot: usize,
    pub(crate) stack: ItemStack,
    pub(crate) against: BlockPos,
    pub(crate) sequence: i32,
    pub(crate) game_mode: GameMode,
}

#[derive(Debug)]
pub(crate) struct BuildStep {
    pub(crate) using: UseItem,
    pub(crate) target: BlockPos,
    pub(crate) old: BlockStateId,
    pub(crate) state: BlockStateId,
    pub(crate) linked: Option<(BlockPos, BlockStateId)>,
    pub(crate) buildable: bool,
}

#[derive(Debug)]
pub(crate) struct PlaceStep {
    pub(crate) using: UseItem,
    pub(crate) changes: Vec<Change>,
    pub(crate) build: bool,
    pub(crate) tool: Option<ToolEffects>,
    pub(crate) forced: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PhysicsStep {
    pub(crate) pos: BlockPos,
    pub(crate) state: BlockStateId,
}

#[derive(Debug)]
pub(crate) struct CookStep {
    pub(crate) pos: BlockPos,
    pub(crate) slot: usize,
    pub(crate) source: ItemStack,
    pub(crate) result: ItemStack,
}

#[derive(Debug)]
pub(crate) struct CampfireStep {
    pub(crate) player: PlayerId,
    pub(crate) hand: Hand,
    pub(crate) slot: usize,
    pub(crate) pos: BlockPos,
    pub(crate) sequence: i32,
    pub(crate) held: ItemStack,
    pub(crate) game_mode: GameMode,
    pub(crate) cooking_slot: usize,
    pub(crate) total: i32,
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
    ClickBlock(Box<ClickStep>),
    UseItemOn(Box<UseOnStep>),
    UseItem(UseStep),
    Dig(DigStep),
    InteractEntity(Box<InteractStep>),
    DropSelected(SelectedDrop),
    Drop(Box<DropStep>),
    Wear(WearRequest),
    ItemDamage(Box<WearStep>),
    TakeBook(Box<BookStep>),
    ExpCooldown(CooldownStep),
    ExpChange(OrbStep),
    PickupArrow(ArrowStep),
    Shear(Box<ShearStep>),
    Damage(Box<DamageStep>),
    Break(Box<BreakStep>),
    Finish(FinishStep),
    Abort(AbortStep),
    DropItems(Box<DropItemsStep>),
    CanBuild(Box<BuildStep>),
    Place(Box<PlaceStep>),
    MultiPlace(Box<PlaceStep>),
    Physics(PhysicsStep),
    Cook(Box<CookStep>),
    CampfireStart(Box<CampfireStep>),
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
            Self::ClickBlock(_) | Self::UseItemOn(_) | Self::UseItem(_) | Self::Dig(_) => {
                EventKind::PlayerInteract
            }
            Self::InteractEntity(_) => EventKind::PlayerInteractAtEntity,
            Self::DropSelected(_) | Self::Drop(_) => EventKind::PlayerDropItem,
            Self::Wear(_) | Self::ItemDamage(_) => EventKind::PlayerItemDamage,
            Self::TakeBook(_) => EventKind::PlayerTakeLecternBook,
            Self::ExpCooldown(_) => EventKind::PlayerExpCooldownChange,
            Self::ExpChange(_) => EventKind::PlayerExpChange,
            Self::PickupArrow(_) => EventKind::PlayerPickupArrow,
            Self::Shear(_) => EventKind::PlayerShearEntity,
            Self::Damage(_) => EventKind::BlockDamage,
            Self::Break(_) | Self::Finish(_) => EventKind::BlockBreak,
            Self::Abort(_) => EventKind::BlockDamageAbort,
            Self::DropItems(_) => EventKind::BlockDropItem,
            Self::CanBuild(_) => EventKind::BlockCanBuild,
            Self::Place(_) => EventKind::BlockPlace,
            Self::MultiPlace(_) => EventKind::BlockMultiPlace,
            Self::Physics(_) => EventKind::BlockPhysics,
            Self::Cook(_) => EventKind::BlockCook,
            Self::CampfireStart(_) => EventKind::CampfireStart,
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
            (Self::PickupArrow(one), Self::PickupArrow(other)) => {
                one.player == other.player && one.arrow == other.arrow
            }
            (Self::Cook(one), Self::Cook(other)) => one.pos == other.pos,
            _ => false,
        }
    }

    pub(crate) const fn may_skip(&self) -> bool {
        matches!(
            self,
            Self::Velocity(_) | Self::PickupArrow(_) | Self::Cook(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Undo {
    SendBack(Vec3),
}
