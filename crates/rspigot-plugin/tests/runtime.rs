use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

use rspigot_plugin::{
    Access, Call, Error, EventKind, GameMode, HandlerSpec, Listeners, Outcome, Player, PlayerJoin,
    PlayerMove, PlayerState, Plugin, Position, Priority, run_with,
};
use rspigot_plugin_protocol::PROTOCOL_VERSION;
use rspigot_plugin_protocol::v1::plugin_host_server::{PluginHost, PluginHostServer};
use rspigot_plugin_protocol::v1::{self, answer, event, from_plugin, to_plugin};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tonic::{Code, Request, Response, Status, Streaming};

const STEVE: &str = "00000000-0000-0000-0000-000000000001";
const ALEX: &str = "00000000-0000-0000-0000-000000000002";

type Reply = Result<v1::ToPlugin, Status>;
type Replies = UnboundedReceiverStream<Reply>;
type Opened = Pin<Box<dyn Future<Output = Result<Response<Replies>, Status>> + Send>>;

struct Knock {
    hello: v1::Hello,
    incoming: Streaming<v1::FromPlugin>,
    answer: oneshot::Sender<Result<Replies, Status>>,
}

impl Knock {
    fn accept(self) -> Session {
        let (says, said) = mpsc::unbounded_channel();
        let session = Session {
            says,
            hears: self.incoming,
        };
        session.say(to_plugin::Body::Accepted(v1::Accepted {}));
        let _ = self.answer.send(Ok(UnboundedReceiverStream::new(said)));
        session
    }

    fn refuse(self, status: Status) {
        let _ = self.answer.send(Err(status));
    }
}

struct Session {
    says: mpsc::UnboundedSender<Reply>,
    hears: Streaming<v1::FromPlugin>,
}

impl Session {
    fn say(&self, body: to_plugin::Body) {
        self.says
            .send(Ok(v1::ToPlugin { body: Some(body) }))
            .unwrap();
    }

    fn end(self, status: Status) {
        self.says.send(Err(status)).unwrap();
    }

    async fn hear(&mut self) -> from_plugin::Body {
        within(self.hears.message())
            .await
            .unwrap()
            .unwrap()
            .body
            .unwrap()
    }
}

struct Stub {
    knocks: mpsc::UnboundedSender<Knock>,
}

impl Stub {
    fn open(&self, request: Request<Streaming<v1::FromPlugin>>) -> Opened {
        let knocks = self.knocks.clone();
        Box::pin(async move {
            let mut incoming = request.into_inner();
            let hello = match incoming.message().await? {
                Some(v1::FromPlugin {
                    body: Some(from_plugin::Body::Hello(hello)),
                }) => hello,
                _ => return Err(Status::invalid_argument("expected a Hello")),
            };
            let (answer, answered) = oneshot::channel();
            let gone = || Status::unavailable("the stub host is gone");
            knocks
                .send(Knock {
                    hello,
                    incoming,
                    answer,
                })
                .map_err(|_| gone())?;
            answered.await.map_err(|_| gone())?.map(Response::new)
        })
    }
}

impl PluginHost for Stub {
    type ConnectStream = Replies;

    fn connect<'host, 'call>(
        &'host self,
        request: Request<Streaming<v1::FromPlugin>>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Replies>, Status>> + Send + 'call>>
    where
        'host: 'call,
        Self: 'call,
    {
        self.open(request)
    }
}

struct Host {
    url: String,
    knocks: mpsc::UnboundedReceiver<Knock>,
}

impl Host {
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (knocks, knocked) = mpsc::unbounded_channel();
        tokio::spawn(
            Server::builder()
                .add_service(PluginHostServer::new(Stub { knocks }))
                .serve_with_incoming(TcpIncoming::from(listener)),
        );
        Self {
            url,
            knocks: knocked,
        }
    }

    async fn knock(&mut self) -> Knock {
        within(self.knocks.recv()).await.unwrap()
    }
}

#[derive(Debug, PartialEq)]
enum Mark {
    Decided {
        handler: u32,
        event: u64,
    },
    Watched {
        event: u64,
        cancelled: bool,
        rolled_back: bool,
    },
}

#[derive(Debug)]
struct Joined {
    player: Player,
    state: Result<PlayerState, Error>,
    online: Result<Vec<Player>, Error>,
}

