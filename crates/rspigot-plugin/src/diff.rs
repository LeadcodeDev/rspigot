use rspigot_plugin_protocol::v1;

#[derive(Debug, PartialEq)]
pub(crate) struct Diff<T> {
    pub(crate) removed: Vec<u32>,
    pub(crate) replaced: Vec<(u32, T)>,
    pub(crate) added: Vec<T>,
}

impl<T> Diff<T> {
    fn changed(self) -> Option<Self> {
        let unchanged =
            self.removed.is_empty() && self.replaced.is_empty() && self.added.is_empty();
        (!unchanged).then_some(self)
    }
}

pub(crate) fn list<T: PartialEq>(snapshot: &[T], values: Vec<T>) -> Option<Diff<T>> {
    list_by(snapshot, values, |_, _, _, _| false)
}

pub(crate) fn list_by<T: PartialEq>(
    snapshot: &[T],
    values: Vec<T>,
    kin: impl Fn(usize, &T, usize, &T) -> bool,
) -> Option<Diff<T>> {
    let mut matched = vec![false; snapshot.len()];
    let mut next = 0;
    let mut unmatched = Vec::new();
    for (position, value) in values.into_iter().enumerate() {
        let found = (next..snapshot.len())
            .chain(0..next)
            .find(|&index| !matched[index] && snapshot[index] == value);
        match found {
            Some(index) => {
                matched[index] = true;
                next = index + 1;
            }
            None => unmatched.push((position, value)),
        }
    }
    let mut replaced = Vec::new();
    let mut added = Vec::new();
    for (position, value) in unmatched {
        let kindred = (0..snapshot.len())
            .find(|&index| !matched[index] && kin(index, &snapshot[index], position, &value));
        match kindred.and_then(|index| Some((index, u32::try_from(index).ok()?))) {
            Some((index, wire)) => {
                matched[index] = true;
                replaced.push((wire, value));
            }
            None => added.push(value),
        }
    }
    Diff {
        removed: unmatched_indices(&matched),
        replaced,
        added,
    }
    .changed()
}

pub(crate) fn slots<T: PartialEq>(snapshot: &[T], values: Vec<T>) -> Option<Diff<T>> {
    Diff {
        removed: Vec::new(),
        replaced: snapshot
            .iter()
            .zip(values)
            .enumerate()
            .filter(|(_, (before, after))| *before != after)
            .filter_map(|(index, (_, after))| Some((u32::try_from(index).ok()?, after)))
            .collect(),
        added: Vec::new(),
    }
    .changed()
}

pub(crate) fn same_item(at: usize, before: &v1::Item, to: usize, after: &v1::Item) -> bool {
    at == to && before.key == after.key
}

pub(crate) fn same_entity(
    _: usize,
    before: &v1::DroppedItem,
    _: usize,
    after: &v1::DroppedItem,
) -> bool {
    let entity = |dropped: &v1::DroppedItem| dropped.entity.as_ref().map(|entity| entity.entity_id);
    entity(before).is_some() && entity(before) == entity(after)
}

