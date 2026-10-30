use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use rspigot_core::ports::{ChunkStore, StoreError};
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::world::WorldHeight;
use rspigot_domain::{ChunkPos, Identifier};

use crate::chunk;
use crate::compression::Compression;
use crate::error::AnvilError;
use crate::layout::DimensionLayout;
use crate::region::{
    ChunkBody, RegionFile, StoredChunk, needs_external, region_file_name, stage_external,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnvilOptions {
    pub compression: Compression,
    pub max_open_regions: usize,
    pub sync_writes: bool,
}

impl Default for AnvilOptions {
    fn default() -> Self {
        Self {
            compression: Compression::Zlib,
            max_open_regions: 256,
            sync_writes: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegionKey {
    dimension: Identifier,
    x: i32,
    z: i32,
}

type Region = Mutex<Option<RegionFile>>;

#[derive(Debug)]
struct Slot {
    region: Arc<Region>,
    used: u64,
}

#[derive(Debug, Default)]
struct RegionCache {
    slots: HashMap<RegionKey, Slot>,
    clock: u64,
}

#[derive(Debug)]
pub struct AnvilStore {
    root: PathBuf,
    layout: DimensionLayout,
    options: AnvilOptions,
    heights: Vec<(Identifier, WorldHeight)>,
    cache: Mutex<RegionCache>,
}

impl AnvilStore {
    pub fn open(world_root: &Path, layout: DimensionLayout) -> Result<Self, StoreError> {
        Self::open_with(world_root, layout, AnvilOptions::default())
    }

    pub fn open_with(
        world_root: &Path,
        layout: DimensionLayout,
        mut options: AnvilOptions,
    ) -> Result<Self, StoreError> {
        options.max_open_regions = options.max_open_regions.max(1);
        for dimension in ["overworld", "the_nether", "the_end"] {
            layout
                .region_dir(world_root, &Identifier::minecraft(dimension))
                .map_err(AnvilError::into_store_io)?;
        }
        Ok(Self {
            root: world_root.to_path_buf(),
            layout,
            options,
            heights: vec![
                (Identifier::minecraft("overworld"), WorldHeight::OVERWORLD),
                (Identifier::minecraft("the_nether"), WorldHeight::NETHER),
                (Identifier::minecraft("the_end"), WorldHeight::END),
            ],
            cache: Mutex::new(RegionCache::default()),
        })
    }

    pub fn with_dimension_height(mut self, dimension: Identifier, height: WorldHeight) -> Self {
        match self
            .heights
            .iter_mut()
            .find(|(known, _)| *known == dimension)
        {
            Some(entry) => entry.1 = height,
            None => self.heights.push((dimension, height)),
        }
        self
    }

    pub fn world_root(&self) -> &Path {
        &self.root
    }

    pub fn layout(&self) -> DimensionLayout {
        self.layout
    }

    pub fn options(&self) -> AnvilOptions {
        self.options
    }

    pub fn cached_regions(&self) -> usize {
        self.cache.lock().slots.len()
    }

    pub fn exists(&self, dimension: &Identifier, pos: ChunkPos) -> Result<bool, StoreError> {
        self.chunk_exists(dimension, pos)
            .map_err(|error| error.into_store(pos))
    }

    fn chunk_exists(&self, dimension: &Identifier, pos: ChunkPos) -> Result<bool, AnvilError> {
        let region = self.acquire(dimension, pos);
        let mut guard = region.lock();
        if guard.is_none() {
            let path = self.region_path(dimension, pos)?;
            *guard = RegionFile::open(&path, false, self.options.sync_writes)?;
        }
        Ok(guard.as_ref().is_some_and(|file| file.has_chunk(pos)))
    }

    fn height_for(&self, dimension: &Identifier) -> Option<WorldHeight> {
        self.heights
            .iter()
            .find(|(known, _)| known == dimension)
            .map(|(_, height)| *height)
    }

    fn acquire(&self, dimension: &Identifier, pos: ChunkPos) -> Arc<Region> {
        let key = RegionKey {
            dimension: dimension.clone(),
            x: pos.x >> 5,
            z: pos.z >> 5,
        };
        let mut evicted = Vec::new();
        let region = {
            let mut cache = self.cache.lock();
            cache.clock += 1;
            let tick = cache.clock;
            if let Some(slot) = cache.slots.get_mut(&key) {
                slot.used = tick;
                return Arc::clone(&slot.region);
            }
            let region: Arc<Region> = Arc::new(Mutex::new(None));
            cache.slots.insert(
                key,
                Slot {
                    region: Arc::clone(&region),
                    used: tick,
                },
            );
            while cache.slots.len() > self.options.max_open_regions {
                let victim = cache
                    .slots
                    .iter()
                    .filter(|(_, slot)| Arc::strong_count(&slot.region) == 1)
                    .min_by_key(|(_, slot)| slot.used)
                    .map(|(key, _)| key.clone());
                match victim.and_then(|key| cache.slots.remove(&key)) {
                    Some(slot) => evicted.push(slot.region),
                    None => break,
                }
            }
            region
        };
        for region in evicted {
            close_region(region);
        }
        region
    }

    fn region_path(&self, dimension: &Identifier, pos: ChunkPos) -> Result<PathBuf, AnvilError> {
        Ok(self
            .layout
            .region_dir(&self.root, dimension)?
            .join(region_file_name(pos)))
    }

    fn read_stored(
        &self,
        dimension: &Identifier,
        pos: ChunkPos,
    ) -> Result<Option<StoredChunk>, AnvilError> {
        let region = self.acquire(dimension, pos);
        let mut guard = region.lock();
        if guard.is_none() {
            let path = self.region_path(dimension, pos)?;
            *guard = RegionFile::open(&path, false, self.options.sync_writes)?;
        }
        match guard.as_mut() {
            Some(file) => file.read(pos),
            None => Ok(None),
        }
    }

    fn load_chunk(
        &self,
        dimension: &Identifier,
        pos: ChunkPos,
    ) -> Result<Option<ChunkColumn>, AnvilError> {
        let Some(stored) = self.read_stored(dimension, pos)? else {
            return Ok(None);
        };
        let compression = Compression::from_id(stored.compression_id)
            .ok_or(AnvilError::UnsupportedCompression(stored.compression_id))?;
        let bytes = compression.decompress(&stored.data)?;
        chunk::decode(&bytes, pos, self.height_for(dimension))
    }

    fn save_chunk(&self, dimension: &Identifier, column: &ChunkColumn) -> Result<(), AnvilError> {
        let pos = column.pos();
        if let Some(expected) = self.height_for(dimension)
            && expected != column.height()
        {
            return Err(AnvilError::corrupt(format!(
                "chunk height {:?} does not match the height {expected:?} of {dimension}",
                column.height()
            )));
        }
        let compression = self.options.compression;
        let data = compression.compress(&chunk::encode(column))?;
        let path = self.region_path(dimension, pos)?;
        let staged = if needs_external(data.len()) {
            let dir = path.parent().unwrap_or_else(|| Path::new("."));
            Some(stage_external(dir, pos, &data)?)
        } else {
            None
        };
        let result = self.commit(
            dimension,
            pos,
            &path,
            compression.id(),
            &data,
            staged.as_deref(),
        );
        if result.is_err()
            && let Some(staged) = &staged
        {
            let _ = std::fs::remove_file(staged);
        }
        result
    }

    fn commit(
        &self,
        dimension: &Identifier,
        pos: ChunkPos,
        path: &Path,
        compression_id: u8,
        data: &[u8],
        staged: Option<&Path>,
    ) -> Result<(), AnvilError> {
        let region = self.acquire(dimension, pos);
        let mut guard = region.lock();
        if guard.is_none() {
            *guard = RegionFile::open(path, true, self.options.sync_writes)?;
        }
        let body = match staged {
            Some(staged) => ChunkBody::Staged(staged),
            None => ChunkBody::Inline(data),
        };
        match guard.as_mut() {
            Some(file) => file.write(pos, compression_id, body),
            None => Err(AnvilError::corrupt("region file could not be created")),
        }
    }
}

fn close_region(region: Arc<Region>) {
    let Ok(region) = Arc::try_unwrap(region) else {
        return;
    };
    if let Some(mut file) = region.into_inner()
        && file.is_dirty()
        && let Err(error) = file.sync_all()
    {
        tracing::warn!(%error, "failed to sync an evicted region file");
    }
}

impl ChunkStore for AnvilStore {
    fn load(
        &self,
        dimension: &Identifier,
        pos: ChunkPos,
    ) -> Result<Option<ChunkColumn>, StoreError> {
        self.load_chunk(dimension, pos)
            .map_err(|error| error.into_store(pos))
    }

    fn save(&self, dimension: &Identifier, column: &ChunkColumn) -> Result<(), StoreError> {
        self.save_chunk(dimension, column)
            .map_err(|error| error.into_store(column.pos()))
    }

    fn flush(&self) -> Result<(), StoreError> {
        let regions: Vec<Arc<Region>> = self
            .cache
            .lock()
            .slots
            .values()
            .map(|slot| Arc::clone(&slot.region))
            .collect();
        let mut failure = None;
        for region in regions {
            let mut guard = region.lock();
            if let Some(file) = guard.as_mut()
                && file.is_dirty()
                && let Err(error) = file.sync_all()
            {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), |error| Err(error.into_store_io()))
    }
}

impl Drop for AnvilStore {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            tracing::warn!(%error, "failed to flush the chunk store on drop");
        }
    }
}
