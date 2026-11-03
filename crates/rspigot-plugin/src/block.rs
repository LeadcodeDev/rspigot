use rspigot_plugin_protocol::v1::{self, answer, block_order, from_plugin, question};

use crate::link::Link;
use crate::{BlockType, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Face {
    pub(crate) fn from_wire(face: i32) -> Result<Self, Error> {
        match v1::Face::try_from(face) {
            Ok(v1::Face::Down) => Ok(Self::Down),
            Ok(v1::Face::Up) => Ok(Self::Up),
            Ok(v1::Face::North) => Ok(Self::North),
            Ok(v1::Face::South) => Ok(Self::South),
            Ok(v1::Face::West) => Ok(Self::West),
            Ok(v1::Face::East) => Ok(Self::East),
            Ok(v1::Face::Unspecified) | Err(_) => {
                Err(Error::Malformed(format!("an unknown face {face}")))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockType,
    pub properties: Vec<(String, String)>,
}

impl BlockState {
    fn from_wire(state: v1::BlockState) -> Result<Self, Error> {
        Ok(Self {
            kind: block_type(&state.block)?,
            properties: state
                .properties
                .into_iter()
                .map(|property| (property.name, property.value))
                .collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub pos: BlockPos,
    link: Link,
}

impl Block {
    pub(crate) const fn new(pos: BlockPos, link: Link) -> Self {
        Self { pos, link }
    }

    pub fn set(&self, kind: BlockType) {
        self.order(block_order::Order::Set(kind.key().to_owned()));
    }

    pub fn set_with(&self, kind: BlockType, properties: &[(&str, &str)]) {
        self.order(block_order::Order::SetWith(v1::SetBlockWith {
            block: kind.key().to_owned(),
            properties: properties
                .iter()
                .map(|(name, value)| v1::Property {
                    name: (*name).to_owned(),
                    value: (*value).to_owned(),
                })
                .collect(),
        }));
    }

    pub fn break_naturally(&self) {
        self.order(block_order::Order::BreakNaturally(v1::Empty {}));
    }

    pub async fn resolve(&self) -> Result<BlockState, Error> {
        let state = self
            .link
            .ask(
                question::Question::Block(self.pos.to_wire()),
                "a block state",
                |body| match body {
                    answer::Body::Block(state) => Some(state),
                    _ => None,
                },
            )
            .await?;
        BlockState::from_wire(state)
    }

    fn order(&self, order: block_order::Order) {
        self.link.order(from_plugin::Body::Block(v1::BlockOrder {
            pos: Some(self.pos.to_wire()),
            order: Some(order),
        }));
    }
}

pub(crate) fn block_type(key: &str) -> Result<BlockType, Error> {
    BlockType::from_key(key).ok_or_else(|| Error::Malformed(format!("an unknown block `{key}`")))
}
