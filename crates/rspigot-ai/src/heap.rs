use crate::walk::Node;

#[derive(Debug, Default)]
pub(crate) struct BinaryHeap {
    heap: Vec<usize>,
}

impl BinaryHeap {
    pub(crate) fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub(crate) fn insert(&mut self, nodes: &mut [Node], node: usize) {
        let index = self.heap.len();
        self.heap.push(node);
        nodes[node].heap_index = Some(index);
        self.up_heap(nodes, index);
    }

    pub(crate) fn pop(&mut self, nodes: &mut [Node]) -> Option<usize> {
        let last = self.heap.pop()?;
        let top = match self.heap.first_mut() {
            Some(first) => std::mem::replace(first, last),
            None => last,
        };
        if !self.heap.is_empty() {
            self.down_heap(nodes, 0);
        }
        nodes[top].heap_index = None;
        Some(top)
    }

    pub(crate) fn change_cost(&mut self, nodes: &mut [Node], node: usize, cost: f32) {
        let old = nodes[node].f;
        nodes[node].f = cost;
        let Some(index) = nodes[node].heap_index else {
            return;
        };
        if cost < old {
            self.up_heap(nodes, index);
        } else {
            self.down_heap(nodes, index);
        }
    }

    fn place(&mut self, nodes: &mut [Node], index: usize, node: usize) {
        self.heap[index] = node;
        nodes[node].heap_index = Some(index);
    }

    fn up_heap(&mut self, nodes: &mut [Node], mut index: usize) {
        let node = self.heap[index];
        let cost = nodes[node].f;
        while index > 0 {
            let parent = (index - 1) >> 1;
            let above = self.heap[parent];
            if !lower(cost, nodes[above].f) {
                break;
            }
            self.place(nodes, index, above);
            index = parent;
        }
        self.place(nodes, index, node);
    }

    fn down_heap(&mut self, nodes: &mut [Node], mut index: usize) {
        let node = self.heap[index];
        let cost = nodes[node].f;
        loop {
            let left = 1 + (index << 1);
            let right = left + 1;
            let Some(&left_node) = self.heap.get(left) else {
                break;
            };
            let left_cost = nodes[left_node].f;
            let (right_node, right_cost) = match self.heap.get(right) {
                Some(&right_node) => (Some(right_node), nodes[right_node].f),
                None => (None, f32::INFINITY),
            };
            if lower(left_cost, right_cost) {
                if !lower(left_cost, cost) {
                    break;
                }
                self.place(nodes, index, left_node);
                index = left;
            } else {
                let Some(right_node) = right_node else {
                    break;
                };
                if !lower(right_cost, cost) {
                    break;
                }
                self.place(nodes, index, right_node);
                index = right;
            }
        }
        self.place(nodes, index, node);
    }
}

fn lower(left: f32, right: f32) -> bool {
    left < right
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspigot_domain::BlockPos;

    fn nodes(costs: &[f32]) -> Vec<Node> {
        costs
            .iter()
            .enumerate()
            .map(|(index, cost)| {
                let mut node = Node::new(BlockPos::new(index as i32, 0, 0));
                node.f = *cost;
                node
            })
            .collect()
    }

    #[test]
    fn pops_in_cost_order() {
        let mut arena = nodes(&[5.0, 1.0, 3.0, 4.0, 2.0, 0.5]);
        let mut heap = BinaryHeap::default();
        for index in 0..arena.len() {
            heap.insert(&mut arena, index);
        }
        heap.change_cost(&mut arena, 0, 0.1);
        let mut order = Vec::new();
        while let Some(node) = heap.pop(&mut arena) {
            assert!(arena[node].heap_index.is_none());
            order.push(node);
        }
        assert_eq!(order, [0, 5, 1, 4, 2, 3]);
        assert!(heap.is_empty());
    }

    #[test]
    fn heap_indices_track_positions() {
        let mut arena = nodes(&[3.0, 2.0, 1.0]);
        let mut heap = BinaryHeap::default();
        for index in 0..arena.len() {
            heap.insert(&mut arena, index);
        }
        for (position, node) in heap.heap.iter().enumerate() {
            assert_eq!(arena[*node].heap_index, Some(position));
        }
    }
}
