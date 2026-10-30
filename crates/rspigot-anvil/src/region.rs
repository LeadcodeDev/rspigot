use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rspigot_domain::ChunkPos;

use crate::bitmap::SectorBitmap;
use crate::error::AnvilError;

pub const SECTOR_BYTES: usize = 4096;
pub const HEADER_BYTES: usize = 8192;
pub const EXTERNAL_FLAG: u8 = 0x80;
const CHUNK_HEADER_BYTES: usize = 5;
const EXTERNAL_SECTOR_THRESHOLD: usize = 256;
const ENTRIES: usize = 1024;
const OVERFLOW_SECTORS: u32 = 255;

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Eq)]
pub struct StoredChunk {
    pub compression_id: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum ChunkBody<'a> {
    Inline(&'a [u8]),
    Staged(&'a Path),
}

#[derive(Debug)]
pub struct RegionFile {
    file: File,
    dir: PathBuf,
    header: Box<[u8; HEADER_BYTES]>,
    bitmap: SectorBitmap,
    sync: bool,
    dirty: bool,
}

pub fn region_file_name(pos: ChunkPos) -> String {
    format!("r.{}.{}.mca", pos.x >> 5, pos.z >> 5)
}

pub fn external_path(dir: &Path, pos: ChunkPos) -> PathBuf {
    dir.join(format!("c.{}.{}.mcc", pos.x, pos.z))
}

pub fn local_index(pos: ChunkPos) -> usize {
    (pos.x & 31) as usize + (pos.z & 31) as usize * 32
}

fn sectors_for(total_bytes: usize) -> usize {
    total_bytes.div_ceil(SECTOR_BYTES)
}

pub fn needs_external(data_len: usize) -> bool {
    sectors_for(data_len + CHUNK_HEADER_BYTES) >= EXTERNAL_SECTOR_THRESHOLD
}

fn overflow_count(length: u32) -> u32 {
    let sectors = (u64::from(length) + 4).div_ceil(SECTOR_BYTES as u64);
    sectors.clamp(u64::from(OVERFLOW_SECTORS), 0x00FF_FFFF) as u32
}

fn now_seconds() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as u32)
}

fn entry_parts(entry: u32) -> (u32, u32) {
    ((entry >> 8) & 0x00FF_FFFF, entry & 0xFF)
}

pub fn stage_external(dir: &Path, pos: ChunkPos, data: &[u8]) -> Result<PathBuf, AnvilError> {
    std::fs::create_dir_all(dir)?;
    let unique = STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let staged = dir.join(format!(
        "c.{}.{}.mcc.{}-{unique}.tmp",
        pos.x,
        pos.z,
        std::process::id()
    ));
    let result = File::create(&staged).and_then(|mut file| file.write_all(data));
    if let Err(error) = result {
        let _ = std::fs::remove_file(&staged);
        return Err(error.into());
    }
    Ok(staged)
}

impl RegionFile {
    pub fn open(path: &Path, create: bool, sync: bool) -> Result<Option<Self>, AnvilError> {
        let dir = path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        if create {
            std::fs::create_dir_all(&dir)?;
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(create);
        let mut file = match options.open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !create => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        let file_len = file.metadata()?.len();
        let mut header = Box::new([0u8; HEADER_BYTES]);
        let mut filled = 0;
        while filled < HEADER_BYTES {
            let read = file.read(&mut header[filled..])?;
            if read == 0 {
                break;
            }
            filled += read;
        }
        if filled != 0 && filled != HEADER_BYTES {
            tracing::warn!(path = %path.display(), read = filled, "region file has a truncated header");
        }
        let mut region = Self {
            file,
            dir,
            header,
            bitmap: SectorBitmap::new(),
            sync,
            dirty: false,
        };
        region.bitmap.force(0, 2);
        region.validate_entries(path, file_len)?;
        if create && file_len < HEADER_BYTES as u64 {
            region.write_header()?;
        }
        Ok(Some(region))
    }

    fn validate_entries(&mut self, path: &Path, file_len: u64) -> Result<(), AnvilError> {
        for index in 0..ENTRIES {
            let entry = self.entry(index);
            if entry == 0 {
                continue;
            }
            let (sector, mut count) = entry_parts(entry);
            if count == OVERFLOW_SECTORS {
                let mut length = [0u8; 4];
                self.file
                    .seek(SeekFrom::Start(u64::from(sector) * SECTOR_BYTES as u64))?;
                if self.file.read_exact(&mut length).is_ok() {
                    count = overflow_count(u32::from_be_bytes(length));
                }
            }
            let reason = if sector < 2 {
                Some("overlaps with the header")
            } else if count == 0 {
                Some("has a size of zero")
            } else if u64::from(sector) * SECTOR_BYTES as u64 > file_len {
                Some("is out of bounds")
            } else {
                None
            };
            match reason {
                Some(reason) => {
                    tracing::warn!(path = %path.display(), index, sector, "region file has an invalid sector entry that {reason}");
                    self.set_entry(index, 0);
                }
                None => self.bitmap.force(sector, count),
            }
        }
        Ok(())
    }

    fn entry(&self, index: usize) -> u32 {
        let at = index * 4;
        u32::from_be_bytes([
            self.header[at],
            self.header[at + 1],
            self.header[at + 2],
            self.header[at + 3],
        ])
    }

    fn set_entry(&mut self, index: usize, value: u32) {
        self.header[index * 4..index * 4 + 4].copy_from_slice(&value.to_be_bytes());
    }

    fn set_timestamp(&mut self, index: usize, value: u32) {
        let at = SECTOR_BYTES + index * 4;
        self.header[at..at + 4].copy_from_slice(&value.to_be_bytes());
    }

    fn write_header(&mut self) -> Result<(), AnvilError> {
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&self.header[..])?;
        Ok(())
    }