struct Guard {
    marks: mpsc::UnboundedSender<Mark>,
    joins: mpsc::UnboundedSender<Joined>,
}

impl Guard {
    async fn on_player_join(&self, event: &PlayerJoin) {
        event
            .player
            .send_message(format!("Welcome, {}!", event.player.name));
        let state = event.player.resolve().await;
        let online = event.player.server().players.fetch().await;
        let _ = self.joins.send(Joined {
            player: event.player.clone(),
            state,
            online,
        });
    }

    async fn on_player_move(&self, event: &mut PlayerMove) {
        let _ = self.marks.send(Mark::Decided {
            handler: 2,
            event: event.id(),
        });
        if event.to.y > 100.0 {
            event.cancel();
        }
    }

    async fn on_player_move_seen(&self, event: &PlayerMove) {
        let _ = self.marks.send(Mark::Watched {
            event: event.id(),
            cancelled: event.is_cancelled(),
            rolled_back: event.is_rolled_back(),
        });
    }

    async fn on_player_move_late(&self, event: &mut PlayerMove) {
        let _ = self.marks.send(Mark::Decided {
            handler: 4,
            event: event.id(),
        });
    }
}

impl Listeners for Guard {
    fn handlers() -> Vec<HandlerSpec> {
        vec![
            spec(
                1,
                EventKind::PlayerJoin,
                Access::Observe,
                Priority::Normal,
                false,
            ),
            spec(
                2,
                EventKind::PlayerMove,
                Access::Decide,
                Priority::High,
                false,
            ),
            spec(
                3,
                EventKind::PlayerMove,
                Access::Observe,
                Priority::Lowest,
                true,
            ),
            spec(
                4,
                EventKind::PlayerMove,
                Access::Decide,
                Priority::Low,
                false,
            ),
        ]
    }

    async fn dispatch(&self, call: Call) -> Outcome {
        match call {
            Call::PlayerJoin { handler: 1, event } => {
                self.on_player_join(&event).await;
                Outcome::Observed
            }
            Call::PlayerMove {
                handler: 2,
                mut event,
            } => {
                self.on_player_move(&mut event).await;
                Outcome::Decided {
                    cancel: event.is_cancelled(),
                }
            }
            Call::PlayerMove { handler: 3, event } => {
                self.on_player_move_seen(&event).await;
                Outcome::Observed
            }
            Call::PlayerMove {
                handler: 4,
                mut event,
            } => {
                self.on_player_move_late(&mut event).await;
                Outcome::Decided {
                    cancel: event.is_cancelled(),
                }
            }
            _ => Outcome::Observed,
        }
    }
}

impl Plugin for Guard {
    const NAME: &'static str = "guard";
    const VERSION: &'static str = "1.2.3";
}

fn spec(
    id: u32,
    event: EventKind,
    access: Access,
    priority: Priority,
    ignore_cancelled: bool,
) -> HandlerSpec {
    HandlerSpec {
        id,
        event,
        access,
        priority,
        ignore_cancelled,
    }
}

struct Running {
    marks: mpsc::UnboundedReceiver<Mark>,
    joins: mpsc::UnboundedReceiver<Joined>,
    ended: JoinHandle<Result<(), Error>>,
}

impl Running {
    async fn joined(&mut self) -> Joined {
        within(self.joins.recv()).await.unwrap()
    }

    fn marks(&mut self) -> Vec<Mark> {
        std::iter::from_fn(|| self.marks.try_recv().ok()).collect()
    }
}

fn launch(host: &Host, token: Option<&str>) -> Running {
    let (marks, marked) = mpsc::unbounded_channel();
    let (joins, joined) = mpsc::unbounded_channel();
    let ended = tokio::spawn(run_with(
        Guard { marks, joins },
        host.url.clone(),
        token.map(str::to_owned),
    ));
    Running {
        marks: marked,
        joins: joined,
        ended,
    }
}

async fn within<F: Future>(future: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(5), future)
        .await
        .expect("timed out")
}

fn snapshot(uuid: &str, name: &str, entity_id: i32) -> v1::PlayerSnapshot {
    v1::PlayerSnapshot {
        uuid: uuid.to_owned(),
        name: name.to_owned(),
        entity_id,
        world: "minecraft:overworld".to_owned(),
        position: Some(v1::Vec3 {
            x: 1.0,
            y: 64.0,
            z: -3.5,
        }),
        game_mode: v1::GameMode::Creative as i32,
    }
}

