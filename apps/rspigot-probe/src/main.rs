use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

use anyhow::{Context, bail};
use rspigot_domain::player::PlayerId;
use rspigot_protocol::codec::{Decode, Encode, decode_exact};
use rspigot_protocol::common::{
    ChatVisibility, ClientInformation, KeepAlive, MainHand, ParticleStatus, Ping, Pong,
};
use rspigot_protocol::configuration::{
    ServerboundFinishConfiguration, ServerboundSelectKnownPacks,
};
use rspigot_protocol::frame::{Compression, FrameDecoder, FrameEncoder};
use rspigot_protocol::handshake::{ClientIntent, Intention};
use rspigot_protocol::login::{LoginAcknowledged, LoginCompression, ServerboundHello};
use rspigot_protocol::types::{KnownPack, Text};
use rspigot_protocol::{VarInt, ids, split_packet_id, write_packet_with_id};

struct Options {
    address: String,
    name: String,
    view_distance: i8,
    target_chunks: usize,
    walk_blocks: f64,
    speed: f64,
    timeout: Duration,
    rate: f32,
    command: Option<String>,
}

fn options() -> Options {
    let mut options = Options {
        address: "127.0.0.1:25565".to_owned(),
        name: "Probe".to_owned(),
        view_distance: 10,
        target_chunks: 0,
        walk_blocks: 0.0,
        speed: 1.0,
        timeout: Duration::from_secs(120),
        rate: 64.0,
        command: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().unwrap_or_default();
        match arg.as_str() {
            "--address" => options.address = value,
            "--name" => options.name = value,
            "--view" => options.view_distance = value.parse().unwrap_or(10),
            "--chunks" => options.target_chunks = value.parse().unwrap_or(0),
            "--walk" => options.walk_blocks = value.parse().unwrap_or(0.0),
            "--speed" => options.speed = value.parse().unwrap_or(1.0),
            "--timeout" => options.timeout = Duration::from_secs(value.parse().unwrap_or(120)),
            "--rate" => options.rate = value.parse().unwrap_or(64.0),
            "--command" => options.command = Some(value),
            _ => {}
        }
    }
    if options.target_chunks == 0 {
        let v = i32::from(options.view_distance);
        options.target_chunks = (-v - 1..=v + 1)
            .flat_map(|x| (-v - 1..=v + 1).map(move |z| (x, z)))
            .filter(|(x, z)| within_view(*x, *z, v))
            .count();
    }
    options
}

fn within_view(dx: i32, dz: i32, view: i32) -> bool {
    let a = i64::from((dx.abs() - 1).max(0));
    let b = i64::from((dz.abs() - 1).max(0));
    let far = (a.max(b) - 1).max(0);
    let near = a.min(b);
    near * near + far * far < i64::from(view) * i64::from(view)
}

struct Client {
    stream: TcpStream,
    encoder: FrameEncoder,
    decoder: FrameDecoder,
    buffer: Vec<u8>,
}

impl Client {
    fn send<P: Encode + ?Sized>(&mut self, id: i32, packet: &P) -> anyhow::Result<()> {
        let mut body = Vec::new();
        write_packet_with_id(id, packet, &mut body);
        let mut frame = Vec::new();
        self.encoder.encode_packet(&body, &mut frame)?;
        self.stream.write_all(&frame)?;
        Ok(())
    }

    fn next(&mut self) -> anyhow::Result<Vec<u8>> {
        loop {
            if let Some(packet) = self.poll()? {
                return Ok(packet);
            }
        }
    }

    fn poll(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        if let Some(packet) = self.decoder.next_packet()? {
            return Ok(Some(packet));
        }
        match self.stream.read(&mut self.buffer) {
            Ok(0) => bail!("connection closed by server"),
            Ok(read) => {
                self.decoder.feed(&self.buffer[..read]);
                Ok(self.decoder.next_packet()?)
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                Ok(None)
            }
            Err(error) => Err(error.into()),
        }
    }
}

struct Empty;

impl Encode for Empty {
    fn encode(&self, _: &mut Vec<u8>) {}
}

struct Raw(Vec<u8>);

impl Encode for Raw {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.0);
    }
}

fn reason(body: &[u8]) -> String {
    let mut input = body;
    Text::decode(&mut input).map_or_else(|_| "<undecodable>".to_owned(), |t| format!("{:?}", t.0))
}