    pub fn has_chunk(&self, pos: ChunkPos) -> bool {
        self.entry(local_index(pos)) != 0
    }

    pub fn timestamp(&self, pos: ChunkPos) -> u32 {
        let at = SECTOR_BYTES + local_index(pos) * 4;
        u32::from_be_bytes([
            self.header[at],
            self.header[at + 1],
            self.header[at + 2],
            self.header[at + 3],
        ])
    }

    pub fn read(&mut self, pos: ChunkPos) -> Result<Option<StoredChunk>, AnvilError> {
        let entry = self.entry(local_index(pos));
        if entry == 0 {
            return Ok(None);
        }
        let (sector, mut count) = entry_parts(entry);
        let start = u64::from(sector) * SECTOR_BYTES as u64;
        self.file.seek(SeekFrom::Start(start))?;
        if count == OVERFLOW_SECTORS {
            let mut length = [0u8; 4];
            self.file.read_exact(&mut length)?;
            count = overflow_count(u32::from_be_bytes(length));
            self.file.seek(SeekFrom::Start(start))?;
        }
        let mut buffer = Vec::with_capacity(count as usize * SECTOR_BYTES);
        (&mut self.file)
            .take(u64::from(count) * SECTOR_BYTES as u64)
            .read_to_end(&mut buffer)?;
        if buffer.len() < CHUNK_HEADER_BYTES {
            return Err(AnvilError::corrupt(format!(
                "chunk {},{} header is truncated",
                pos.x, pos.z
            )));
        }
        let length = i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let tag = buffer[4];
        if tag == 0 {
            tracing::warn!(
                x = pos.x,
                z = pos.z,
                "chunk is allocated but its stream is missing"
            );
            return Ok(None);
        }
        let stream_len = i64::from(length) - 1;
        if stream_len < 0 {
            return Err(AnvilError::corrupt(format!(
                "declared size {length} of chunk {},{} is negative",
                pos.x, pos.z
            )));
        }
        let compression_id = tag & !EXTERNAL_FLAG;
        if tag & EXTERNAL_FLAG != 0 {
            let path = external_path(&self.dir, pos);
            if !path.is_file() {
                return Err(AnvilError::corrupt(format!(
                    "external chunk file {} is missing",
                    path.display()
                )));
            }
            let data = std::fs::read(&path)?;
            return Ok(Some(StoredChunk {
                compression_id,
                data,
            }));
        }
        let stream_len = stream_len as usize;
        if stream_len > buffer.len() - CHUNK_HEADER_BYTES {
            return Err(AnvilError::corrupt(format!(
                "chunk {},{} stream is truncated: expected {stream_len} but read {}",
                pos.x,
                pos.z,
                buffer.len() - CHUNK_HEADER_BYTES
            )));
        }
        buffer.truncate(CHUNK_HEADER_BYTES + stream_len);
        buffer.drain(..CHUNK_HEADER_BYTES);
        Ok(Some(StoredChunk {
            compression_id,
            data: buffer,
        }))
    }

