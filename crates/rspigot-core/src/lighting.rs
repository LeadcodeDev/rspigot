use std::collections::VecDeque;
use std::sync::OnceLock;

use rspigot_domain::BlockStateId;
use rspigot_domain::chunk::{ChunkColumn, LightArray};

const MAX_LIGHT: u8 = 15;

struct LightProperties {
    opacity: Box<[u8]>,
    emission: Box<[u8]>,
}

fn properties() -> &'static LightProperties {
    static TABLE: OnceLock<LightProperties> = OnceLock::new();
    TABLE.get_or_init(build_properties)
}

fn build_properties() -> LightProperties {
    let count = rspigot_data::blocks::STATE_COUNT;
    let mut opacity = vec![MAX_LIGHT; count];
    let mut emission = vec![0u8; count];
    for state in 0..count {
        let id = state as u16;
        let name = rspigot_data::blocks::block_name(id).unwrap_or("");
        let path = name.strip_prefix("minecraft:").unwrap_or(name);
        opacity[state] = opacity_of(id, path);
        emission[state] = emission_of(id, path);
    }
    LightProperties {
        opacity: opacity.into_boxed_slice(),
        emission: emission.into_boxed_slice(),
    }
}

fn opacity_of(state: u16, path: &str) -> u8 {
    if rspigot_data::blocks::is_air(state) {
        return 0;
    }
    const FILTERING: [&str; 5] = ["water", "leaves", "ice", "cobweb", "slime_block"];
    const TRANSPARENT: [&str; 40] = [
        "glass",
        "pane",
        "torch",
        "flower",
        "sapling",
        "grass",
        "fern",
        "bush",
        "door",
        "trapdoor",
        "fence",
        "wall",
        "rail",
        "button",
        "pressure_plate",
        "lever",
        "sign",
        "banner",
        "carpet",
        "snow",
        "ladder",
        "vine",
        "lichen",
        "roots",
        "mushroom",
        "sugar_cane",
        "kelp",
        "seagrass",
        "coral",
        "wire",
        "repeater",
        "comparator",
        "candle",
        "lantern",
        "chain",
        "bars",
        "slab",
        "stairs",
        "head",
        "skull",
    ];
    const CROPS: [&str; 10] = [
        "wheat",
        "carrots",
        "potatoes",
        "beetroots",
        "dandelion",
        "poppy",
        "tulip",
        "orchid",
        "allium",
        "azure_bluet",
    ];
    if path == "water" || path == "bubble_column" {
        return 1;
    }
    if FILTERING.iter().any(|f| path.ends_with(f)) {
        return 1;
    }
    let full_block = path.ends_with("_block")
        || path.ends_with("_ore")
        || path.ends_with("_planks")
        || path == "tinted_glass";
    if !full_block
        && TRANSPARENT
            .iter()
            .chain(CROPS.iter())
            .any(|t| path.contains(t))
    {
        return 0;
    }
    MAX_LIGHT
}

fn emission_of(state: u16, path: &str) -> u8 {
    let lit = rspigot_data::blocks::property(state, "lit");
    match path {
        "glowstone"
        | "sea_lantern"
        | "jack_o_lantern"
        | "beacon"
        | "conduit"
        | "lava"
        | "fire"
        | "shroomlight"
        | "end_gateway"
        | "end_portal"
        | "ochre_froglight"
        | "verdant_froglight"
        | "pearlescent_froglight"
        | "lantern"
        | "respawn_anchor" => 15,
        "campfire" | "redstone_lamp" | "furnace" | "smoker" | "blast_furnace" => {
            if lit == Some("true") {
                if path == "furnace" || path == "smoker" || path == "blast_furnace" {
                    13
                } else {
                    15
                }
            } else {
                0
            }
        }
        "torch" | "wall_torch" | "end_rod" | "copper_torch" | "copper_wall_torch" => 14,
        "soul_campfire" => {
            if lit == Some("true") {
                10
            } else {
                0
            }
        }
        "soul_torch" | "soul_wall_torch" | "soul_lantern" | "soul_fire" | "crying_obsidian" => 10,
        "nether_portal" => 11,
        "redstone_torch" | "redstone_wall_torch" => {
            if lit == Some("false") {
                0
            } else {
                7
            }
        }
        "glow_lichen"
        | "cave_vines"
        | "cave_vines_plant"
        | "redstone_ore"
        | "deepslate_redstone_ore" => {
            if path.contains("redstone") && lit != Some("true") {
                0
            } else {
                7
            }
        }
        "magma_block" => 3,
        "brewing_stand" | "brown_mushroom" | "dragon_egg" | "end_portal_frame" | "sculk_sensor" => {
            1
        }
        "amethyst_cluster" => 5,
        "large_amethyst_bud" => 4,
        "medium_amethyst_bud" => 2,
        "small_amethyst_bud" => 1,
        "enchanting_table" | "ender_chest" => 7,
        _ => 0,
    }
}

