use rspigot_domain::{BlockStateId, Vec3};

use crate::events::Direction;

pub fn horizontal_facing(yaw: f32) -> &'static str {
    let index = ((f64::from(yaw) / 90.0 + 0.5).floor() as i64).rem_euclid(4);
    match index {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

fn opposite(direction: &str) -> &'static str {
    match direction {
        "south" => "north",
        "north" => "south",
        "east" => "west",
        "west" => "east",
        "up" => "down",
        _ => "up",
    }
}

const fn face_name(face: Direction) -> &'static str {
    match face {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

const DIRECTIONAL: [&str; 4] = ["piston", "observer", "dispenser", "dropper"];

const FACES_PLAYER: [&str; 8] = [
    "stairs",
    "door",
    "fence_gate",
    "bed",
    "anvil",
    "repeater",
    "comparator",
    "glazed_terracotta",
];

pub fn placed_state(
    default: BlockStateId,
    face: Direction,
    yaw: f32,
    pitch: f32,
    cursor: Vec3,
) -> BlockStateId {
    let Some(name) = rspigot_data::blocks::block_name(default.0) else {
        return default;
    };
    let base = wall_variant(name, face).unwrap_or(default.0);
    let mut state = base;
    let set = |state: u16, key: &str, value: &str| {
        rspigot_data::blocks::with_property(state, key, value).unwrap_or(state)
    };
    let looking = horizontal_facing(yaw);
    if rspigot_data::blocks::property(state, "axis").is_some() {
        let axis = match face {
            Direction::Up | Direction::Down => "y",
            Direction::North | Direction::South => "z",
            Direction::East | Direction::West => "x",
        };
        state = set(state, "axis", axis);
    }
    if let Some(current) = rspigot_data::blocks::property(state, "facing") {
        let supports_vertical =
            rspigot_data::blocks::with_property(state, "facing", "up").is_some();
        let facing = if base != default.0 {
            face_name(face)
        } else if supports_vertical && DIRECTIONAL.iter().any(|d| name.ends_with(d)) {
            let vertical = if pitch > 45.0 {
                Some("up")
            } else if pitch < -45.0 {
                Some("down")
            } else {
                None
            };
            let toward = vertical.unwrap_or(looking);
            if name.ends_with("observer") {
                toward
            } else {
                opposite(toward)
            }
        } else if FACES_PLAYER.iter().any(|f| name.ends_with(f)) {
            looking
        } else if current == "up" || current == "down" {
            face_name(face)
        } else {
            opposite(looking)
        };
        state = set(state, "facing", facing);
    }
    let upper = match face {
        Direction::Down => true,
        Direction::Up => false,
        _ => cursor.y > 0.5,
    };
    if name.ends_with("_slab") {
        state = set(state, "type", if upper { "top" } else { "bottom" });
    }
    if rspigot_data::blocks::property(state, "half").is_some_and(|h| h == "top" || h == "bottom") {
        state = set(state, "half", if upper { "top" } else { "bottom" });
    }
    BlockStateId(state)
}

fn wall_variant(name: &str, face: Direction) -> Option<u16> {
    if matches!(face, Direction::Up | Direction::Down) {
        return None;
    }
    let path = name.strip_prefix("minecraft:")?;
    let wall = if let Some(prefix) = path.strip_suffix("torch") {
        format!("minecraft:{prefix}wall_torch")
    } else if let Some(prefix) = path
        .strip_suffix("_sign")
        .filter(|p| !p.ends_with("_hanging"))
    {
        format!("minecraft:{prefix}_wall_sign")
    } else if let Some(prefix) = path.strip_suffix("_banner") {
        format!("minecraft:{prefix}_wall_banner")
    } else if let Some(prefix) = path.strip_suffix("_head") {
        format!("minecraft:{prefix}_wall_head")
    } else {
        let prefix = path.strip_suffix("_skull")?;
        format!("minecraft:{prefix}_wall_skull")
    };
    rspigot_data::blocks::default_state(&wall)
}

pub fn is_replaceable(state: BlockStateId) -> bool {
    if state.is_air() {
        return true;
    }
    let Some(name) = rspigot_data::blocks::block_name(state.0) else {
        return false;
    };
    matches!(
        name,
        "minecraft:water"
            | "minecraft:lava"
            | "minecraft:short_grass"
            | "minecraft:tall_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:dead_bush"
            | "minecraft:snow"
            | "minecraft:vine"
            | "minecraft:seagrass"
            | "minecraft:tall_seagrass"
            | "minecraft:fire"
            | "minecraft:soul_fire"
            | "minecraft:structure_void"
            | "minecraft:glow_lichen"
            | "minecraft:short_dry_grass"
            | "minecraft:tall_dry_grass"
            | "minecraft:bush"
            | "minecraft:leaf_litter"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(name: &str) -> BlockStateId {
        BlockStateId(rspigot_data::blocks::default_state(name).unwrap_or(0))
    }

    #[test]
    fn yaw_maps_to_vanilla_directions() {
        assert_eq!(horizontal_facing(0.0), "south");
        assert_eq!(horizontal_facing(90.0), "west");
        assert_eq!(horizontal_facing(180.0), "north");
        assert_eq!(horizontal_facing(-90.0), "east");
        assert_eq!(horizontal_facing(359.0), "south");
    }

    #[test]
    fn log_axis_follows_clicked_face() {
        let placed = placed_state(
            state("minecraft:oak_log"),
            Direction::East,
            0.0,
            0.0,
            Vec3::ZERO,
        );
        assert_eq!(rspigot_data::blocks::property(placed.0, "axis"), Some("x"));
    }

    #[test]
    fn furnace_faces_the_player() {
        let placed = placed_state(
            state("minecraft:furnace"),
            Direction::Up,
            180.0,
            0.0,
            Vec3::ZERO,
        );
        assert_eq!(
            rspigot_data::blocks::property(placed.0, "facing"),
            Some("south")
        );
    }

    #[test]
    fn slab_on_upper_half_is_top() {
        let placed = placed_state(
            state("minecraft:oak_slab"),
            Direction::North,
            0.0,
            0.0,
            Vec3::new(0.5, 0.75, 0.0),
        );
        assert_eq!(
            rspigot_data::blocks::property(placed.0, "type"),
            Some("top")
        );
    }

    #[test]
    fn torch_on_wall_becomes_wall_torch() {
        let placed = placed_state(
            state("minecraft:torch"),
            Direction::North,
            0.0,
            0.0,
            Vec3::ZERO,
        );
        assert_eq!(
            rspigot_data::blocks::block_name(placed.0),
            Some("minecraft:wall_torch")
        );
        assert_eq!(
            rspigot_data::blocks::property(placed.0, "facing"),
            Some("north")
        );
    }
}