fn steve() -> v1::PlayerSnapshot {
    snapshot(STEVE, "Steve", 12)
}

fn alex() -> v1::PlayerSnapshot {
    snapshot(ALEX, "Alex", 13)
}

fn joining(player: v1::PlayerSnapshot) -> event::Payload {
    event::Payload::PlayerJoin(v1::PlayerJoin {
        player: Some(player),
    })
}

fn moving_to(height: f64) -> event::Payload {
    event::Payload::PlayerMove(v1::PlayerMove {
        player: Some(steve()),
        from: Some(v1::Vec3 {
            x: 1.0,
            y: 64.0,
            z: -3.5,
        }),
        to: Some(v1::Vec3 {
            x: 1.0,
            y: height,
            z: -3.5,
        }),
    })
}

fn event(
    id: u64,
    handlers: &[u32],
    phase: v1::Phase,
    cancelled: bool,
    payload: event::Payload,
) -> to_plugin::Body {
    to_plugin::Body::Event(v1::Event {
        id,
        handlers: handlers.to_vec(),
        phase: phase as i32,
        cancelled,
        payload: Some(payload),
    })
}

fn answer(request: u64, body: answer::Body) -> to_plugin::Body {
    to_plugin::Body::Answer(v1::Answer {
        request,
        body: Some(body),
    })
}

fn decision(event: u64, cancel: bool) -> from_plugin::Body {
    from_plugin::Body::Decision(v1::Decision { event, cancel })
}

fn message(player: &str, text: &str) -> from_plugin::Body {
    from_plugin::Body::SendMessage(v1::SendMessage {
        player: player.to_owned(),
        text: text.to_owned(),
    })
}

fn wire(
    id: u32,
    event: v1::EventKind,
    access: v1::Access,
    priority: v1::Priority,
    ignore_cancelled: bool,
) -> v1::Handler {
    v1::Handler {
        id,
        event: event as i32,
        access: access as i32,
        priority: priority as i32,
        ignore_cancelled,
    }
}

#[tokio::test]
async fn a_plugin_introduces_its_handlers() {
    let mut host = Host::start().await;
    let _plugin = launch(&host, Some("s3cret"));

    assert_eq!(
        host.knock().await.hello,
        v1::Hello {
            protocol_version: PROTOCOL_VERSION,
            name: "guard".to_owned(),
            version: "1.2.3".to_owned(),
            token: "s3cret".to_owned(),
            handlers: vec![
                wire(
                    1,
                    v1::EventKind::PlayerJoin,
                    v1::Access::Observe,
                    v1::Priority::Normal,
                    false
                ),
                wire(
                    2,
                    v1::EventKind::PlayerMove,
                    v1::Access::Decide,
                    v1::Priority::High,
                    false
                ),
                wire(
                    3,
                    v1::EventKind::PlayerMove,
                    v1::Access::Observe,
                    v1::Priority::Lowest,
                    true
                ),
                wire(
                    4,
                    v1::EventKind::PlayerMove,
                    v1::Access::Decide,
                    v1::Priority::Low,
                    false
                ),
            ],
        }
    );

    let mut open = Host::start().await;
    let _plugin = launch(&open, None);
    assert_eq!(open.knock().await.hello.token, "");
}

#[tokio::test]
async fn a_deciding_handler_answers_its_event() {
    let mut host = Host::start().await;
    let mut plugin = launch(&host, None);
    let mut session = host.knock().await.accept();

    session.say(event(
        7,
        &[2, 4],
        v1::Phase::Decide,
        false,
        moving_to(200.0),
    ));
    assert_eq!(session.hear().await, decision(7, true));
    session.say(event(8, &[2, 4], v1::Phase::Decide, false, moving_to(65.0)));
    assert_eq!(session.hear().await, decision(8, false));
    let nowhere = event::Payload::PlayerMove(v1::PlayerMove {
        player: Some(steve()),
        from: None,
        to: None,
    });
    session.say(event(9, &[2, 4], v1::Phase::Decide, false, nowhere));
    assert_eq!(session.hear().await, decision(9, false));
    let stranger = event::Payload::PlayerMove(v1::PlayerMove {
        player: Some(snapshot("not a uuid", "Steve", 12)),
        from: Some(v1::Vec3 {
            x: 1.0,
            y: 64.0,
            z: -3.5,
        }),
        to: Some(v1::Vec3 {
            x: 1.0,
            y: 200.0,
            z: -3.5,
        }),
    });
    session.say(event(10, &[2, 4], v1::Phase::Decide, false, stranger));
    assert_eq!(session.hear().await, decision(10, false));

    assert_eq!(
        plugin.marks(),
        [
            Mark::Decided {
                handler: 2,
                event: 7
            },
            Mark::Decided {
                handler: 2,
                event: 8
            },
            Mark::Decided {
                handler: 4,
                event: 8
            },
        ]
    );
}

