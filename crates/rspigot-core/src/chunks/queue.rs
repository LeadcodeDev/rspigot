use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use parking_lot::{Condvar, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u32);

impl Priority {
    pub const URGENT: Self = Self(0);
    pub const BACKGROUND: Self = Self(u32::MAX);
}

struct Entry<T> {
    priority: Priority,
    sequence: u64,
    cancelled: Arc<AtomicBool>,
    job: T,
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl<T> Eq for Entry<T> {}

impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

struct State<T> {
    heap: BinaryHeap<Entry<T>>,
    sequence: u64,
    closed: bool,
}

pub struct JobQueue<T> {
    state: Mutex<State<T>>,
    available: Condvar,
}

impl<T> Default for JobQueue<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                heap: BinaryHeap::new(),
                sequence: 0,
                closed: false,
            }),
            available: Condvar::new(),
        }
    }
}

impl<T> JobQueue<T> {
    pub fn push(&self, priority: Priority, job: T) -> Arc<AtomicBool> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let mut state = self.state.lock();
        state.sequence += 1;
        let sequence = state.sequence;
        state.heap.push(Entry {
            priority,
            sequence,
            cancelled: Arc::clone(&cancelled),
            job,
        });
        drop(state);
        self.available.notify_one();
        cancelled
    }

    pub fn pop_blocking(&self) -> Option<T> {
        let mut state = self.state.lock();
        loop {
            while let Some(entry) = state.heap.pop() {
                if !entry.cancelled.load(AtomicOrdering::Acquire) {
                    return Some(entry.job);
                }
            }
            if state.closed {
                return None;
            }
            self.available.wait(&mut state);
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        let mut state = self.state.lock();
        while let Some(entry) = state.heap.pop() {
            if !entry.cancelled.load(AtomicOrdering::Acquire) {
                return Some(entry.job);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.state.lock().heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn close(&self) {
        self.state.lock().closed = true;
        self.available.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pops_lowest_priority_value_first_and_fifo_on_ties() {
        let queue = JobQueue::default();
        queue.push(Priority(5), "far");
        queue.push(Priority(1), "near-a");
        queue.push(Priority(1), "near-b");
        queue.push(Priority::URGENT, "urgent");
        assert_eq!(queue.try_pop(), Some("urgent"));
        assert_eq!(queue.try_pop(), Some("near-a"));
        assert_eq!(queue.try_pop(), Some("near-b"));
        assert_eq!(queue.try_pop(), Some("far"));
        assert_eq!(queue.try_pop(), None);
    }

    #[test]
    fn cancelled_jobs_are_skipped() {
        let queue = JobQueue::default();
        let token = queue.push(Priority(1), 1);
        queue.push(Priority(2), 2);
        token.store(true, AtomicOrdering::Release);
        assert_eq!(queue.try_pop(), Some(2));
    }

    #[test]
    fn close_releases_blocked_workers() {
        let queue = Arc::new(JobQueue::<u8>::default());
        let worker = {
            let queue = Arc::clone(&queue);
            std::thread::spawn(move || queue.pop_blocking())
        };
        queue.close();
        assert_eq!(worker.join().ok().flatten(), None);
    }
}
