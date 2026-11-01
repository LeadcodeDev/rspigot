use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::{Rotation, Vec3};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleSpec {
    pub title: String,
    pub subtitle: Option<String>,
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundSpec {
    pub key: String,
    pub volume: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleDataSpec {
    None,
    Item(String),
    Block(String),
    Dust { color: u32, scale: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSpec {
    pub key: String,
    pub data: ParticleDataSpec,
    pub position: Vec3,
    pub count: i32,
    pub spread: Vec3,
    pub speed: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSpec {
    pub key: String,
    pub duration: i32,
    pub amplifier: i32,
    pub ambient: bool,
    pub particles: bool,
    pub icon: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Player {
        player: PlayerId,
        order: PlayerOrder,
    },
}

impl Order {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Player { order, .. } => order.name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerOrder {
    SendMessage(String),
    SendActionBar(String),
    SendTitle(TitleSpec),
    ResetTitle,
    SetTabList { header: String, footer: String },
    Teleport(Vec3),
    TeleportFacing(Vec3, Rotation),
    SetGameMode(GameMode),
    Kick(String),
    SetHealth(f32),
    SetAbsorption(f32),
    Damage(f64),
    Kill,
    SetFoodLevel(i32),
    SetSaturation(f32),
    SetExhaustion(f32),
    GiveExp(i32),
    GiveLevels(i32),
    SetLevel(i32),
    SetExp(f32),
    AddEffect(EffectSpec),
    RemoveEffect(String),
    ClearEffects,
    SetFireTicks(i32),
    SetVelocity(Vec3),
    SetAllowFlight(bool),
    SetFlying(bool),
    SetFlySpeed(f32),
    SetWalkSpeed(f32),
    SetOp(bool),
    Respawn,
    PerformCommand(String),
    Chat(String),
    PlaySound(SoundSpec),
    SpawnParticle(ParticleSpec),
    CloseInventory,
    UpdateInventory,
}

impl PlayerOrder {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::SendMessage(_) => "player.send_message",
            Self::SendActionBar(_) => "player.send_action_bar",
            Self::SendTitle(_) => "player.send_title",
            Self::ResetTitle => "player.reset_title",
            Self::SetTabList { .. } => "player.set_tab_list",
            Self::Teleport(_) => "player.teleport",
            Self::TeleportFacing(..) => "player.teleport_facing",
            Self::SetGameMode(_) => "player.set_game_mode",
            Self::Kick(_) => "player.kick",
            Self::SetHealth(_) => "player.set_health",
            Self::SetAbsorption(_) => "player.set_absorption",
            Self::Damage(_) => "player.damage",
            Self::Kill => "player.kill",
            Self::SetFoodLevel(_) => "player.set_food_level",
            Self::SetSaturation(_) => "player.set_saturation",
            Self::SetExhaustion(_) => "player.set_exhaustion",
            Self::GiveExp(_) => "player.give_exp",
            Self::GiveLevels(_) => "player.give_levels",
            Self::SetLevel(_) => "player.set_level",
            Self::SetExp(_) => "player.set_exp",
            Self::AddEffect(_) => "player.add_effect",
            Self::RemoveEffect(_) => "player.remove_effect",
            Self::ClearEffects => "player.clear_effects",
            Self::SetFireTicks(_) => "player.set_fire_ticks",
            Self::SetVelocity(_) => "player.set_velocity",
            Self::SetAllowFlight(_) => "player.set_allow_flight",
            Self::SetFlying(_) => "player.set_flying",
            Self::SetFlySpeed(_) => "player.set_fly_speed",
            Self::SetWalkSpeed(_) => "player.set_walk_speed",
            Self::SetOp(_) => "player.set_op",
            Self::Respawn => "player.respawn",
            Self::PerformCommand(_) => "player.perform_command",
            Self::Chat(_) => "player.chat",
            Self::PlaySound(_) => "player.play_sound",
            Self::SpawnParticle(_) => "player.spawn_particle",
            Self::CloseInventory => "player.close_inventory",
            Self::UpdateInventory => "player.update_inventory",
        }
    }
}