#[tokio::test]
async fn an_observer_answers_nothing() {
    let mut host = Host::start().await;
    let mut plugin = launch(&host, None);
    let mut session = host.knock().await.accept();

    session.say(event(9, &[3], v1::Phase::Settled, true, moving_to(200.0)));
    session.say(event(
        9,
        &[3],
        v1::Phase::RolledBack,
        true,
        moving_to(200.0),
    ));
    session.say(event(10, &[3], v1::Phase::Settled, false, moving_to(65.0)));
    session.say(event(11, &[2], v1::Phase::Decide, false, moving_to(65.0)));
    assert_eq!(session.hear().await, decision(11, false));

    assert_eq!(
        plugin.marks(),
        [
            Mark::Watched {
                event: 9,
                cancelled: true,
                rolled_back: false
            },
            Mark::Watched {
                event: 9,
                cancelled: true,
                rolled_back: true
            },
            Mark::Watched {
                event: 10,
                cancelled: false,
                rolled_back: false
            },
            Mark::Decided {
                handler: 2,
                event: 11
            },
        ]
    );
}

#[tokio::test]
async fn a_handler_can_ask_a_question_while_handling_an_event() {
    let mut host = Host::start().await;
    let mut plugin = launch(&host, None);
    let mut session = host.knock().await.accept();

    session.say(event(1, &[1], v1::Phase::Settled, false, joining(steve())));
    assert_eq!(session.hear().await, message(STEVE, "Welcome, Steve!"));
    let from_plugin::Body::ResolvePlayer(resolve) = session.hear().await else {
        panic!("expected a question about Steve");
    };
    assert_eq!(resolve.player, STEVE);
    session.say(answer(
        resolve.request,
        answer::Body::Player(v1::PlayerState {
            player: Some(steve()),
            health: 17.5,
            food: 12,
            level: 3,
        }),
    ));
    let from_plugin::Body::FetchPlayers(fetch) = session.hear().await else {
        panic!("expected a question about the players");
    };
    assert_ne!(fetch.request, resolve.request);
    session.say(answer(
        fetch.request,
        answer::Body::Players(v1::PlayerList {
            players: vec![steve(), alex()],
        }),
    ));

    let joined = plugin.joined().await;
    assert_eq!(joined.player.uuid.to_string(), STEVE);
    assert_eq!(joined.player.name, "Steve");
    assert_eq!(joined.player.entity_id, 12);
    assert_eq!(joined.player.world, "minecraft:overworld");
    assert_eq!(
        joined.player.position,
        Position {
            x: 1.0,
            y: 64.0,
            z: -3.5
        }
    );
    assert_eq!(joined.player.game_mode, GameMode::Creative);
    let state = joined.state.unwrap();
    assert_eq!(state.player.name, "Steve");
    assert_eq!((state.health, state.food, state.level), (17.5, 12, 3));
    let online = joined.online.unwrap();
    let names: Vec<&str> = online.iter().map(|player| player.name.as_str()).collect();
    assert_eq!(names, ["Steve", "Alex"]);
    assert_eq!(online[1].uuid.to_string(), ALEX);

    session.say(event(2, &[1], v1::Phase::Settled, false, joining(alex())));
    assert_eq!(session.hear().await, message(ALEX, "Welcome, Alex!"));
    let from_plugin::Body::ResolvePlayer(resolve) = session.hear().await else {
        panic!("expected a question about Alex");
    };
    session.say(answer(
        resolve.request,
        answer::Body::NotFound(v1::NotFound {}),
    ));
    let from_plugin::Body::FetchPlayers(fetch) = session.hear().await else {
        panic!("expected a question about the players");
    };
    session.say(answer(
        fetch.request,
        answer::Body::Players(v1::PlayerList {
            players: vec![steve()],
        }),
    ));
    let joined = plugin.joined().await;
    assert!(matches!(joined.state, Err(Error::NotFound)));
    assert_eq!(joined.online.unwrap().len(), 1);
}

