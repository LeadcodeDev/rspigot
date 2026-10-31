use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rspigot_core::config::{GameConfig, LevelData, PluginSettings};
use rspigot_core::events::{ClientEvent, Inbound, PlayerInput};
use rspigot_core::plugins::{
    Access, Answer, EventKind, EventPayload, FromPlugin, HandlerSpec, Phase, PluginIntro, Priority,
    ToPlugin,
};
use rspigot_core::ports::{
    ChunkGenerator, ChunkStore, ClientLink, EntityStore, PluginOutbox, StoreError,
};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Vec3};
use rspigot_nbt::Compound;

#[derive(Clone, Default)]
struct Link {
    events: Arc<Mutex<Vec<ClientEvent>>>,
    closed: Arc<AtomicBool>,
}

impl Link {
    fn take(&self) -> Vec<ClientEvent> {
        self.events
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }
}

impl ClientLink for Link {
    fn send(&self, event: ClientEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    fn disconnect(&self, _: Component) {
        self.closed.store(true, Ordering::Release);
    }

    fn is_connected(&self) -> bool {
        !self.closed.load(Ordering::Acquire)
    }
}

struct NoStore;

impl ChunkStore for NoStore {
    fn load(&self, _: &Identifier, _: ChunkPos) -> Result<Option<ChunkColumn>, StoreError> {
        Ok(None)
    }

    fn save(&self, _: &Identifier, _: &ChunkColumn) -> Result<(), StoreError> {
        Ok(())
    }

    fn flush(&self) -> Result<(), StoreError> {
        Ok(())
    }
}

impl EntityStore for NoStore {
    fn load_entities(
        &self,
        _: &Identifier,
        _: ChunkPos,
    ) -> Result<Option<Vec<Compound>>, StoreError> {
        Ok(None)
    }

    fn save_entities(&self, _: &Identifier, _: ChunkPos, _: &[Compound]) -> Result<(), StoreError> {
        Ok(())
    }
}

struct Floor;

impl ChunkGenerator for Floor {
    fn generate(&self, dimension: &Dimension, pos: ChunkPos) -> ChunkColumn {
        let mut chunk = ChunkColumn::empty(pos, dimension.height, BiomeId(0));
        for x in 0..16 {
            for z in 0..16 {
                for y in 60..=63 {
                    chunk.set_block(
                        BlockPos::new(pos.min_block_x() + x, y, pos.min_block_z() + z),
                        BlockStateId(1),
                    );
                }
            }
        }
        chunk
    }

    fn spawn_point(&self, _: &Dimension) -> Vec3 {
        Vec3::new(0.5, 64.0, 0.5)
    }
}

#[derive(Clone, Default)]
struct Mailbox(Arc<Mutex<Vec<ToPlugin>>>);

impl Mailbox {
    fn received(&self) -> Vec<ToPlugin> {
        self.0.lock().map(|m| m.clone()).unwrap_or_default()
    }
}

impl PluginOutbox for Mailbox {
    fn send(&self, message: ToPlugin) {
        if let Ok(mut inbox) = self.0.lock() {
            inbox.push(message);
        }
    }
}

struct Player {
    id: PlayerId,
    link: Link,
}

fn server(plugins: PluginSettings) -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        spawn_protection: 0,
        plugins,
        ..GameConfig::default()
    };
    let mut server = Server::new(
        config,
        LevelData::default(),
        Arc::new(NoStore),
        Arc::new(Floor),
        StatusBoard::new(20),
    );
    server.wait_for_spawn(Duration::from_secs(10));
    server
}

fn join(server: &mut Server<Link>, name: &str) -> Player {
    let link = Link::default();
    let id = PlayerId::offline(name);
    server.handle(Inbound::Join {
        profile: GameProfile {
            id,
            name: name.to_owned(),
            properties: Vec::new(),
        },
        settings: ClientSettings::default(),
        link: link.clone(),
    });
    server.deliver();
    for event in link.take() {
        if let ClientEvent::Teleport { id: teleport, .. } = event {
            server.handle(Inbound::Input {
                player: id,
                input: PlayerInput::TeleportConfirm(teleport),
            });
        }
    }
    server.handle(Inbound::Input {
        player: id,
        input: PlayerInput::Loaded,
    });
    Player { id, link }
}

fn observe(id: u32, event: EventKind) -> HandlerSpec {
    HandlerSpec {
        id,
        event,
        access: Access::Observe,
        priority: Priority::Normal,
        ignore_cancelled: false,
    }
}

fn connect_with(
    server: &mut Server<Link>,
    connection: u64,
    name: &str,
    handlers: Vec<HandlerSpec>,
) -> Mailbox {
    let mailbox = Mailbox::default();
    server.handle(Inbound::PluginConnected {
        connection,
        intro: PluginIntro {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            handlers,
        },
        outbox: Box::new(mailbox.clone()),
    });
    mailbox
}

