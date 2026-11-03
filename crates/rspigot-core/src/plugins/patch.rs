use rspigot_domain::Vec3;

use crate::plugins::resolve::position_of;
use crate::plugins::{EventKind, EventPayload};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Verdict {
    pub handler: u32,
    pub cancel: bool,
    pub patch: Option<EventPatch>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventPatch {
    PlayerMove(PlayerMovePatch),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerMovePatch {
    pub to: Option<Vec3>,
}

impl EventPatch {
    pub const fn kind(&self) -> EventKind {
        match self {
            Self::PlayerMove(_) => EventKind::PlayerMove,
        }
    }

    pub(crate) fn then(self, later: Self) -> Self {
        match (self, later) {
            (Self::PlayerMove(earlier), Self::PlayerMove(later)) => {
                Self::PlayerMove(earlier.then(later))
            }
        }
    }

    pub(crate) fn refusal(&self) -> Option<String> {
        match self {
            Self::PlayerMove(patch) => patch.refusal(),
        }
    }

    pub(crate) fn settle(self, payload: &mut EventPayload) {
        if let (Self::PlayerMove(patch), EventPayload::PlayerMove { to, .. }) = (self, payload) {
            patch.settle(to);
        }
    }
}

impl PlayerMovePatch {
    fn then(self, later: Self) -> Self {
        Self {
            to: later.to.or(self.to),
        }
    }

    fn refusal(&self) -> Option<String> {
        self.to.and_then(|to| position_of(to).err())
    }

    fn settle(self, to: &mut Vec3) {
        if let Some(patched) = self.to {
            *to = patched;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moving(to: Option<Vec3>) -> EventPatch {
        EventPatch::PlayerMove(PlayerMovePatch { to })
    }

    #[test]
    fn a_later_patch_wins_each_field_it_sets_and_keeps_the_others() {
        let near = Vec3::new(1.0, 64.0, 0.0);
        let far = Vec3::new(9.0, 64.0, 0.0);
        assert_eq!(
            moving(Some(near)).then(moving(Some(far))),
            moving(Some(far))
        );
        assert_eq!(moving(Some(near)).then(moving(None)), moving(Some(near)));
        assert_eq!(moving(None).then(moving(Some(far))), moving(Some(far)));
    }

    #[test]
    fn a_destination_outside_the_world_is_refused() {
        assert_eq!(moving(Some(Vec3::new(1.0, 64.0, 0.0))).refusal(), None);
        assert_eq!(moving(None).refusal(), None);
        for nowhere in [
            Vec3::new(f64::NAN, 64.0, 0.0),
            Vec3::new(0.0, f64::INFINITY, 0.0),
            Vec3::new(0.0, 64.0, 4.0e7),
        ] {
            assert!(moving(Some(nowhere)).refusal().is_some());
        }
    }
}