#[tokio::test]
async fn a_refused_introduction_stops_the_plugin() {
    for code in [
        Code::Unauthenticated,
        Code::InvalidArgument,
        Code::FailedPrecondition,
    ] {
        let mut host = Host::start().await;
        let plugin = launch(&host, Some("wrong"));
        host.knock().await.refuse(Status::new(code, "refused"));

        let ended = within(plugin.ended).await.unwrap();
        let Err(Error::Refused(status)) = ended else {
            panic!("expected a refusal, got {ended:?}");
        };
        assert_eq!(status.code(), code);
    }
}

#[tokio::test]
async fn a_replaced_plugin_stops() {
    let replaced = || Some(Status::already_exists("replaced by a newer connection"));
    for (told, ending) in [(true, replaced()), (true, None), (false, replaced())] {
        let mut host = Host::start().await;
        let plugin = launch(&host, None);
        let session = host.knock().await.accept();

        if told {
            session.say(to_plugin::Body::Replaced(v1::Replaced {}));
        }
        match ending {
            Some(status) => session.end(status),
            None => drop(session),
        }

        let ended = within(plugin.ended).await.unwrap();
        assert!(matches!(ended, Err(Error::Replaced)), "got {ended:?}");
        assert!(host.knocks.try_recv().is_err());
    }
}

#[tokio::test]
async fn a_lost_connection_is_retried() {
    let mut host = Host::start().await;
    let mut plugin = launch(&host, None);
    let knock = host.knock().await;
    let hello = knock.hello.clone();
    let mut first = knock.accept();

    first.say(event(1, &[1], v1::Phase::Settled, false, joining(steve())));
    assert_eq!(first.hear().await, message(STEVE, "Welcome, Steve!"));
    let from_plugin::Body::ResolvePlayer(_) = first.hear().await else {
        panic!("expected a question about Steve");
    };
    let lost = Instant::now();
    drop(first);

    let joined = plugin.joined().await;
    assert!(matches!(joined.state, Err(Error::Disconnected)));
    assert!(matches!(joined.online, Err(Error::Disconnected)));
    let steve = joined.player;
    assert!(matches!(steve.resolve().await, Err(Error::Disconnected)));
    steve.send_message("nobody hears this");

    let knock = host.knock().await;
    assert!(lost.elapsed() >= Duration::from_millis(900));
    assert_eq!(knock.hello, hello);
    let mut second = knock.accept();
    second.say(event(2, &[2], v1::Phase::Decide, false, moving_to(65.0)));
    assert_eq!(second.hear().await, decision(2, false));
    steve.send_message("back again");
    assert_eq!(second.hear().await, message(STEVE, "back again"));

    second.end(Status::unavailable("the server is stopping"));
    let stopped = Instant::now();
    let knock = host.knock().await;
    let waited = stopped.elapsed();
    assert!(waited >= Duration::from_millis(900));
    assert!(waited < Duration::from_secs(2));
    assert_eq!(knock.hello, hello);
    let mut third = knock.accept();
    third.say(event(3, &[2], v1::Phase::Decide, false, moving_to(200.0)));
    assert_eq!(third.hear().await, decision(3, true));
}

#[tokio::test]
async fn an_old_event_is_not_decided_after_a_reconnect() {
    let mut host = Host::start().await;
    let mut plugin = launch(&host, None);
    let mut first = host.knock().await.accept();

    first.say(event(1, &[1], v1::Phase::Settled, false, joining(steve())));
    assert_eq!(first.hear().await, message(STEVE, "Welcome, Steve!"));
    let from_plugin::Body::ResolvePlayer(_) = first.hear().await else {
        panic!("expected a question about Steve");
    };
    first.say(event(2, &[2], v1::Phase::Decide, false, moving_to(200.0)));
    let Session {
        says,
        hears: _still_open,
    } = first;
    drop(says);

    let joined = plugin.joined().await;
    assert!(matches!(joined.state, Err(Error::Disconnected)));
    let mut second = host.knock().await.accept();
    second.say(event(3, &[2], v1::Phase::Decide, false, moving_to(65.0)));
    assert_eq!(second.hear().await, decision(3, false));
    assert_eq!(
        plugin.marks(),
        [Mark::Decided {
            handler: 2,
            event: 3
        }]
    );
}