fn connect(server: &mut Server<Link>, connection: u64, name: &str) -> Mailbox {
    connect_with(server, connection, name, Vec::new())
}

fn heard(mailbox: &Mailbox) -> Vec<ToPlugin> {
    mailbox
        .received()
        .into_iter()
        .filter(|message| *message != ToPlugin::Accepted)
        .collect()
}

fn order(server: &mut Server<Link>, connection: u64, message: FromPlugin) {
    server.handle(Inbound::PluginMessage {
        connection,
        message,
    });
    server.deliver();
}

fn plugins_line(server: &mut Server<Link>, player: &Player) -> String {
    server.handle(Inbound::Input {
        player: player.id,
        input: PlayerInput::Command("plugins".to_owned()),
    });
    server.deliver();
    player
        .link
        .take()
        .into_iter()
        .find_map(|event| match event {
            ClientEvent::SystemChat { message, .. } => Some(message.plain_text()),
            _ => None,
        })
        .expect("the plugins command answers")
}

#[test]
fn a_plugin_is_accepted_when_it_connects() {
    let mut server = server(PluginSettings::default());
    let welcome = connect(&mut server, 1, "welcome");
    assert_eq!(welcome.received(), [ToPlugin::Accepted]);
}

#[test]
fn the_plugins_command_lists_connected_plugins() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (0): ");
    connect(&mut server, 1, "welcome");
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (1): welcome");
}

#[test]
fn the_plugins_command_follows_the_configured_order() {
    let mut server = server(PluginSettings {
        order: vec!["guard".to_owned()],
        ..PluginSettings::default()
    });
    let alice = join(&mut server, "Alice");
    connect(&mut server, 1, "welcome");
    connect(&mut server, 2, "chat");
    connect(&mut server, 3, "guard");
    assert_eq!(
        plugins_line(&mut server, &alice),
        "Plugins (3): guard, chat, welcome"
    );
}

#[test]
fn a_plugin_that_takes_a_connected_name_replaces_the_old_connection() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    let old = connect(&mut server, 1, "welcome");
    let new = connect(&mut server, 2, "welcome");
    assert_eq!(old.received(), [ToPlugin::Accepted, ToPlugin::Replaced]);
    assert_eq!(new.received(), [ToPlugin::Accepted]);
    server.handle(Inbound::PluginDisconnected { connection: 1 });
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (1): welcome");
}

#[test]
fn a_disconnected_plugin_leaves_the_list() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    connect(&mut server, 1, "welcome");
    server.handle(Inbound::PluginDisconnected { connection: 1 });
    assert_eq!(plugins_line(&mut server, &alice), "Plugins (0): ");
}

#[test]
fn observers_hear_a_player_join_and_leave() {
    let mut server = server(PluginSettings::default());
    let observer = connect_with(
        &mut server,
        1,
        "welcome",
        vec![
            observe(1, EventKind::PlayerJoin),
            observe(2, EventKind::PlayerLeave),
            observe(3, EventKind::PlayerJoin),
        ],
    );
    let alice = join(&mut server, "Alice");

    let joined = heard(&observer);
    assert_eq!(joined.len(), 1);
    let ToPlugin::Event {
        id,
        handlers,
        phase,
        cancelled,
        payload: EventPayload::PlayerJoin { player },
    } = &joined[0]
    else {
        panic!("expected a join event, got {:?}", joined[0]);
    };
    assert_eq!(
        (*id, handlers.as_slice(), *phase, *cancelled),
        (1, &[1, 3][..], Phase::Settled, false)
    );
    assert_eq!(player.id, alice.id);
    assert_eq!(player.name, "Alice");
    assert_eq!(player.world, "minecraft:overworld");
    assert_eq!(player.position, Vec3::new(0.5, 64.0, 0.5));
    assert_eq!(player.game_mode, GameConfig::default().default_game_mode);

    server.handle(Inbound::Leave { player: alice.id });

    let left = heard(&observer);
    assert_eq!(left.len(), 2);
    let ToPlugin::Event {
        id,
        handlers,
        phase,
        cancelled,
        payload: EventPayload::PlayerLeave { player },
    } = &left[1]
    else {
        panic!("expected a leave event, got {:?}", left[1]);
    };
    assert_eq!(
        (*id, handlers.as_slice(), *phase, *cancelled),
        (2, &[2][..], Phase::Settled, false)
    );
    assert_eq!(player.id, alice.id);
    assert_eq!(player.name, "Alice");
    assert_eq!(player.position, Vec3::new(0.5, 64.0, 0.5));
}