pub fn opacity(state: BlockStateId) -> u8 {
    properties()
        .opacity
        .get(usize::from(state.0))
        .copied()
        .unwrap_or(MAX_LIGHT)
}

pub fn emission(state: BlockStateId) -> u8 {
    properties()
        .emission
        .get(usize::from(state.0))
        .copied()
        .unwrap_or(0)
}

pub fn has_light(chunk: &ChunkColumn) -> bool {
    chunk
        .light
        .sky
        .iter()
        .chain(chunk.light.block.iter())
        .any(Option::is_some)
}

struct Volume {
    height: usize,
    levels: Vec<u8>,
}

impl Volume {
    fn new(height: usize) -> Self {
        Self {
            height,
            levels: vec![0; 256 * height],
        }
    }

    const fn index(x: usize, y: usize, z: usize) -> usize {
        (y << 8) | (z << 4) | x
    }
}

struct Cells<'a> {
    chunk: &'a ChunkColumn,
    opacity: Vec<u8>,
}

impl<'a> Cells<'a> {
    fn new(chunk: &'a ChunkColumn) -> Self {
        let height = chunk.height().height as usize;
        let mut opacity = vec![0u8; 256 * height];
        for (section_index, section) in chunk.sections().iter().enumerate() {
            if section.is_empty() {
                continue;
            }
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        opacity[Volume::index(x, section_index * 16 + y, z)] =
                            self::opacity(section.block(x, y, z));
                    }
                }
            }
        }
        Self { chunk, opacity }
    }
}

fn propagate(volume: &mut Volume, cells: &Cells<'_>, mut queue: VecDeque<(usize, usize, usize)>) {
    while let Some((x, y, z)) = queue.pop_front() {
        let level = volume.levels[Volume::index(x, y, z)];
        if level <= 1 {
            continue;
        }
        let mut visit = |nx: usize, ny: usize, nz: usize| {
            let index = Volume::index(nx, ny, nz);
            let cost = cells.opacity[index].max(1);
            if cost >= MAX_LIGHT {
                return;
            }
            let next = level.saturating_sub(cost);
            if next > volume.levels[index] {
                volume.levels[index] = next;
                queue.push_back((nx, ny, nz));
            }
        };
        if x > 0 {
            visit(x - 1, y, z);
        }
        if x < 15 {
            visit(x + 1, y, z);
        }
        if z > 0 {
            visit(x, y, z - 1);
        }
        if z < 15 {
            visit(x, y, z + 1);
        }
        if y > 0 {
            visit(x, y - 1, z);
        }
        if y + 1 < volume.height {
            visit(x, y + 1, z);
        }
    }
}

fn sky_volume(cells: &Cells<'_>, height: usize) -> Volume {
    let mut volume = Volume::new(height);
    for z in 0..16 {
        for x in 0..16 {
            let mut level = MAX_LIGHT;
            for y in (0..height).rev() {
                let cost = cells.opacity[Volume::index(x, y, z)];
                level = level.saturating_sub(cost);
                volume.levels[Volume::index(x, y, z)] = level;
                if level == 0 {
                    break;
                }
            }
        }
    }
    let mut queue = VecDeque::new();
    for y in 0..height {
        for z in 0..16 {
            for x in 0..16 {
                let level = volume.levels[Volume::index(x, y, z)];
                if level < 2 {
                    continue;
                }
                let neighbours_darker = [
                    (x.wrapping_sub(1), z),
                    (x + 1, z),
                    (x, z.wrapping_sub(1)),
                    (x, z + 1),
                ]
                .iter()
                .any(|(nx, nz)| {
                    *nx < 16 && *nz < 16 && volume.levels[Volume::index(*nx, y, *nz)] + 1 < level
                });
                let below_darker = y > 0 && volume.levels[Volume::index(x, y - 1, z)] + 1 < level;
                if neighbours_darker || below_darker {
                    queue.push_back((x, y, z));
                }
            }
        }
    }
    propagate(&mut volume, cells, queue);
    volume
}

