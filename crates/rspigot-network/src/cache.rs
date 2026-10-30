use bytes::Bytes;
use quick_cache::Weighter;
use quick_cache::sync::Cache;
use rspigot_domain::{ChunkPos, Identifier};

pub type ChunkKey = (Identifier, ChunkPos, u64);

#[derive(Clone, Copy)]
struct FrameWeight;

impl Weighter<ChunkKey, Bytes> for FrameWeight {
    fn weight(&self, _key: &ChunkKey, frame: &Bytes) -> u64 {
        frame.len() as u64 + 64
    }
}

pub struct ChunkPacketCache {
    frames: Cache<ChunkKey, Bytes, FrameWeight>,
}

impl ChunkPacketCache {
    pub fn new(max_bytes: u64) -> Self {
        let estimated = (max_bytes / 16_384).max(64) as usize;
        Self {
            frames: Cache::with_weighter(estimated, max_bytes, FrameWeight),
        }
    }

    pub fn get_or_build<E>(
        &self,
        key: ChunkKey,
        build: impl FnOnce() -> Result<Bytes, E>,
    ) -> Result<Bytes, E> {
        self.frames.get_or_insert_with(&key, build)
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn builds_once_per_revision() {
        let cache = ChunkPacketCache::new(1 << 20);
        let builds = Cell::new(0);
        let key = (Identifier::minecraft("overworld"), ChunkPos::new(1, 2), 3);
        for _ in 0..3 {
            let frame: Result<Bytes, ()> = cache.get_or_build(key.clone(), || {
                builds.set(builds.get() + 1);
                Ok(Bytes::from_static(b"frame"))
            });
            assert_eq!(frame.as_deref(), Ok(&b"frame"[..]));
        }
        assert_eq!(builds.get(), 1);
        let next: Result<Bytes, ()> = cache.get_or_build(
            (Identifier::minecraft("overworld"), ChunkPos::new(1, 2), 4),
            || {
                builds.set(builds.get() + 1);
                Ok(Bytes::from_static(b"frame2"))
            },
        );
        assert!(next.is_ok());
        assert_eq!(builds.get(), 2);
    }
}