#[test]
fn a_plugin_that_does_not_listen_hears_nothing() {
    let mut server = server(PluginSettings::default());
    let quiet = connect(&mut server, 1, "quiet");
    let leaver = connect_with(
        &mut server,
        2,
        "leaver",
        vec![observe(1, EventKind::PlayerLeave)],
    );
    let alice = join(&mut server, "Alice");
    assert!(heard(&quiet).is_empty());
    assert!(heard(&leaver).is_empty());

    server.handle(Inbound::Leave { player: alice.id });
    assert!(heard(&quiet).is_empty());
    assert_eq!(heard(&leaver).len(), 1);
}

#[test]
fn send_message_reaches_the_player() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    connect(&mut server, 1, "welcome");
    alice.link.take();
    order(
        &mut server,
        1,
        FromPlugin::SendMessage {
            player: alice.id,
            text: "hi".to_owned(),
        },
    );
    let messages: Vec<(String, bool)> = alice
        .link
        .take()
        .into_iter()
        .filter_map(|event| match event {
            ClientEvent::SystemChat { message, overlay } => Some((message.plain_text(), overlay)),
            _ => None,
        })
        .collect();
    assert_eq!(messages, [("hi".to_owned(), false)]);
}

#[test]
fn resolve_answers_with_the_current_state_or_not_found() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    let welcome = connect(&mut server, 1, "welcome");

    order(
        &mut server,
        1,
        FromPlugin::ResolvePlayer {
            request: 7,
            player: alice.id,
        },
    );
    server.give_experience(alice.id, 7);
    order(
        &mut server,
        1,
        FromPlugin::ResolvePlayer {
            request: 8,
            player: alice.id,
        },
    );
    order(
        &mut server,
        1,
        FromPlugin::ResolvePlayer {
            request: 9,
            player: PlayerId::offline("Nobody"),
        },
    );

    let answers = heard(&welcome);
    assert_eq!(answers.len(), 3);
    let ToPlugin::Answer {
        request: 7,
        answer: Answer::Player(before),
    } = &answers[0]
    else {
        panic!("expected the state of Alice, got {:?}", answers[0]);
    };
    assert_eq!(before.player.id, alice.id);
    assert_eq!(before.player.name, "Alice");
    assert_eq!(before.health, 20.0);
    assert_eq!(before.food, 20);
    assert_eq!(before.level, 0);
    let ToPlugin::Answer {
        request: 8,
        answer: Answer::Player(after),
    } = &answers[1]
    else {
        panic!("expected the state of Alice, got {:?}", answers[1]);
    };
    assert_eq!(after.level, 1);
    assert_eq!(
        answers[2],
        ToPlugin::Answer {
            request: 9,
            answer: Answer::NotFound
        }
    );
}

#[test]
fn fetch_lists_the_online_players() {
    let mut server = server(PluginSettings::default());
    let welcome = connect(&mut server, 1, "welcome");
    order(&mut server, 1, FromPlugin::FetchPlayers { request: 8 });
    join(&mut server, "Alice");
    join(&mut server, "Bob");
    order(&mut server, 1, FromPlugin::FetchPlayers { request: 9 });

    let answers = heard(&welcome);
    assert_eq!(answers.len(), 2);
    assert_eq!(
        answers[0],
        ToPlugin::Answer {
            request: 8,
            answer: Answer::Players(Vec::new())
        }
    );
    let ToPlugin::Answer {
        request: 9,
        answer: Answer::Players(players),
    } = &answers[1]
    else {
        panic!("expected the online players, got {:?}", answers[1]);
    };
    let names: Vec<&str> = players.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Alice", "Bob"]);
    assert_eq!(players[0].id, PlayerId::offline("Alice"));
    assert_eq!(players[1].id, PlayerId::offline("Bob"));
}

#[test]
fn an_answer_goes_only_to_the_connection_that_asked() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    let asker = connect(&mut server, 1, "asker");
    let bystander = connect(&mut server, 2, "bystander");
    order(
        &mut server,
        1,
        FromPlugin::ResolvePlayer {
            request: 1,
            player: alice.id,
        },
    );
    order(&mut server, 1, FromPlugin::FetchPlayers { request: 2 });
    assert_eq!(heard(&asker).len(), 2);
    assert!(heard(&bystander).is_empty());
}

#[test]
fn messages_from_an_unknown_connection_are_ignored() {
    let mut server = server(PluginSettings::default());
    let alice = join(&mut server, "Alice");
    let old = connect(&mut server, 1, "welcome");
    let new = connect(&mut server, 2, "welcome");
    alice.link.take();
    for connection in [1, 99] {
        order(
            &mut server,
            connection,
            FromPlugin::SendMessage {
                player: alice.id,
                text: "hi".to_owned(),
            },
        );
        order(
            &mut server,
            connection,
            FromPlugin::ResolvePlayer {
                request: 1,
                player: alice.id,
            },
        );
        order(
            &mut server,
            connection,
            FromPlugin::FetchPlayers { request: 2 },
        );
    }
    assert!(alice.link.take().is_empty());
    assert_eq!(heard(&old), [ToPlugin::Replaced]);
    assert!(heard(&new).is_empty());
}
