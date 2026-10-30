use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::time::{Duration, Instant};

use rspigot_core::events::{ClientEvent, Inbound};
use rspigot_core::ports::SessionVerifier;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_protocol::common::KeepAlive;
use rspigot_protocol::ids;
use rspigot_protocol::play::ServerboundPlay;
use rspigot_protocol::split_packet_id;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use crate::connection::{ConnectionError, PacketReader, PacketWriter, Session};
use crate::decode::{Decoded, decode};
use crate::encode::Encoder;
use crate::link::{NetLink, Outgoing};
use crate::listener::Shared;

const MAX_BATCH_BYTES: usize = 256 * 1024;

struct KeepAliveState {
    pending: AtomicI64,
    answered_at: parking_lot::Mutex<Instant>,
}

pub async fn run<V: SessionVerifier>(
    shared: Arc<Shared<V>>,
    session: Session,
    profile: GameProfile,
    settings: ClientSettings,
) -> Result<(), ConnectionError> {
    let (sender, receiver) = unbounded_channel();
    let connected = Arc::new(AtomicBool::new(true));
    let link = NetLink::new(sender, Arc::clone(&connected));
    let reader_link = link.clone();
    let player = profile.id;
    let name = profile.name.clone();
    if shared
        .game
        .send(Inbound::Join {
            profile,
            settings,
            link,
        })
        .is_err()
    {
        return Err(ConnectionError::Rejected("game loop stopped".to_owned()));
    }
    let Session {
        reader,
        writer,
        addr,
    } = session;
    let keep_alive = Arc::new(KeepAliveState {
        pending: AtomicI64::new(0),
        answered_at: parking_lot::Mutex::new(Instant::now()),
    });
    let read_task = tokio::spawn(read_loop(
        Arc::clone(&shared),
        reader,
        player,
        Arc::clone(&keep_alive),
        reader_link,
    ));
    let result = write_loop(&shared, writer, receiver, &keep_alive).await;
    connected.store(false, Ordering::Release);
    read_task.abort();
    let _ = shared.game.send(Inbound::Leave { player });
    match &result {
        Ok(reason) => tracing::info!("{name} ({addr}) lost connection: {reason}"),
        Err(error) => tracing::info!("{name} ({addr}) lost connection: {error}"),
    }
    result.map(|_| ())
}

async fn read_loop<V: SessionVerifier>(
    shared: Arc<Shared<V>>,
    mut reader: PacketReader,
    player: PlayerId,
    keep_alive: Arc<KeepAliveState>,
    link: NetLink,
) {
    loop {
        let packet = match reader.next().await {
            Ok(packet) => packet,
            Err(error) => {
                tracing::debug!(%error, "read loop ended");
                return;
            }
        };
        let Ok((id, body)) = split_packet_id(&packet) else {
            return;
        };
        let parsed = match ServerboundPlay::decode(id, body) {
            Ok(parsed) => parsed,
            Err(error) => {
                let name = ids::packet_name(ids::State::Play, ids::Direction::Serverbound, id)
                    .unwrap_or("unknown");
                tracing::debug!(%error, packet = name, "dropping undecodable packet");
                continue;
            }
        };
        match decode(parsed) {
            Decoded::Input(input) => {
                if shared.game.send(Inbound::Input { player, input }).is_err() {
                    return;
                }
            }
            Decoded::KeepAlive(id) => {
                if keep_alive
                    .pending
                    .compare_exchange(id, 0, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    *keep_alive.answered_at.lock() = Instant::now();
                }
            }
            Decoded::Ping(payload) => link.pong(payload),
            Decoded::Ignored => {}
        }
    }
}

async fn write_loop<V: SessionVerifier>(
    shared: &Shared<V>,
    mut writer: PacketWriter,
    mut receiver: UnboundedReceiver<Outgoing>,
    keep_alive: &KeepAliveState,
) -> Result<String, ConnectionError> {
    let mut encoder = Encoder::new(shared.config.online_mode);
    let mut ticker = tokio::time::interval(shared.config.keep_alive_interval);
    ticker.tick().await;
    let timeout = shared.config.keep_alive_timeout;
    loop {
        tokio::select! {
            message = receiver.recv() => {
                let Some(message) = message else {
                    return Ok("Disconnected".to_owned());
                };
                if let Some(reason) = handle(&mut encoder, shared, &mut writer, message)? {
                    return finish(&mut writer, &reason).await;
                }
                while writer.pending.len() < MAX_BATCH_BYTES {
                    let Ok(message) = receiver.try_recv() else {
                        break;
                    };
                    if let Some(reason) = handle(&mut encoder, shared, &mut writer, message)? {
                        return finish(&mut writer, &reason).await;
                    }
                }
                writer.flush().await?;
            }
            _ = ticker.tick() => {
                if keep_alive.pending.load(Ordering::Acquire) != 0 && keep_alive.answered_at.lock().elapsed() > timeout {
                    return finish(&mut writer, &Component::translatable("disconnect.timeout", Vec::new())).await;
                }
                if keep_alive.pending.load(Ordering::Acquire) == 0 {
                    let id = millis_since_epoch().max(1);
                    keep_alive.pending.store(id, Ordering::Release);
                    writer.send(ids::play::clientbound::KEEP_ALIVE, &KeepAlive { id }).await?;
                }
            }
        }
    }
}

fn millis_since_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |d| d.as_millis() as i64)
}

fn handle<V: SessionVerifier>(
    encoder: &mut Encoder,
    shared: &Shared<V>,
    writer: &mut PacketWriter,
    message: Outgoing,
) -> Result<Option<Component>, ConnectionError> {
    match message {
        Outgoing::Event(ClientEvent::Disconnect(reason)) | Outgoing::Disconnect(reason) => {
            Ok(Some(reason))
        }
        Outgoing::Event(event) => {
            encoder.encode(
                &event,
                writer,
                &shared.chunks,
                shared.config.compression_threshold,
                shared.config.compression_level,
            )?;
            Ok(None)
        }
        Outgoing::Pong(payload) => {
            writer.queue(
                ids::play::clientbound::PONG_RESPONSE,
                &rspigot_protocol::status::PongResponse { time: payload },
            )?;
            Ok(None)
        }
    }
}

async fn finish(writer: &mut PacketWriter, reason: &Component) -> Result<String, ConnectionError> {
    let packet = rspigot_protocol::common::Disconnect {
        reason: rspigot_protocol::types::Text(crate::text::to_nbt(reason)),
    };
    writer.queue(ids::play::clientbound::DISCONNECT, &packet)?;
    writer.flush().await?;
    tokio::time::sleep(Duration::from_millis(50)).await;
    writer.shutdown().await;
    Ok(rspigot_core::console::console_text(reason))
}
