use std::io;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::Sender;
use parking_lot::Mutex;
use rspigot_core::events::Inbound;
use rspigot_core::ports::SessionVerifier;
use rspigot_core::status::StatusBoard;
use rspigot_protocol::crypto::ServerKey;
use rustc_hash::FxHashMap;
use tokio::net::TcpListener;
use tokio::sync::watch;
use uuid::Uuid;

use crate::NetworkConfig;
use crate::cache::ChunkPacketCache;
use crate::connection;
use crate::link::NetLink;

pub struct Shared<V> {
    pub config: NetworkConfig,
    pub verifier: Arc<V>,
    pub status: StatusBoard,
    pub game: Sender<Inbound<NetLink>>,
    pub key: ServerKey,
    pub session_id: Uuid,
    pub chunks: ChunkPacketCache,
    throttle: Mutex<FxHashMap<IpAddr, Instant>>,
}

impl<V> Shared<V> {
    pub fn throttled(&self, ip: IpAddr) -> bool {
        let window = self.config.connection_throttle;
        if window.is_zero() || ip.is_loopback() {
            return false;
        }
        let now = Instant::now();
        let mut throttle = self.throttle.lock();
        throttle.retain(|_, at| now.duration_since(*at) < window);
        let blocked = throttle.contains_key(&ip);
        throttle.insert(ip, now);
        blocked
    }
}

pub async fn run<V: SessionVerifier>(
    config: NetworkConfig,
    verifier: Arc<V>,
    status: StatusBoard,
    game: Sender<Inbound<NetLink>>,
    mut shutdown: watch::Receiver<bool>,
) -> io::Result<()> {
    let key = ServerKey::generate().map_err(|e| io::Error::other(e.to_string()))?;
    let listener = TcpListener::bind(config.bind).await?;
    tracing::info!("Starting Minecraft server on {}", config.bind);
    if !config.online_mode {
        tracing::warn!("**** SERVER IS RUNNING IN OFFLINE/INSECURE MODE!");
        tracing::warn!("The server will make no attempt to authenticate usernames. Beware.");
    }
    let shared = Arc::new(Shared {
        chunks: ChunkPacketCache::new(config.chunk_cache_bytes),
        config,
        verifier,
        status,
        game,
        key,
        session_id: Uuid::new_v4(),
        throttle: Mutex::new(FxHashMap::default()),
    });
    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, addr)) => {
                    let _ = stream.set_nodelay(true);
                    let shared = Arc::clone(&shared);
                    tokio::spawn(async move {
                        if let Err(error) = connection::handle(shared, stream, addr).await {
                            tracing::debug!(%addr, %error, "connection closed");
                        }
                    });
                }
                Err(error) => tracing::warn!(%error, "accept failed"),
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    return Ok(());
                }
            }
        }
    }
}
