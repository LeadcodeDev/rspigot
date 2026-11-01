use rspigot_plugin_protocol::v1::{self, answer, from_plugin, question, world_order};
use uuid::Uuid;

use crate::block::{Block, BlockPos};
use crate::entity::{Entities, Entity};
use crate::link::Link;
use crate::wire::parse_uuid;
use crate::{EntityType, Error, ItemStack, Particle, Position, Sound, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

impl Weather {
    const fn to_wire(self) -> v1::Weather {
        match self {
            Self::Clear => v1::Weather::Clear,
            Self::Rain => v1::Weather::Rain,
            Self::Thunder => v1::Weather::Thunder,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    fn from_wire(difficulty: i32) -> Result<Self, Error> {
        match v1::Difficulty::try_from(difficulty) {
            Ok(v1::Difficulty::Peaceful) => Ok(Self::Peaceful),
            Ok(v1::Difficulty::Easy) => Ok(Self::Easy),
            Ok(v1::Difficulty::Normal) => Ok(Self::Normal),
            Ok(v1::Difficulty::Hard) => Ok(Self::Hard),
            Ok(v1::Difficulty::Unspecified) | Err(_) => Err(Error::Malformed(format!(
                "an unknown difficulty {difficulty}"
            ))),
        }
    }

    const fn to_wire(self) -> v1::Difficulty {
        match self {
            Self::Peaceful => v1::Difficulty::Peaceful,
            Self::Easy => v1::Difficulty::Easy,
            Self::Normal => v1::Difficulty::Normal,
            Self::Hard => v1::Difficulty::Hard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameRule {
    DaylightCycle,
    NaturalRegeneration,
    KeepInventory,
    ShowDeathMessages,
    ImmediateRespawn,
    FallDamage,
    MobDrops,
    MobGriefing,
    MobExplosionDropDecay,
    BlockDrops,
    UniversalAnger,
}

impl GameRule {
    fn from_wire(rule: i32) -> Result<Self, Error> {
        match v1::GameRule::try_from(rule) {
            Ok(v1::GameRule::DaylightCycle) => Ok(Self::DaylightCycle),
            Ok(v1::GameRule::NaturalRegeneration) => Ok(Self::NaturalRegeneration),
            Ok(v1::GameRule::KeepInventory) => Ok(Self::KeepInventory),
            Ok(v1::GameRule::ShowDeathMessages) => Ok(Self::ShowDeathMessages),
            Ok(v1::GameRule::ImmediateRespawn) => Ok(Self::ImmediateRespawn),
            Ok(v1::GameRule::FallDamage) => Ok(Self::FallDamage),
            Ok(v1::GameRule::MobDrops) => Ok(Self::MobDrops),
            Ok(v1::GameRule::MobGriefing) => Ok(Self::MobGriefing),
            Ok(v1::GameRule::MobExplosionDropDecay) => Ok(Self::MobExplosionDropDecay),
            Ok(v1::GameRule::BlockDrops) => Ok(Self::BlockDrops),
            Ok(v1::GameRule::UniversalAnger) => Ok(Self::UniversalAnger),
            Ok(v1::GameRule::Unspecified) | Err(_) => {
                Err(Error::Malformed(format!("an unknown game rule {rule}")))
            }
        }
    }

    const fn to_wire(self) -> v1::GameRule {
        match self {
            Self::DaylightCycle => v1::GameRule::DaylightCycle,
            Self::NaturalRegeneration => v1::GameRule::NaturalRegeneration,
            Self::KeepInventory => v1::GameRule::KeepInventory,
            Self::ShowDeathMessages => v1::GameRule::ShowDeathMessages,
            Self::ImmediateRespawn => v1::GameRule::ImmediateRespawn,
            Self::FallDamage => v1::GameRule::FallDamage,
            Self::MobDrops => v1::GameRule::MobDrops,
            Self::MobGriefing => v1::GameRule::MobGriefing,
            Self::MobExplosionDropDecay => v1::GameRule::MobExplosionDropDecay,
            Self::BlockDrops => v1::GameRule::BlockDrops,
            Self::UniversalAnger => v1::GameRule::UniversalAnger,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldState {
    pub name: String,
    pub uuid: Option<Uuid>,
    pub seed: i64,
    pub day_time: i64,
    pub game_time: i64,
    pub raining: bool,
    pub thundering: bool,
    pub difficulty: Difficulty,
    pub spawn: BlockPos,
    pub pvp: bool,
    pub rules: Vec<(GameRule, bool)>,
}

impl WorldState {
    fn from_wire(state: v1::WorldState) -> Result<Self, Error> {
        Ok(Self {
            uuid: state
                .uuid
                .as_deref()
                .map(|text| parse_uuid(text, "a world"))
                .transpose()?,
            seed: state.seed,
            day_time: state.day_time,
            game_time: state.game_time,
            raining: state.raining,
            thundering: state.thundering,
            difficulty: Difficulty::from_wire(state.difficulty)?,
            spawn: BlockPos::from_wire(state.spawn, "spawn")?,
            pvp: state.pvp,
            rules: state
                .rules
                .into_iter()
                .map(|rule| Ok((GameRule::from_wire(rule.rule)?, rule.value)))
                .collect::<Result<_, Error>>()?,
            name: state.name,
        })
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub entities: Entities,
    link: Link,
}

impl World {
    pub(crate) fn new(link: Link) -> Self {
        Self {
            entities: Entities::new(link.clone()),
            link,
        }
    }

    pub fn set_time(&self, time_of_day: i64) {
        self.order(world_order::Order::SetTime(time_of_day));
    }

    pub fn set_weather(&self, weather: Weather) {
        self.order(world_order::Order::SetWeather(weather.to_wire() as i32));
    }

    pub fn set_difficulty(&self, difficulty: Difficulty) {
        self.order(world_order::Order::SetDifficulty(
            difficulty.to_wire() as i32
        ));
    }

    pub fn set_spawn(&self, pos: BlockPos) {
        self.order(world_order::Order::SetSpawn(pos.to_wire()));
    }

    pub fn set_pvp(&self, pvp: bool) {
        self.order(world_order::Order::SetPvp(pvp));
    }

    pub fn set_game_rule(&self, rule: GameRule, value: bool) {
        self.order(world_order::Order::SetGameRule(v1::SetGameRule {
            rule: rule.to_wire() as i32,
            value,
        }));
    }

    pub fn save(&self) {
        self.order(world_order::Order::Save(v1::Empty {}));
    }

    pub fn play_sound(&self, sound: Sound, position: Position, volume: f32, pitch: f32) {
        self.order(world_order::Order::PlaySound(v1::Sound {
            key: sound.key().to_owned(),
            volume,
            pitch,
            position: Some(position.to_wire()),
        }));
    }

    pub fn spawn_particle(
        &self,
        particle: Particle,
        position: Position,
        count: i32,
        spread: Vector,
        speed: f64,
    ) {
        self.order(world_order::Order::SpawnParticle(
            particle.to_wire(position, count, spread, speed),
        ));
    }

    pub fn explode(&self, position: Position, power: f32, breaks_blocks: bool) {
        self.order(world_order::Order::Explode(v1::Explode {
            position: Some(position.to_wire()),
            power,
            breaks_blocks,
        }));
    }

    pub fn block(&self, pos: BlockPos) -> Block {
        Block::new(pos, self.link.clone())
    }

    pub async fn spawn(&self, kind: EntityType, position: Position) -> Result<Entity, Error> {
        self.created(question::Question::Spawn(v1::Spawn {
            kind: kind.key().to_owned(),
            position: Some(position.to_wire()),
        }))
        .await
    }

    pub async fn drop_item(&self, position: Position, item: ItemStack) -> Result<Entity, Error> {
        self.created(question::Question::DropItem(v1::DropItem {
            position: Some(position.to_wire()),
            item: Some(item.into_wire()),
        }))
        .await
    }

    pub async fn resolve(&self) -> Result<WorldState, Error> {
        let state = self
            .link
            .ask(
                question::Question::World(v1::Empty {}),
                "a world state",
                |body| match body {
                    answer::Body::World(state) => Some(state),
                    _ => None,
                },
            )
            .await?;
        WorldState::from_wire(state)
    }

    async fn created(&self, question: question::Question) -> Result<Entity, Error> {
        let snapshot = self
            .link
            .ask(question, "a created entity", |body| match body {
                answer::Body::Spawned(snapshot) => Some(snapshot),
                _ => None,
            })
            .await?;
        Entity::from_wire(snapshot, &self.link)
    }

    fn order(&self, order: world_order::Order) {
        self.link.order(from_plugin::Body::World(v1::WorldOrder {
            order: Some(order),
        }));
    }
}
