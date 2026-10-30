use rspigot_domain::{BlockPos, Vec3};

use crate::PathType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathNode {
    pub pos: BlockPos,
    pub path_type: PathType,
    pub cost_malus: f32,
    pub walked_distance: f32,
}

impl PathNode {
    pub fn new(pos: BlockPos, path_type: PathType) -> Self {
        Self {
            pos,
            path_type,
            cost_malus: 0.0,
            walked_distance: 0.0,
        }
    }

    pub fn moved(&self, pos: BlockPos) -> Self {
        Self { pos, ..*self }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    nodes: Vec<PathNode>,
    next_node_index: usize,
    target: BlockPos,
    dist_to_target: f32,
    reached: bool,
    visited_nodes: usize,
}

impl Path {
    pub fn new(nodes: Vec<PathNode>, target: BlockPos, reached: bool) -> Self {
        let dist_to_target = nodes
            .last()
            .map_or(f32::MAX, |node| manhattan(node.pos, target));
        Self {
            nodes,
            next_node_index: 0,
            target,
            dist_to_target,
            reached,
            visited_nodes: 0,
        }
    }

    pub(crate) fn with_visited(mut self, visited_nodes: usize) -> Self {
        self.visited_nodes = visited_nodes;
        self
    }

    pub fn advance(&mut self) {
        self.next_node_index += 1;
    }

    pub fn not_started(&self) -> bool {
        self.next_node_index == 0
    }

    pub fn is_done(&self) -> bool {
        self.next_node_index >= self.nodes.len()
    }

    pub fn end_node(&self) -> Option<&PathNode> {
        self.nodes.last()
    }

    pub fn node(&self, index: usize) -> &PathNode {
        &self.nodes[index]
    }

    pub fn nodes(&self) -> &[PathNode] {
        &self.nodes
    }

    pub fn truncate_nodes(&mut self, length: usize) {
        self.nodes.truncate(length);
    }

    pub fn replace_node(&mut self, index: usize, node: PathNode) {
        self.nodes[index] = node;
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn next_node_index(&self) -> usize {
        self.next_node_index
    }

    pub fn set_next_node_index(&mut self, index: usize) {
        self.next_node_index = index;
    }

    pub fn entity_pos_at_node(&self, width: f32, index: usize) -> Vec3 {
        let pos = self.nodes[index].pos;
        let offset = f64::from((width + 1.0) as i32) * 0.5;
        Vec3::new(
            f64::from(pos.x) + offset,
            f64::from(pos.y),
            f64::from(pos.z) + offset,
        )
    }

    pub fn next_entity_pos(&self, width: f32) -> Vec3 {
        self.entity_pos_at_node(width, self.next_node_index)
    }

    pub fn node_pos(&self, index: usize) -> BlockPos {
        self.nodes[index].pos
    }

    pub fn next_node_pos(&self) -> BlockPos {
        self.nodes[self.next_node_index].pos
    }

    pub fn next_node(&self) -> &PathNode {
        &self.nodes[self.next_node_index]
    }

    pub fn previous_node(&self) -> Option<&PathNode> {
        self.next_node_index
            .checked_sub(1)
            .and_then(|index| self.nodes.get(index))
    }

    pub fn same_as(&self, other: Option<&Path>) -> bool {
        other.is_some_and(|other| {
            self.nodes.len() == other.nodes.len()
                && self
                    .nodes
                    .iter()
                    .zip(&other.nodes)
                    .all(|(left, right)| left.pos == right.pos)
        })
    }

    pub fn can_reach(&self) -> bool {
        self.reached
    }

    pub fn target(&self) -> BlockPos {
        self.target
    }

    pub fn dist_to_target(&self) -> f32 {
        self.dist_to_target
    }

    pub fn visited_nodes(&self) -> usize {
        self.visited_nodes
    }
}

pub(crate) fn manhattan(from: BlockPos, to: BlockPos) -> f32 {
    (to.x - from.x).abs() as f32 + (to.y - from.y).abs() as f32 + (to.z - from.z).abs() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Path {
        let nodes = (0..3)
            .map(|x| PathNode::new(BlockPos::new(x, 64, 0), PathType::Walkable))
            .collect();
        Path::new(nodes, BlockPos::new(5, 64, 1), false)
    }

    #[test]
    fn progress_and_lookups() {
        let mut path = line();
        assert!(path.not_started());
        assert_eq!(path.dist_to_target(), 4.0);
        assert!(path.previous_node().is_none());
        path.advance();
        assert_eq!(path.previous_node().map(|node| node.pos.x), Some(0));
        assert_eq!(path.next_node_pos(), BlockPos::new(1, 64, 0));
        path.set_next_node_index(3);
        assert!(path.is_done());
        path.truncate_nodes(1);
        assert_eq!(path.node_count(), 1);
    }

    #[test]
    fn empty_path_is_far_and_done() {
        let path = Path::new(Vec::new(), BlockPos::new(0, 0, 0), false);
        assert_eq!(path.dist_to_target(), f32::MAX);
        assert!(path.is_done());
        assert!(path.end_node().is_none());
    }

    #[test]
    fn same_as_compares_positions() {
        let path = line();
        let mut other = line();
        assert!(path.same_as(Some(&other)));
        other.replace_node(1, other.node(1).moved(BlockPos::new(1, 65, 0)));
        assert!(!path.same_as(Some(&other)));
        assert!(!path.same_as(None));
    }
}
