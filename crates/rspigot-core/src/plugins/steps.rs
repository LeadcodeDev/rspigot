use rspigot_domain::player::PlayerId;
use rspigot_domain::{Rotation, Vec3};

use crate::plugins::EventKind;

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
pub(crate) enum Continuation {
    PlayerMove(MoveStep),
    Notice(EventKind),
}

impl Continuation {
    pub(crate) const fn kind(&self) -> EventKind {
        match self {
            Self::PlayerMove(_) => EventKind::PlayerMove,
            Self::Notice(kind) => *kind,
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

    pub(crate) fn same_as(&self, _other: &Self) -> bool {
        false
    }

    pub(crate) const fn hot(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Undo {
    SendBack(Vec3),
}