fn main() -> anyhow::Result<()> {
    let options = options();
    let started = Instant::now();
    let stream = TcpStream::connect(&options.address)
        .with_context(|| format!("connecting to {}", options.address))?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(options.timeout))?;
    let mut client = Client {
        stream,
        encoder: FrameEncoder::new(),
        decoder: FrameDecoder::new(),
        buffer: vec![0u8; 64 * 1024],
    };
    let (host, port) = options
        .address
        .rsplit_once(':')
        .unwrap_or(("127.0.0.1", "25565"));
    client.send(
        ids::handshake::serverbound::INTENTION,
        &Intention {
            protocol_version: rspigot_data::PROTOCOL_VERSION,
            host: host.to_owned(),
            port: port.parse().unwrap_or(25565),
            intent: ClientIntent::Login,
        },
    )?;
    client.send(
        ids::login::serverbound::HELLO,
        &ServerboundHello {
            name: options.name.clone(),
            profile_id: PlayerId::offline(&options.name).0,
        },
    )?;
    loop {
        let packet = client.next()?;
        let (id, body) = split_packet_id(&packet)?;
        match id {
            ids::login::clientbound::LOGIN_COMPRESSION => {
                let LoginCompression { threshold } = decode_exact(body)?;
                let threshold = usize::try_from(threshold).ok();
                client
                    .encoder
                    .set_compression(threshold.map(|threshold| Compression {
                        threshold,
                        level: 6,
                    }));
                client.decoder.set_compression(threshold);
            }
            ids::login::clientbound::LOGIN_FINISHED => {
                client.send(
                    ids::login::serverbound::LOGIN_ACKNOWLEDGED,
                    &LoginAcknowledged,
                )?;
                break;
            }
            ids::login::clientbound::HELLO => {
                bail!("server is in online mode; the probe only supports offline mode")
            }
            ids::login::clientbound::LOGIN_DISCONNECT => bail!(
                "disconnected during login: {}",
                String::from_utf8_lossy(body)
            ),
            _ => {}
        }
    }
    let login_done = started.elapsed();
    client.send(
        ids::configuration::serverbound::CLIENT_INFORMATION,
        &ClientInformation {
            language: "en_us".to_owned(),
            view_distance: options.view_distance,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customisation: 0x7F,
            main_hand: MainHand::Right,
            text_filtering_enabled: false,
            allows_listing: true,
            particle_status: ParticleStatus::All,
        },
    )?;
    let mut registries = 0usize;
    loop {
        let packet = client.next()?;
        let (id, body) = split_packet_id(&packet)?;
        match id {
            ids::configuration::clientbound::SELECT_KNOWN_PACKS => {
                let core = rspigot_data::KNOWN_PACK;
                client.send(
                    ids::configuration::serverbound::SELECT_KNOWN_PACKS,
                    &ServerboundSelectKnownPacks {
                        packs: vec![KnownPack {
                            namespace: core.namespace.to_owned(),
                            id: core.id.to_owned(),
                            version: core.version.to_owned(),
                        }],
                    },
                )?;
            }
            ids::configuration::clientbound::REGISTRY_DATA => registries += 1,
            ids::configuration::clientbound::KEEP_ALIVE => {
                let keep_alive: KeepAlive = decode_exact(body)?;
                client.send(ids::configuration::serverbound::KEEP_ALIVE, &keep_alive)?;
            }
            ids::configuration::clientbound::PING => {
                let Ping { id } = decode_exact(body)?;
                client.send(ids::configuration::serverbound::PONG, &Pong { id })?;
            }
            ids::configuration::clientbound::CODE_OF_CONDUCT => client.send(
                ids::configuration::serverbound::ACCEPT_CODE_OF_CONDUCT,
                &Empty,
            )?,
            ids::configuration::clientbound::FINISH_CONFIGURATION => {
                client.send(
                    ids::configuration::serverbound::FINISH_CONFIGURATION,
                    &ServerboundFinishConfiguration,
                )?;
                break;
            }
            ids::configuration::clientbound::DISCONNECT => {
                bail!("disconnected during configuration: {}", reason(body))
            }
            _ => {}
        }
    }
    let configuration_done = started.elapsed();
    let mut chunks = HashSet::new();
    let mut duplicates = 0usize;
    let mut first_chunk = None;
    let mut position = (0.0f64, 0.0f64, 0.0f64);
    let mut joined = false;
    let mut target_reached = None;
    let mut walked = 0.0f64;
    let mut last_step = Instant::now();
    let mut walk_started = None;
    let mut walk_chunks = 0usize;
    let mut missing_samples = Vec::new();
    let mut command_sent: Option<Instant> = None;
    let mut command_log: Vec<String> = Vec::new();
    client
        .stream
        .set_read_timeout(Some(Duration::from_millis(5)))?;
    loop {
        if started.elapsed() > options.timeout {
            bail!("timed out with {} chunks", chunks.len());
        }
        let packet = client.poll()?;
        if packet.is_none() && target_reached.is_none() {
            continue;
        }
        let packet = packet.unwrap_or_default();
        let (id, body) = if packet.is_empty() {
            (-1, &packet[..])
        } else {
            split_packet_id(&packet)?
        };
        match id {
            ids::play::clientbound::LOGIN => joined = true,
            ids::play::clientbound::KEEP_ALIVE => {
                let keep_alive: KeepAlive = decode_exact(body)?;
                client.send(ids::play::serverbound::KEEP_ALIVE, &keep_alive)?;
            }
            ids::play::clientbound::PING => {
                let Ping { id } = decode_exact(body)?;
                client.send(ids::play::serverbound::PONG, &Pong { id })?;
            }
            ids::play::clientbound::PLAYER_POSITION => {
                let mut input = body;
                let teleport = VarInt::decode(&mut input)?;
                let x = f64::decode(&mut input)?;
                let y = f64::decode(&mut input)?;
                let z = f64::decode(&mut input)?;
                position = (x, y, z);
                let mut raw = Vec::new();
                teleport.encode(&mut raw);
                x.encode(&mut raw);
                y.encode(&mut raw);
                z.encode(&mut raw);
                0.0f32.encode(&mut raw);
                0.0f32.encode(&mut raw);
                client.send(ids::play::serverbound::ACCEPT_TELEPORTATION, &Raw(raw))?;
            }
            ids::play::clientbound::CHUNK_BATCH_FINISHED => {
                client.send(ids::play::serverbound::CHUNK_BATCH_RECEIVED, &options.rate)?
            }
            ids::play::clientbound::LEVEL_CHUNK_WITH_LIGHT => {
                let mut input = body;
                let x = i32::decode(&mut input)?;
                let z = i32::decode(&mut input)?;
                if !chunks.insert((x, z)) {
                    duplicates += 1;
                }
                if walk_started.is_some() {
                    walk_chunks += 1;
                }
                first_chunk.get_or_insert_with(|| started.elapsed());
            }
            ids::play::clientbound::FORGET_LEVEL_CHUNK => {
                let mut input = body;
                let packed = i64::decode(&mut input)?;
                let z = (packed >> 32) as i32;
                let x = packed as i32;
                chunks.remove(&(x, z));
            }
            ids::play::clientbound::DISCONNECT => bail!("disconnected: {}", reason(body)),
            _ => {}
        }
        if command_sent.is_some() {
            match id {
                ids::play::clientbound::UPDATE_MOB_EFFECT => {
                    let mut input = body;
                    let entity = VarInt::decode(&mut input)?.0;
                    let effect = VarInt::decode(&mut input)?.0;
                    let amplifier = VarInt::decode(&mut input)?.0;
                    let duration = VarInt::decode(&mut input)?.0;
                    let flags = u8::decode(&mut input)?;
                    command_log.push(format!("{{\"packet\":\"update_mob_effect\",\"entity\":{entity},\"effect\":{effect},\"amplifier\":{amplifier},\"duration\":{duration},\"flags\":{flags},\"trailing\":{}}}", input.len()));
                }
                ids::play::clientbound::UPDATE_ATTRIBUTES => {
                    let mut input = body;
                    let entity = VarInt::decode(&mut input)?.0;
                    let count = VarInt::decode(&mut input)?.0;
                    let mut parts = Vec::new();
                    for _ in 0..count {
                        let attribute = VarInt::decode(&mut input)?.0;
                        let base = f64::decode(&mut input)?;
                        let modifiers = VarInt::decode(&mut input)?.0;
                        for _ in 0..modifiers {
                            let modifier = String::decode(&mut input)?;
                            let amount = f64::decode(&mut input)?;
                            let operation = VarInt::decode(&mut input)?.0;
                            parts.push(format!("{{\"attribute\":{attribute},\"base\":{base},\"modifier\":\"{modifier}\",\"amount\":{amount},\"operation\":{operation}}}"));
                        }
                    }
                    command_log.push(format!("{{\"packet\":\"update_attributes\",\"entity\":{entity},\"modifiers\":[{}],\"trailing\":{}}}", parts.join(","), input.len()));
                }
                ids::play::clientbound::SYSTEM_CHAT => command_log.push(format!(
                    "{{\"packet\":\"system_chat\",\"text\":\"{}\"}}",
                    reason(&body[..body.len().saturating_sub(1)]).replace('"', "'")
                )),
                _ => {}
            }
            if command_sent.is_some_and(|at: Instant| at.elapsed() > Duration::from_millis(500)) {
                break;
            }
        }
        if joined && target_reached.is_none() && chunks.len() >= options.target_chunks {
            target_reached = Some(started.elapsed());
            client.send(ids::play::serverbound::PLAYER_LOADED, &Empty)?;
            if let Some(command) = &options.command {
                client.send(ids::play::serverbound::CHAT_COMMAND, command)?;
                command_sent = Some(Instant::now());
                continue;
            }
            if options.walk_blocks <= 0.0 {
                break;
            }
        }
        if target_reached.is_some()
            && options.walk_blocks > 0.0
            && last_step.elapsed() >= Duration::from_millis(50)
        {
            last_step = Instant::now();
            walk_started.get_or_insert_with(Instant::now);
            position.0 += options.speed;
            walked += options.speed;
            let cx = (position.0.floor() as i32) >> 4;
            let cz = (position.2.floor() as i32) >> 4;
            let inner = i32::from(options.view_distance) - 2;
            let missing = (-inner..=inner)
                .flat_map(|dx| (-inner..=inner).map(move |dz| (dx, dz)))
                .filter(|(dx, dz)| {
                    within_view(*dx, *dz, inner) && !chunks.contains(&(cx + dx, cz + dz))
                })
                .count();
            missing_samples.push(missing);
            let mut raw = Vec::new();
            position.0.encode(&mut raw);
            position.1.encode(&mut raw);
            position.2.encode(&mut raw);
            raw.push(0);
            client.send(ids::play::serverbound::MOVE_PLAYER_POS, &Raw(raw))?;
            if walked >= options.walk_blocks {
                break;
            }
        }
    }
    for line in &command_log {
        println!("{line}");
    }
    let total = started.elapsed();
    let walk_seconds = walk_started.map_or(0.0, |w| w.elapsed().as_secs_f64());
    let max_missing = missing_samples.iter().copied().max().unwrap_or(0);
    let mean_missing = if missing_samples.is_empty() {
        0.0
    } else {
        missing_samples.iter().sum::<usize>() as f64 / missing_samples.len() as f64
    };
    let ready = target_reached.unwrap_or(total);
    let since_first = ready
        .saturating_sub(first_chunk.unwrap_or(ready))
        .as_secs_f64()
        .max(f64::EPSILON);
    println!(
        "{{\"address\":\"{}\",\"view\":{},\"target\":{},\"registries\":{},\"login_ms\":{},\"config_ms\":{},\"first_chunk_ms\":{},\"ready_ms\":{},\"chunks\":{},\"duplicates\":{},\"chunks_per_s\":{:.1},\"walked\":{},\"walk_s\":{:.1},\"walk_chunks\":{},\"walk_missing_max\":{},\"walk_missing_mean\":{:.2},\"total_ms\":{}}}",
        options.address,
        options.view_distance,
        options.target_chunks,
        registries,
        login_done.as_millis(),
        configuration_done.as_millis(),
        first_chunk.map_or(0, |d| d.as_millis()),
        ready.as_millis(),
        chunks.len(),
        duplicates,
        options.target_chunks as f64 / since_first,
        walked,
        walk_seconds,
        walk_chunks,
        max_missing,
        mean_missing,
        total.as_millis()
    );
    Ok(())
}
