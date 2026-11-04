pub(crate) type Diff<T> = (Vec<u32>, Vec<T>);

pub(crate) fn list<T: PartialEq>(snapshot: &[T], values: Vec<T>) -> Option<Diff<T>> {
    let mut kept = vec![false; snapshot.len()];
    let mut next = 0;
    let mut added = Vec::new();
    for value in values {
        let found = (next..snapshot.len())
            .chain(0..next)
            .find(|&index| !kept[index] && snapshot[index] == value);
        match found {
            Some(index) => {
                kept[index] = true;
                next = index + 1;
            }
            None => added.push(value),
        }
    }
    let removed: Vec<u32> = kept
        .iter()
        .enumerate()
        .filter(|(_, kept)| !**kept)
        .filter_map(|(index, _)| u32::try_from(index).ok())
        .collect();
    (!removed.is_empty() || !added.is_empty()).then_some((removed, added))
}

pub(crate) fn slots<T: PartialEq>(snapshot: &[T], values: Vec<T>) -> Option<Diff<T>> {
    let (removed, added): (Vec<u32>, Vec<T>) = snapshot
        .iter()
        .zip(values)
        .enumerate()
        .filter(|(_, (before, after))| *before != after)
        .filter_map(|(index, (_, after))| Some((u32::try_from(index).ok()?, after)))
        .unzip();
    (!removed.is_empty()).then_some((removed, added))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(values: &[&'static str]) -> Vec<&'static str> {
        values.to_vec()
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
            Some((vec![1, 3], Vec::new()))
        );
        assert_eq!(
            list(&SNAPSHOT, Vec::new()),
            Some((vec![0, 1, 2, 3], Vec::new()))
        );
    }

    #[test]
    fn a_new_element_is_added_in_the_order_of_the_list() {
        assert_eq!(
            list(
                &SNAPSHOT,
                names(&["erin", "alice", "bob", "carol", "dave", "frank"])
            ),
            Some((Vec::new(), names(&["erin", "frank"])))
        );
    }

    #[test]
    fn a_reordered_list_keeps_the_order_of_the_snapshot() {
        assert_eq!(
            list(&SNAPSHOT, names(&["dave", "carol", "bob", "alice"])),
            None
        );
    }

    #[test]
    fn a_replaced_element_is_a_removal_and_an_addition() {
        assert_eq!(
            list(&SNAPSHOT, names(&["alice", "bea", "carol", "dave"])),
            Some((vec![1], names(&["bea"])))
        );
    }

    #[test]
    fn duplicates_are_counted() {
        let twice = ["alice", "bob", "alice"];
        assert_eq!(
            list(&twice, names(&["bob", "alice"])),
            Some((vec![0], Vec::new()))
        );
        assert_eq!(
            list(&twice, names(&["alice", "bob", "alice", "alice"])),
            Some((Vec::new(), names(&["alice"])))
        );
        assert_eq!(
            list(&SNAPSHOT, names(&["alice", "bob", "bob", "carol", "dave"])),
            Some((Vec::new(), names(&["bob"])))
        );
    }

    #[test]
    fn a_slot_is_replaced_in_place() {
        let lines = ["a", "b", "c", "d"];
        assert_eq!(slots(&lines, names(&lines)), None);
        assert_eq!(
            slots(&lines, names(&["a", "x", "c", "y"])),
            Some((vec![1, 3], names(&["x", "y"])))
        );
        assert_eq!(slots(&lines, names(&["z"])), Some((vec![0], names(&["z"]))));
        assert_eq!(slots(&lines, names(&["a", "b", "c", "d", "e"])), None);
    }
}
