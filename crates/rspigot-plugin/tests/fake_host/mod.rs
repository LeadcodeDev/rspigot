#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use rspigot_plugin_protocol::v1::plugin_host_server::{PluginHost, PluginHostServer};
use rspigot_plugin_protocol::v1::{self, answer, event, from_plugin, to_plugin};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tonic::{Request, Response, Status, Streaming};

pub const STEVE: &str = "00000000-0000-0000-0000-000000000001";
pub const ALEX: &str = "00000000-0000-0000-0000-000000000002";

pub type Reply = Result<v1::ToPlugin, Status>;
pub type Replies = UnboundedReceiverStream<Reply>;
type Opened = Pin<Box<dyn Future<Output = Result<Response<Replies>, Status>> + Send>>;

pub struct Knock {
    pub hello: v1::Hello,
    pub incoming: Streaming<v1::FromPlugin>,
    pub answer: oneshot::Sender<Result<Replies, Status>>,
}

impl Knock {
    pub fn accept(self) -> Session {
        let (says, said) = mpsc::unbounded_channel();
        let session = Session {
            says,
            hears: self.incoming,
        };
        session.say(to_plugin::Body::Accepted(v1::Accepted {}));
        let _ = self.answer.send(Ok(UnboundedReceiverStream::new(said)));
        session
    }

    pub fn refuse(self, status: Status) {
        let _ = self.answer.send(Err(status));
    }
}

pub struct Session {
    pub says: mpsc::UnboundedSender<Reply>,
    pub hears: Streaming<v1::FromPlugin>,
}

impl Session {
    pub fn say(&self, body: to_plugin::Body) {
        self.says
            .send(Ok(v1::ToPlugin { body: Some(body) }))
            .unwrap();
    }

    pub fn end(self, status: Status) {
        self.says.send(Err(status)).unwrap();
    }

    pub async fn hear(&mut self) -> from_plugin::Body {
        within(self.hears.message())
            .await
            .unwrap()
            .unwrap()
            .body
            .unwrap()
    }
}

pub struct Stub {
    pub knocks: mpsc::UnboundedSender<Knock>,
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

pub struct Host {
    pub url: String,
    pub knocks: mpsc::UnboundedReceiver<Knock>,
}

impl Host {
    pub async fn start() -> Self {
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

    pub async fn knock(&mut self) -> Knock {
        within(self.knocks.recv()).await.unwrap()
    }
}

pub async fn within<F: Future>(future: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(5), future)
        .await
        .expect("timed out")
}

pub fn snapshot(uuid: &str, name: &str, entity_id: i32) -> v1::PlayerSnapshot {
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
        rotation: Some(v1::Rotation {
            yaw: 90.0,
            pitch: -30.0,
        }),
    }
}

pub fn steve() -> v1::PlayerSnapshot {
    snapshot(STEVE, "Steve", 12)
}

pub fn alex() -> v1::PlayerSnapshot {
    snapshot(ALEX, "Alex", 13)
}

pub fn event(
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

pub fn answer(request: u64, body: answer::Body) -> to_plugin::Body {
    to_plugin::Body::Answer(v1::Answer {
        request,
        body: Some(body),
    })
}
