use crate::{MobClass, SpawnPredicate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpawnPlacementType {
    NoRestrictions,
    InWater,
    InLava,
    OnGround,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpawnHeightmap {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpawnPlacement {
    pub kind: SpawnPlacementType,
    pub heightmap: SpawnHeightmap,
    pub predicate: SpawnPredicate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobSpawn {
    pub placement: Option<SpawnPlacement>,
    pub check_spawn_rules: MobClass,
    pub walk_target_value: Option<MobClass>,
    pub check_spawn_obstruction: MobClass,
    pub max_spawn_cluster_size: Option<u8>,
    pub max_spawn_cluster_size_owner: MobClass,
    pub is_max_group_size_reached: MobClass,
    pub remove_when_far_away: MobClass,
    pub requires_custom_persistence: MobClass,
    pub spawn_dimensions_scale: f32,
    pub immune_to: &'static str,
}

impl MobSpawn {
    pub const fn placement_type(&self) -> SpawnPlacementType {
        match self.placement {
            Some(placement) => placement.kind,
            None => SpawnPlacementType::NoRestrictions,
        }
    }

    pub const fn heightmap(&self) -> SpawnHeightmap {
        match self.placement {
            Some(placement) => placement.heightmap,
            None => SpawnHeightmap::MotionBlockingNoLeaves,
        }
    }
}
