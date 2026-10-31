use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_channel::Sender;
use rspigot_core::events::Inbound;
use rspigot_core::plugins::{PluginIntro, ToPlugin};
use rspigot_core::ports::ClientLink;
use rspigot_plugin_protocol::PROTOCOL_VERSION;
use rspigot_plugin_protocol::v1::plugin_host_server::PluginHost;
use rspigot_plugin_protocol::v1::{self, from_plugin};
use tokio::sync::{Mutex, mpsc, watch};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tonic_health::ServingStatus;
use tonic_health::server::HealthReporter;

use crate::convert;
use crate::outbox::Outbox;

type Reply = Result<v1::ToPlugin, Status>;

pub(crate) async fn stopped(stop: &mut watch::Receiver<bool>) {
    let _ = stop.wait_for(|stopped| *stopped).await;
}

#[derive(Clone)]
pub(crate) struct Health {
    reporter: HealthReporter,
    holders: Arc<Mutex<HashMap<String, u64>>>,
}

impl Health {
    pub(crate) fn new(reporter: HealthReporter) -> Self {
        Self {
            reporter,
            holders: Arc::default(),
        }
    }

    async fn join(&self, name: &str, connection: u64, announce: impl FnOnce() -> bool) -> bool {
        let mut holders = self.holders.lock().await;
        if !announce() {
            return false;
        }
        holders.insert(name.to_owned(), connection);
        self.reporter
            .set_service_status(name, ServingStatus::Serving)
            .await;
        true
    }

    async fn leave(&self, name: &str, connection: u64) {
        let mut holders = self.holders.lock().await;
        if holders.get(name) == Some(&connection) {
            holders.remove(name);
            self.reporter
                .set_service_status(name, ServingStatus::NotServing)
                .await;
        }
    }
}

pub(crate) struct Host<L> {
    token: Option<String>,
    inbound: Sender<Inbound<L>>,
    health: Health,
    stop: watch::Receiver<bool>,
    next_connection: AtomicU64,
}

impl<L: ClientLink> Host<L> {
    pub(crate) fn new(
        token: Option<String>,
        inbound: Sender<Inbound<L>>,
        health: Health,
        stop: watch::Receiver<bool>,
    ) -> Self {
        Self {
            token,
            inbound,
            health,
            stop,
            next_connection: AtomicU64::new(1),
        }
    }

    async fn open(
        &self,
        mut incoming: Streaming<v1::FromPlugin>,
    ) -> Result<UnboundedReceiverStream<Reply>, Status> {
        let hello = self.introduction(&mut incoming).await?;
        let intro = self.admit(hello)?;
        let connection = self.next_connection.fetch_add(1, Ordering::Relaxed);
        let name = intro.name.clone();
        let version = intro.version.clone();
        let (outbox, from_core) = mpsc::unbounded_channel();
        let (replies, stream) = mpsc::unbounded_channel();
        let connected = Inbound::PluginConnected {
            connection,
            intro,
            outbox: Box::new(Outbox::new(outbox)),
        };
        let announce = || self.inbound.send(connected).is_ok();
        if !self.health.join(&name, connection, announce).await {
            return Err(Status::unavailable("the server is stopping"));
        }
        tracing::info!("Plugin {name} {version} connected");
        let session = Session {
            connection,
            name,
            incoming,
            from_core,
            replies,
            inbound: self.inbound.clone(),
            health: self.health.clone(),
            stop: self.stop.clone(),
        };
        tokio::spawn(session.run());
        Ok(UnboundedReceiverStream::new(stream))
    }

    async fn introduction(
        &self,
        incoming: &mut Streaming<v1::FromPlugin>,
    ) -> Result<v1::Hello, Status> {
        let mut stop = self.stop.clone();
        let first = tokio::select! {
            first = incoming.message() => first?,
            () = stopped(&mut stop) => return Err(Status::unavailable("the server is stopping")),
        };
        match first.and_then(|message| message.body) {
            Some(from_plugin::Body::Hello(hello)) => Ok(hello),
            _ => Err(Status::invalid_argument(
                "the first message must be a Hello",
            )),
        }
    }

    fn admit(&self, hello: v1::Hello) -> Result<PluginIntro, Status> {
        if let Some(token) = &self.token
            && !same_token(token, &hello.token)
        {
            return Err(Status::unauthenticated("wrong plugin token"));
        }
        if hello.protocol_version != PROTOCOL_VERSION {
            return Err(Status::failed_precondition(format!(
                "the server speaks plugin protocol {PROTOCOL_VERSION}, the plugin speaks {}",
                hello.protocol_version
            )));
        }
        let intro =
            convert::intro(hello).map_err(|error| Status::invalid_argument(error.to_string()))?;
        intro
            .validate()
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        Ok(intro)
    }
}

fn same_token(expected: &str, given: &str) -> bool {
    expected.len() == given.len()
        && expected
            .bytes()
            .zip(given.bytes())
            .fold(0, |difference, (a, b)| difference | (a ^ b))
            == 0
}

#[tonic::async_trait]
impl<L: ClientLink> PluginHost for Host<L> {
    type ConnectStream = UnboundedReceiverStream<Reply>;

    async fn connect(
        &self,
        request: Request<Streaming<v1::FromPlugin>>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let opened = self.open(request.into_inner()).await;
        if let Err(status) = &opened {
            tracing::warn!("Refused a plugin: {}", status.message());
        }
        opened.map(Response::new)
    }
}

struct Session<L> {
    connection: u64,
    name: String,
    incoming: Streaming<v1::FromPlugin>,
    from_core: mpsc::UnboundedReceiver<ToPlugin>,
    replies: mpsc::UnboundedSender<Reply>,
    inbound: Sender<Inbound<L>>,
    health: Health,
    stop: watch::Receiver<bool>,
}

impl<L: ClientLink> Session<L> {
    async fn run(mut self) {
        while self.step().await {}
        self.health.leave(&self.name, self.connection).await;
        let _ = self.inbound.send(Inbound::PluginDisconnected {
            connection: self.connection,
        });
        tracing::info!("Plugin {} disconnected", self.name);
    }

    async fn step(&mut self) -> bool {
        tokio::select! {
            received = self.incoming.message() => match received {
                Ok(Some(message)) => {
                    self.forward(message);
                    true
                }
                Ok(None) => false,
                Err(status) => {
                    tracing::debug!(plugin = %self.name, %status, "plugin stream failed");
                    false
                }
            },
            sent = self.from_core.recv() => sent.is_some_and(|message| self.reply(message)),
            () = self.replies.closed() => false,
            () = stopped(&mut self.stop) => false,
        }
    }

    fn forward(&self, message: v1::FromPlugin) {
        match convert::from_plugin(message) {
            Ok(message) => {
                let _ = self.inbound.send(Inbound::PluginMessage {
                    connection: self.connection,
                    message,
                });
            }
            Err(error) => tracing::warn!("Dropped a message from plugin {}: {error}", self.name),
        }
    }

    fn reply(&self, message: ToPlugin) -> bool {
        let replaced = matches!(message, ToPlugin::Replaced);
        let delivered = self.replies.send(Ok(convert::to_plugin(message))).is_ok();
        if replaced {
            let _ = self.replies.send(Err(Status::already_exists(
                "replaced by a newer connection",
            )));
            return false;
        }
        delivered
    }
}