fn unmatched_indices(matched: &[bool]) -> Vec<u32> {
    matched
        .iter()
        .enumerate()
        .filter(|(_, matched)| !**matched)
        .filter_map(|(index, _)| u32::try_from(index).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(values: &[&'static str]) -> Vec<&'static str> {
        values.to_vec()
    }

    fn changed<T>(removed: &[u32], replaced: Vec<(u32, T)>, added: Vec<T>) -> Option<Diff<T>> {
        Some(Diff {
            removed: removed.to_vec(),
            replaced,
            added,
        })
    }

    const SNAPSHOT: [&str; 4] = ["alice", "bob", "carol", "dave"];

    #[test]
    fn an_unchanged_list_has_no_diff() {
        assert_eq!(list(&SNAPSHOT, names(&SNAPSHOT)), None);
        assert_eq!(list::<&str>(&[], Vec::new()), None);
    }

    #[test]
    fn a_dropped_element_is_removed_by_its_index_in_the_snapshot() {
        assert_eq!(
            list(&SNAPSHOT, names(&["alice", "carol"])),
            changed(&[1, 3], Vec::new(), Vec::new())
        );
        assert_eq!(
            list(&SNAPSHOT, Vec::new()),
            changed(&[0, 1, 2, 3], Vec::new(), Vec::new())
        );
    }

    #[test]
    fn a_new_element_is_added_in_the_order_of_the_list() {
        assert_eq!(
            list(
                &SNAPSHOT,
                names(&["erin", "alice", "bob", "carol", "dave", "frank"])
            ),
            changed(&[], Vec::new(), names(&["erin", "frank"]))
        );
    }

    #[test]
    fn a_reordered_list_carries_nothing() {
        assert_eq!(
            list(&SNAPSHOT, names(&["dave", "carol", "bob", "alice"])),
            None
        );
    }

    #[test]
    fn an_element_changed_without_an_identity_is_a_removal_and_an_addition() {
        assert_eq!(
            list(&SNAPSHOT, names(&["alice", "bea", "carol", "dave"])),
            changed(&[1], Vec::new(), names(&["bea"]))
        );
    }

    #[test]
    fn duplicates_are_counted() {
        let twice = ["alice", "bob", "alice"];
        assert_eq!(
            list(&twice, names(&["bob", "alice"])),
            changed(&[0], Vec::new(), Vec::new())
        );
        assert_eq!(
            list(&twice, names(&["alice", "bob", "alice", "alice"])),
            changed(&[], Vec::new(), names(&["alice"]))
        );
    }

    #[test]
    fn an_element_pushed_again_is_an_addition() {
        let mut again = names(&SNAPSHOT);
        again.push("bob");
        assert_eq!(
            list(&SNAPSHOT, again),
            changed(&[], Vec::new(), names(&["bob"]))
        );
    }

    fn stack(key: &str, count: i32) -> v1::Item {
        v1::Item {
            key: key.to_owned(),
            count,
            ..v1::Item::default()
        }
    }

    #[test]
    fn an_item_changed_at_its_index_is_replaced() {
        let drops = [stack("minecraft:diamond", 5), stack("minecraft:stick", 2)];
        assert_eq!(
            list_by(
                &drops,
                vec![stack("minecraft:diamond", 10), stack("minecraft:stick", 2)],
                same_item
            ),
            changed(&[], vec![(0, stack("minecraft:diamond", 10))], Vec::new())
        );
        assert_eq!(
            list_by(
                &drops,
                vec![stack("minecraft:stick", 2), stack("minecraft:diamond", 10)],
                same_item
            ),
            changed(&[0], Vec::new(), vec![stack("minecraft:diamond", 10)])
        );
        assert_eq!(
            list_by(
                &drops,
                vec![stack("minecraft:apple", 5), stack("minecraft:stick", 2)],
                same_item
            ),
            changed(&[0], Vec::new(), vec![stack("minecraft:apple", 5)])
        );
    }

    fn dropped(entity_id: i32, key: &str, count: i32) -> v1::DroppedItem {
        v1::DroppedItem {
            entity: Some(v1::EntitySnapshot {
                entity_id,
                ..v1::EntitySnapshot::default()
            }),
            item: Some(stack(key, count)),
        }
    }

    #[test]
    fn a_dropped_item_with_a_new_stack_is_replaced_wherever_it_moved() {
        let items = [
            dropped(7, "minecraft:dirt", 1),
            dropped(8, "minecraft:dirt", 1),
        ];
        assert_eq!(
            list_by(
                &items,
                vec![
                    dropped(8, "minecraft:diamond", 2),
                    dropped(7, "minecraft:dirt", 1)
                ],
                same_entity
            ),
            changed(
                &[],
                vec![(1, dropped(8, "minecraft:diamond", 2))],
                Vec::new()
            )
        );
        assert_eq!(
            list_by(&items, vec![dropped(9, "minecraft:dirt", 1)], same_entity),
            changed(&[0, 1], Vec::new(), vec![dropped(9, "minecraft:dirt", 1)])
        );
    }

    #[test]
    fn a_slot_is_replaced_in_place() {
        let lines = ["a", "b", "c", "d"];
        assert_eq!(slots(&lines, names(&lines)), None);
        assert_eq!(
            slots(&lines, names(&["a", "x", "c", "y"])),
            changed(&[], vec![(1, "x"), (3, "y")], Vec::new())
        );
        assert_eq!(
            slots(&lines, names(&["z"])),
            changed(&[], vec![(0, "z")], Vec::new())
        );
        assert_eq!(slots(&lines, names(&["a", "b", "c", "d", "e"])), None);
    }
}