    pub fn write(
        &mut self,
        pos: ChunkPos,
        compression_id: u8,
        body: ChunkBody<'_>,
    ) -> Result<(), AnvilError> {
        match body {
            ChunkBody::Inline(data) if needs_external(data.len()) => {
                let staged = stage_external(&self.dir, pos, data)?;
                let result = self.write_external(pos, compression_id, &staged);
                if result.is_err() {
                    let _ = std::fs::remove_file(&staged);
                }
                result
            }
            ChunkBody::Inline(data) => self.write_inline(pos, compression_id, data),
            ChunkBody::Staged(staged) => self.write_external(pos, compression_id, staged),
        }
    }

    fn write_inline(
        &mut self,
        pos: ChunkPos,
        compression_id: u8,
        data: &[u8],
    ) -> Result<(), AnvilError> {
        let sectors = sectors_for(CHUNK_HEADER_BYTES + data.len());
        let mut buffer = vec![0u8; sectors * SECTOR_BYTES];
        buffer[..4].copy_from_slice(&((data.len() + 1) as u32).to_be_bytes());
        buffer[4] = compression_id;
        buffer[CHUNK_HEADER_BYTES..CHUNK_HEADER_BYTES + data.len()].copy_from_slice(data);
        let old = self.commit_sectors(pos, &buffer, sectors as u32, || Ok(()))?;
        let external = external_path(&self.dir, pos);
        if let Err(error) = std::fs::remove_file(&external)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %external.display(), %error, "failed to remove a stale external chunk file");
        }
        self.release(old);
        Ok(())
    }

    fn write_external(
        &mut self,
        pos: ChunkPos,
        compression_id: u8,
        staged: &Path,
    ) -> Result<(), AnvilError> {
        let mut buffer = vec![0u8; SECTOR_BYTES];
        buffer[..4].copy_from_slice(&1u32.to_be_bytes());
        buffer[4] = compression_id | EXTERNAL_FLAG;
        let destination = external_path(&self.dir, pos);
        let old = self.commit_sectors(pos, &buffer, 1, || {
            std::fs::rename(staged, &destination).map_err(AnvilError::from)
        })?;
        self.release(old);
        Ok(())
    }

    fn commit_sectors(
        &mut self,
        pos: ChunkPos,
        buffer: &[u8],
        sectors: u32,
        before_header: impl FnOnce() -> Result<(), AnvilError>,
    ) -> Result<u32, AnvilError> {
        let index = local_index(pos);
        let old = self.entry(index);
        let start = self.bitmap.allocate(sectors);
        let result = self.write_at(start, buffer).and_then(|()| before_header());
        if let Err(error) = result {
            self.bitmap.free(start, sectors);
            return Err(error);
        }
        let old_timestamp = self.timestamp(pos);
        self.set_entry(index, (start << 8) | sectors.min(OVERFLOW_SECTORS));
        self.set_timestamp(index, now_seconds());
        let header_result = self.write_header().and_then(|()| self.sync_data());
        if let Err(error) = header_result {
            self.set_entry(index, old);
            self.set_timestamp(index, old_timestamp);
            self.bitmap.free(start, sectors);
            return Err(error);
        }
        self.dirty = !self.sync;
        Ok(old)
    }

    fn write_at(&mut self, sector: u32, buffer: &[u8]) -> Result<(), AnvilError> {
        self.file
            .seek(SeekFrom::Start(u64::from(sector) * SECTOR_BYTES as u64))?;
        self.file.write_all(buffer)?;
        self.sync_data()
    }

    fn sync_data(&mut self) -> Result<(), AnvilError> {
        if self.sync {
            self.file.sync_data()?;
        }
        Ok(())
    }

    fn release(&mut self, old_entry: u32) {
        if old_entry != 0 {
            let (sector, count) = entry_parts(old_entry);
            self.bitmap.free(sector, count);
        }
    }

    pub fn clear(&mut self, pos: ChunkPos) -> Result<(), AnvilError> {
        let index = local_index(pos);
        let old = self.entry(index);
        if old == 0 {
            return Ok(());
        }
        self.set_entry(index, 0);
        self.set_timestamp(index, now_seconds());
        self.write_header()?;
        self.sync_data()?;
        let external = external_path(&self.dir, pos);
        if let Err(error) = std::fs::remove_file(&external)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(error.into());
        }
        self.release(old);
        self.dirty = !self.sync;
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn sync_all(&mut self) -> Result<(), AnvilError> {
        self.file.sync_all()?;
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    fn open_new(dir: &TempDir) -> RegionFile {
        RegionFile::open(&dir.path().join("r.0.0.mca"), true, false)
            .unwrap()
            .unwrap()
    }

    fn pattern(len: usize, seed: u8) -> Vec<u8> {
        (0..len).map(|i| (i as u8).wrapping_add(seed)).collect()
    }

    fn header_entry(path: &Path, index: usize) -> u32 {
        let bytes = std::fs::read(path).unwrap();
        u32::from_be_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    }

    #[test]
    fn names_and_indices_follow_vanilla() {
        assert_eq!(region_file_name(ChunkPos::new(0, 0)), "r.0.0.mca");
        assert_eq!(region_file_name(ChunkPos::new(31, 32)), "r.0.1.mca");
        assert_eq!(region_file_name(ChunkPos::new(-1, -32)), "r.-1.-1.mca");
        assert_eq!(region_file_name(ChunkPos::new(-33, 64)), "r.-2.2.mca");
        assert_eq!(local_index(ChunkPos::new(0, 0)), 0);
        assert_eq!(local_index(ChunkPos::new(5, 7)), 5 + 7 * 32);
        assert_eq!(local_index(ChunkPos::new(-1, -1)), 1023);
        assert_eq!(local_index(ChunkPos::new(-32, 33)), 32);
        assert_eq!(
            external_path(Path::new("/w/region"), ChunkPos::new(-3, 40)),
            Path::new("/w/region/c.-3.40.mcc")
        );
    }

    #[test]
    fn missing_file_is_not_created_without_create() {
        let dir = TempDir::new("missing");
        let path = dir.path().join("r.0.0.mca");
        assert!(RegionFile::open(&path, false, false).unwrap().is_none());
        assert!(!path.exists());
    }

    #[test]
    fn fresh_region_has_no_chunks() {
        let dir = TempDir::new("fresh");
        let mut region = open_new(&dir);
        assert_eq!(region.read(ChunkPos::new(3, 4)).unwrap(), None);
        assert!(!region.has_chunk(ChunkPos::new(3, 4)));
    }

    #[test]
    fn write_then_read_and_header_layout() {
        let dir = TempDir::new("layout");
        let path = dir.path().join("r.0.0.mca");
        let mut region = open_new(&dir);
        let pos = ChunkPos::new(2, 1);
        let data = pattern(5000, 1);
        region.write(pos, 2, ChunkBody::Inline(&data)).unwrap();
        assert_eq!(
            region.read(pos).unwrap(),
            Some(StoredChunk {
                compression_id: 2,
                data: data.clone()
            })
        );
        assert!(region.has_chunk(pos));
        assert!(region.timestamp(pos) > 1_600_000_000);
        let index = 2 + 32;
        assert_eq!(header_entry(&path, index), (2 << 8) | 2);
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes.len(), HEADER_BYTES + 2 * SECTOR_BYTES);
        assert_eq!(&bytes[8192..8196], &(5001u32).to_be_bytes());
        assert_eq!(bytes[8196], 2);
        assert_eq!(&bytes[8197..8197 + 5000], &data[..]);
        assert!(bytes[8197 + 5000..].iter().all(|b| *b == 0));
        assert!(header_entry(&path, 1024 + index) > 1_600_000_000);
    }

    #[test]
    fn second_chunk_takes_the_next_free_sectors() {
        let dir = TempDir::new("second");
        let path = dir.path().join("r.0.0.mca");
        let mut region = open_new(&dir);
        region
            .write(ChunkPos::new(0, 0), 2, ChunkBody::Inline(&pattern(100, 0)))
            .unwrap();
        region
            .write(ChunkPos::new(1, 0), 2, ChunkBody::Inline(&pattern(100, 0)))
            .unwrap();
        assert_eq!(header_entry(&path, 0), (2 << 8) | 1);
        assert_eq!(header_entry(&path, 1), (3 << 8) | 1);
    }

    #[test]
    fn overwrite_relocates_and_frees_old_sectors() {
        let dir = TempDir::new("relocate");
        let path = dir.path().join("r.0.0.mca");
        let mut region = open_new(&dir);
        let a = ChunkPos::new(0, 0);
        let b = ChunkPos::new(1, 0);
        region
            .write(a, 2, ChunkBody::Inline(&pattern(100, 1)))
            .unwrap();
        assert_eq!(header_entry(&path, 0), (2 << 8) | 1);
        let bigger = pattern(10_000, 2);
        region.write(a, 2, ChunkBody::Inline(&bigger)).unwrap();
        assert_eq!(header_entry(&path, 0), (3 << 8) | 3);
        region
            .write(b, 2, ChunkBody::Inline(&pattern(100, 3)))
            .unwrap();
        assert_eq!(header_entry(&path, 1), (2 << 8) | 1);
        assert_eq!(region.read(a).unwrap().unwrap().data, bigger);
        assert_eq!(region.read(b).unwrap().unwrap().data, pattern(100, 3));
        let len = std::fs::metadata(&path).unwrap().len() as usize;
        assert_eq!(len, HEADER_BYTES + 4 * SECTOR_BYTES);
    }

    #[test]
    fn reopen_rebuilds_the_sector_bitmap() {
        let dir = TempDir::new("reopen");
        let path = dir.path().join("r.0.0.mca");
        {
            let mut region = open_new(&dir);
            for i in 0..4 {
                region
                    .write(
                        ChunkPos::new(i, 0),
                        2,
                        ChunkBody::Inline(&pattern(6000, i as u8)),
                    )
                    .unwrap();
            }
        }
        let mut region = RegionFile::open(&path, false, false).unwrap().unwrap();
        region
            .write(ChunkPos::new(9, 9), 2, ChunkBody::Inline(&pattern(10, 9)))
            .unwrap();
        for i in 0..4 {
            assert_eq!(
                region.read(ChunkPos::new(i, 0)).unwrap().unwrap().data,
                pattern(6000, i as u8)
            );
        }
        let index = 9 + 9 * 32;
        assert_eq!(header_entry(&path, index) >> 8, 10);
    }

    #[test]
    fn invalid_header_entries_are_discarded_on_open() {
        let dir = TempDir::new("invalid");
        let path = dir.path().join("r.0.0.mca");
        let mut bytes = vec![0u8; HEADER_BYTES + SECTOR_BYTES];
        bytes[0..4].copy_from_slice(&((1u32 << 8) | 1).to_be_bytes());
        bytes[4..8].copy_from_slice(&((50u32 << 8) | 1).to_be_bytes());
        bytes[8..12].copy_from_slice(&(2u32 << 8).to_be_bytes());
        bytes[12..16].copy_from_slice(&((2u32 << 8) | 1).to_be_bytes());
        bytes[8192..8196].copy_from_slice(&3u32.to_be_bytes());
        bytes[8196] = 3;
        bytes[8197..8199].copy_from_slice(&[7, 8]);
        std::fs::write(&path, &bytes).unwrap();
        let mut region = RegionFile::open(&path, false, false).unwrap().unwrap();
        assert!(!region.has_chunk(ChunkPos::new(0, 0)));
        assert!(!region.has_chunk(ChunkPos::new(1, 0)));
        assert!(!region.has_chunk(ChunkPos::new(2, 0)));
        assert_eq!(
            region.read(ChunkPos::new(3, 0)).unwrap(),
            Some(StoredChunk {
                compression_id: 3,
                data: vec![7, 8]
            })
        );
    }

    #[test]
    fn truncated_stream_is_corrupt() {
        let dir = TempDir::new("truncated");
        let path = dir.path().join("r.0.0.mca");
        let mut bytes = vec![0u8; HEADER_BYTES + SECTOR_BYTES];
        bytes[0..4].copy_from_slice(&((2u32 << 8) | 1).to_be_bytes());
        bytes[8192..8196].copy_from_slice(&9000u32.to_be_bytes());
        bytes[8196] = 2;
        std::fs::write(&path, &bytes).unwrap();
        let mut region = RegionFile::open(&path, false, false).unwrap().unwrap();
        assert!(matches!(
            region.read(ChunkPos::new(0, 0)),
            Err(AnvilError::Corrupt(_))
        ));
    }

    #[test]
    fn clear_removes_the_chunk_and_frees_its_sectors() {
        let dir = TempDir::new("region-clear");
        let mut region = open_new(&dir);
        let pos = ChunkPos::new(3, 4);
        region
            .write(pos, 2, ChunkBody::Inline(&pattern(5000, 7)))
            .unwrap();
        assert!(region.has_chunk(pos));
        region.clear(pos).unwrap();
        assert!(!region.has_chunk(pos));
        assert_eq!(region.read(pos).unwrap(), None);
        region.clear(pos).unwrap();
        let other = ChunkPos::new(5, 5);
        region
            .write(other, 2, ChunkBody::Inline(&pattern(100, 1)))
            .unwrap();
        assert_eq!(
            header_entry(&dir.path().join("r.0.0.mca"), local_index(other)) >> 8,
            2
        );
    }

    #[test]
    fn allocated_but_empty_stream_reads_as_missing() {
        let dir = TempDir::new("emptystream");
        let path = dir.path().join("r.0.0.mca");
        let mut bytes = vec![0u8; HEADER_BYTES + SECTOR_BYTES];
        bytes[0..4].copy_from_slice(&((2u32 << 8) | 1).to_be_bytes());
        std::fs::write(&path, &bytes).unwrap();
        let mut region = RegionFile::open(&path, false, false).unwrap().unwrap();
        assert_eq!(region.read(ChunkPos::new(0, 0)).unwrap(), None);
    }

    #[test]
    fn chunk_filling_255_sectors_stays_internal() {
        let dir = TempDir::new("max");
        let path = dir.path().join("r.0.0.mca");
        let mut region = open_new(&dir);
        let len = 255 * SECTOR_BYTES - CHUNK_HEADER_BYTES;
        assert!(!needs_external(len));
        assert!(needs_external(len + 1));
        let data = pattern(len, 4);
        region
            .write(ChunkPos::new(0, 0), 2, ChunkBody::Inline(&data))
            .unwrap();
        assert_eq!(header_entry(&path, 0), (2 << 8) | 255);
        drop(region);
        let mut region = RegionFile::open(&path, false, false).unwrap().unwrap();
        assert_eq!(
            region.read(ChunkPos::new(0, 0)).unwrap().unwrap().data,
            data
        );
        region
            .write(ChunkPos::new(1, 0), 2, ChunkBody::Inline(&pattern(10, 0)))
            .unwrap();
        assert_eq!(header_entry(&path, 1) >> 8, 257);
        assert_eq!(header_entry(&path, 0), (2 << 8) | 255);
    }

    #[test]
    fn oversized_chunk_goes_to_an_external_file() {
        let dir = TempDir::new("external");
        let path = dir.path().join("r.0.0.mca");
        let mut region = open_new(&dir);
        let pos = ChunkPos::new(4, 5);
        let data = pattern(1_100_000, 6);
        assert!(needs_external(data.len()));
        let staged = stage_external(dir.path(), pos, &data).unwrap();
        assert!(staged.exists());
        region.write(pos, 2, ChunkBody::Staged(&staged)).unwrap();
        assert!(!staged.exists());
        let mcc = external_path(dir.path(), pos);
        assert_eq!(std::fs::read(&mcc).unwrap(), data);
        let index = 4 + 5 * 32;
        assert_eq!(header_entry(&path, index), (2 << 8) | 1);
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[8192..8196], &1u32.to_be_bytes());
        assert_eq!(bytes[8196], 2 | EXTERNAL_FLAG);
        assert_eq!(
            region.read(pos).unwrap(),
            Some(StoredChunk {
                compression_id: 2,
                data: data.clone()
            })
        );
        region
            .write(pos, 2, ChunkBody::Inline(&pattern(50, 1)))
            .unwrap();
        assert!(!mcc.exists());
        assert_eq!(region.read(pos).unwrap().unwrap().data, pattern(50, 1));
    }

    #[test]
    fn missing_external_file_is_an_error() {
        let dir = TempDir::new("noexternal");
        let mut region = open_new(&dir);
        let pos = ChunkPos::new(0, 0);
        let data = pattern(1_100_000, 6);
        let staged = stage_external(dir.path(), pos, &data).unwrap();
        region.write(pos, 2, ChunkBody::Staged(&staged)).unwrap();
        std::fs::remove_file(external_path(dir.path(), pos)).unwrap();
        assert!(region.read(pos).is_err());
    }

    #[test]
    fn sync_mode_and_flush_succeed() {
        let dir = TempDir::new("sync");
        let mut region = RegionFile::open(&dir.path().join("r.0.0.mca"), true, true)
            .unwrap()
            .unwrap();
        region
            .write(ChunkPos::new(0, 0), 3, ChunkBody::Inline(&[1, 2, 3]))
            .unwrap();
        region.sync_all().unwrap();
    }
}
