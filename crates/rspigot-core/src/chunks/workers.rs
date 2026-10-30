use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, unbounded};
use rspigot_domain::ChunkPos;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::world::Dimension;

use super::queue::{JobQueue, Priority};
use crate::lighting;
use crate::ports::{ChunkGenerator, ChunkStore};

pub enum ChunkJob {
    Load {
        dimension: Arc<Dimension>,
        pos: ChunkPos,
    },
    Save {
        dimension: Arc<Dimension>,
        chunk: Arc<ChunkColumn>,
    },
    Flush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkSource {
    Disk,
    Generated,
}

pub enum ChunkResult {
    Loaded {
        dimension: Arc<Dimension>,
        pos: ChunkPos,
        chunk: Box<ChunkColumn>,
        source: ChunkSource,
        elapsed: Duration,
    },
    Saved {
        pos: ChunkPos,
    },
    Failed {
        pos: ChunkPos,
        error: String,
    },
    Flushed,
}

pub struct ChunkWorkers {
    queue: Arc<JobQueue<ChunkJob>>,
    results: Receiver<ChunkResult>,
    handles: Vec<JoinHandle<()>>,
}

impl ChunkWorkers {
    pub fn spawn<S: ChunkStore, G: ChunkGenerator>(
        threads: usize,
        store: Arc<S>,
        generator: Arc<G>,
    ) -> Self {
        let queue = Arc::new(JobQueue::default());
        let (sender, results) = unbounded();
        let handles = (0..threads.max(1))
            .map(|index| {
                let queue = Arc::clone(&queue);
                let store = Arc::clone(&store);
                let generator = Arc::clone(&generator);
                let sender = sender.clone();
                std::thread::Builder::new()
                    .name(format!("chunk-worker-{index}"))
                    .spawn(move || run_worker(&queue, &*store, &*generator, &sender))
                    .ok()
            })
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default();
        Self {
            queue,
            results,
            handles,
        }
    }

    pub fn submit(&self, priority: Priority, job: ChunkJob) -> Arc<AtomicBool> {
        self.queue.push(priority, job)
    }

    pub fn results(&self) -> &Receiver<ChunkResult> {
        &self.results
    }

    pub fn backlog(&self) -> usize {
        self.queue.len()
    }

    pub fn thread_count(&self) -> usize {
        self.handles.len()
    }

    pub fn shutdown(self) {
        self.queue.close();
        for handle in self.handles {
            let _ = handle.join();
        }
    }
}

fn run_worker<S: ChunkStore, G: ChunkGenerator>(
    queue: &JobQueue<ChunkJob>,
    store: &S,
    generator: &G,
    results: &Sender<ChunkResult>,
) {
    while let Some(job) = queue.pop_blocking() {
        let result = execute(job, store, generator);
        if results.send(result).is_err() {
            return;
        }
    }
}

fn execute<S: ChunkStore, G: ChunkGenerator>(
    job: ChunkJob,
    store: &S,
    generator: &G,
) -> ChunkResult {
    match job {
        ChunkJob::Load { dimension, pos } => load(dimension, pos, store, generator),
        ChunkJob::Save { dimension, chunk } => match store.save(&dimension.key, &chunk) {
            Ok(()) => ChunkResult::Saved { pos: chunk.pos() },
            Err(error) => ChunkResult::Failed {
                pos: chunk.pos(),
                error: error.to_string(),
            },
        },
        ChunkJob::Flush => {
            if let Err(error) = store.flush() {
                tracing::error!(%error, "chunk store flush failed");
            }
            ChunkResult::Flushed
        }
    }
}

fn load<S: ChunkStore, G: ChunkGenerator>(
    dimension: Arc<Dimension>,
    pos: ChunkPos,
    store: &S,
    generator: &G,
) -> ChunkResult {
    let started = Instant::now();
    let (mut chunk, source) = match store.load(&dimension.key, pos) {
        Ok(Some(chunk)) => (chunk, ChunkSource::Disk),
        Ok(None) => (generator.generate(&dimension, pos), ChunkSource::Generated),
        Err(error) => {
            tracing::warn!(?pos, %error, "chunk load failed, regenerating");
            (generator.generate(&dimension, pos), ChunkSource::Generated)
        }
    };
    if !lighting::has_light(&chunk) {
        lighting::compute(&mut chunk, dimension.has_skylight);
    }
    ChunkResult::Loaded {
        dimension,
        pos,
        chunk: Box::new(chunk),
        source,
        elapsed: started.elapsed(),
    }
}
