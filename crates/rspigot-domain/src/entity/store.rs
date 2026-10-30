use std::collections::BTreeMap;

use rustc_hash::FxHashMap;
use uuid::Uuid;

use super::Entity;
use crate::{ChunkPos, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("an entity with uuid {0} already exists")]
pub struct DuplicateUuid(pub Uuid);

#[derive(Debug, Default)]
pub struct Entities {
    by_id: BTreeMap<EntityId, Entity>,
    by_uuid: FxHashMap<Uuid, EntityId>,
    by_chunk: FxHashMap<ChunkPos, Vec<EntityId>>,
    chunk_of: FxHashMap<EntityId, ChunkPos>,
}

impl Entities {
    pub fn insert(&mut self, entity: Entity) -> Result<EntityId, Box<(DuplicateUuid, Entity)>> {
        let uuid = entity.base.uuid;
        if self.by_uuid.contains_key(&uuid) {
            return Err(Box::new((DuplicateUuid(uuid), entity)));
        }
        let id = entity.base.id;
        let chunk = entity.base.chunk();
        self.by_uuid.insert(uuid, id);
        self.by_chunk.entry(chunk).or_default().push(id);
        self.chunk_of.insert(id, chunk);
        self.by_id.insert(id, entity);
        Ok(id)
    }

    pub fn remove(&mut self, id: EntityId) -> Option<Entity> {
        let entity = self.by_id.remove(&id)?;
        self.by_uuid.remove(&entity.base.uuid);
        if let Some(chunk) = self.chunk_of.remove(&id) {
            self.unindex(chunk, id);
        }
        Some(entity)
    }

    fn unindex(&mut self, chunk: ChunkPos, id: EntityId) {
        if let Some(ids) = self.by_chunk.get_mut(&chunk) {
            ids.retain(|candidate| *candidate != id);
            if ids.is_empty() {
                self.by_chunk.remove(&chunk);
            }
        }
    }

    pub fn reindex(&mut self, id: EntityId) -> Option<(ChunkPos, ChunkPos)> {
        let current = self.by_id.get(&id)?.base.chunk();
        let previous = *self.chunk_of.get(&id)?;
        if previous == current {
            return None;
        }
        self.unindex(previous, id);
        self.by_chunk.entry(current).or_default().push(id);
        self.chunk_of.insert(id, current);
        Some((previous, current))
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.by_id.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.by_id.get_mut(&id)
    }

    pub fn by_uuid(&self, uuid: Uuid) -> Option<EntityId> {
        self.by_uuid.get(&uuid).copied()
    }

    pub fn in_chunk(&self, chunk: ChunkPos) -> &[EntityId] {
        self.by_chunk.get(&chunk).map_or(&[], Vec::as_slice)
    }

    pub fn chunks(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.by_chunk.keys().copied()
    }

    pub fn ids(&self) -> Vec<EntityId> {
        self.by_id.keys().copied().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.by_id.values()
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn count_where(&self, predicate: impl Fn(&Entity) -> bool) -> usize {
        self.by_id.values().filter(|e| predicate(e)).count()
    }
}

#[cfg(test)]
mod tests {
    use rspigot_entities::EntityKind;

    use super::*;
    use crate::Vec3;
    use crate::entity::{EntityBase, EntityState};

    fn marker(id: i32, uuid: u128, x: f64) -> Entity {
        let mut base = EntityBase::new(EntityId(id), Uuid::from_u128(uuid), EntityKind::Marker);
        base.position = Vec3::new(x, 0.0, 0.0);
        Entity::new(base, EntityState::Inert(rspigot_nbt::Compound::new()))
    }

    #[test]
    fn chunk_index_follows_moves_and_uuids_stay_unique() {
        let mut store = Entities::default();
        store.insert(marker(1, 1, 1.0)).ok();
        assert!(store.insert(marker(2, 1, 1.0)).is_err());
        assert_eq!(store.in_chunk(ChunkPos::new(0, 0)), &[EntityId(1)]);
        if let Some(entity) = store.get_mut(EntityId(1)) {
            entity.base.position.x = 17.0;
        }
        assert_eq!(
            store.reindex(EntityId(1)),
            Some((ChunkPos::new(0, 0), ChunkPos::new(1, 0)))
        );
        assert!(store.in_chunk(ChunkPos::new(0, 0)).is_empty());
        assert_eq!(store.in_chunk(ChunkPos::new(1, 0)), &[EntityId(1)]);
        assert!(store.remove(EntityId(1)).is_some());
        assert!(store.by_uuid(Uuid::from_u128(1)).is_none());
        assert!(store.is_empty());
    }
}