fn block_volume(cells: &Cells<'_>, height: usize) -> Volume {
    let mut volume = Volume::new(height);
    let mut queue = VecDeque::new();
    for (section_index, section) in cells.chunk.sections().iter().enumerate() {
        if section.is_empty() {
            continue;
        }
        let emitting: Vec<bool> = section
            .blocks()
            .palette()
            .iter()
            .map(|s| emission(*s) > 0)
            .collect();
        if !emitting.iter().any(|e| *e) {
            continue;
        }
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let state = section.block(x, y, z);
                    let level = emission(state);
                    if level > 0 {
                        let wy = section_index * 16 + y;
                        volume.levels[Volume::index(x, wy, z)] = level;
                        queue.push_back((x, wy, z));
                    }
                }
            }
        }
    }
    propagate(&mut volume, cells, queue);
    volume
}

fn to_sections(
    volume: &Volume,
    sections: usize,
    above: Option<LightArray>,
) -> Vec<Option<LightArray>> {
    let mut out = Vec::with_capacity(sections + 2);
    out.push(None);
    for section in 0..sections {
        let base = section * 16;
        let slice = &volume.levels[Volume::index(0, base, 0)..Volume::index(0, base + 16, 0)];
        if slice.iter().all(|l| *l == 0) {
            out.push(None);
            continue;
        }
        let mut array = LightArray::filled(0);
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    array.set(x, y, z, volume.levels[Volume::index(x, base + y, z)]);
                }
            }
        }
        out.push(Some(array));
    }
    out.push(above);
    out
}

pub fn compute(chunk: &mut ChunkColumn, has_skylight: bool) {
    let height = chunk.height().height as usize;
    let sections = chunk.height().section_count();
    let (sky, block) = {
        let cells = Cells::new(chunk);
        let sky = has_skylight.then(|| {
            to_sections(
                &sky_volume(&cells, height),
                sections,
                Some(LightArray::filled(MAX_LIGHT)),
            )
        });
        let block = to_sections(&block_volume(&cells, height), sections, None);
        (sky, block)
    };
    chunk.light.sky = sky.unwrap_or_else(|| vec![None; sections + 2]);
    chunk.light.block = block;
}

#[cfg(test)]
mod tests {
    use rspigot_domain::chunk::ChunkColumn;
    use rspigot_domain::world::WorldHeight;
    use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos};

    use super::*;

    fn stone() -> BlockStateId {
        BlockStateId(1)
    }

    #[test]
    fn open_sky_is_full_above_ground_and_dark_below() {
        let mut chunk = ChunkColumn::empty(ChunkPos::new(0, 0), WorldHeight::OVERWORLD, BiomeId(0));
        for x in 0..16 {
            for z in 0..16 {
                for y in -64..64 {
                    chunk.set_block(BlockPos::new(x, y, z), stone());
                }
            }
        }
        compute(&mut chunk, true);
        let above = chunk.light.sky[1 + 8].as_ref().map(|a| a.get(0, 0, 0));
        assert_eq!(above, Some(15));
        assert!(chunk.light.sky[1].is_none());
        assert_eq!(chunk.light.sky.len(), 26);
    }

    #[test]
    fn torch_lights_neighbourhood() {
        let mut chunk = ChunkColumn::empty(ChunkPos::new(0, 0), WorldHeight::OVERWORLD, BiomeId(0));
        let torch =
            BlockStateId(rspigot_data::blocks::default_state("minecraft:torch").unwrap_or(0));
        chunk.set_block(BlockPos::new(8, 0, 8), torch);
        compute(&mut chunk, false);
        let section = chunk.light.block[1 + 4]
            .as_ref()
            .map(|a| (a.get(8, 0, 8), a.get(9, 0, 8), a.get(12, 0, 8)));
        assert_eq!(section, Some((14, 13, 10)));
    }
}
