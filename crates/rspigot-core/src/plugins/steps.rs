use crossbeam_channel::Sender;
use rspigot_domain::player::{ClientSettings, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::{Rotation, Vec3};

use crate::events::PreLoginVerdict;
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
