use std::collections::HashMap;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crossbeam_channel::Sender;
use rspigot_core::events::Inbound;
use rspigot_core::plugins::{PluginIntro, ToPlugin};
use rspigot_core::ports::ClientLink;
use rspigot_plugin_protocol::PROTOCOL_VERSION;
use rspigot_plugin_protocol::v1::plugin_host_server::PluginHost;
use rspigot_plugin_protocol::v1::{self, from_plugin};
use tokio::sync::{Mutex, mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tonic_health::ServingStatus;
use tonic_health::server::HealthReporter;

use crate::convert;
use crate::outbox::Outbox;

const REPLY_CAPACITY: usize = 4096;

type Reply = Result<v1::ToPlugin, Status>;
type Ending = ControlFlow<Option<Status>>;

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
    introduction_timeout: Duration,
    next_connection: AtomicU64,
}

impl<L: ClientLink> Host<L> {
    pub(crate) fn new(
        token: Option<String>,
        inbound: Sender<Inbound<L>>,
        health: Health,
        stop: watch::Receiver<bool>,
        introduction_timeout: Duration,
    ) -> Self {
        Self {
            token,
            inbound,
            health,
            stop,
            introduction_timeout,
            next_connection: AtomicU64::new(1),
        }
    }

    async fn open(
        &self,
        mut incoming: Streaming<v1::FromPlugin>,
    ) -> Result<ReceiverStream<Reply>, Status> {
        let hello = self.introduction(&mut incoming).await?;
        let intro = self.admit(hello)?;
        let (outbox, from_core) = mpsc::unbounded_channel();
        let (replies, stream) = mpsc::channel(REPLY_CAPACITY + 1);
        let session = Session {
            connection: self.next_connection.fetch_add(1, Ordering::Relaxed),
            name: intro.name.clone(),
            incoming,
            from_core,
            replies,
            inbound: self.inbound.clone(),
            health: self.health.clone(),
            stop: self.stop.clone(),
        };
        tokio::spawn(session.run(intro, Outbox::new(outbox)));
        Ok(ReceiverStream::new(stream))
    }

    async fn introduction(
        &self,
        incoming: &mut Streaming<v1::FromPlugin>,
    ) -> Result<v1::Hello, Status> {
        let mut stop = self.stop.clone();
        let first = tokio::select! {
            first = tokio::time::timeout(self.introduction_timeout, incoming.message()) => {
                first.map_err(|_| Status::deadline_exceeded("no introduction"))??
            }
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
    type ConnectStream = ReceiverStream<Reply>;

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
    replies: mpsc::Sender<Reply>,
    inbound: Sender<Inbound<L>>,
    health: Health,
    stop: watch::Receiver<bool>,
}

impl<L: ClientLink> Session<L> {
    async fn run(mut self, intro: PluginIntro, outbox: Outbox) {
        let version = intro.version.clone();
        let connected = Inbound::PluginConnected {
            connection: self.connection,
            intro,
            outbox: Box::new(outbox),
        };
        let announce = || self.inbound.send(connected).is_ok();
        if !self
            .health
            .join(&self.name, self.connection, announce)
            .await
        {
            let _ = self
                .replies
                .try_send(Err(Status::unavailable("the server is stopping")));
            return;
        }
        tracing::info!("Plugin {} {version} connected", self.name);
        let ending = loop {
            if let ControlFlow::Break(ending) = self.step().await {
                break ending;
            }
        };
        if let Some(status) = ending {
            let _ = self.replies.try_send(Err(status));
        }
        self.health.leave(&self.name, self.connection).await;
        let _ = self.inbound.send(Inbound::PluginDisconnected {
            connection: self.connection,
        });
        tracing::info!("Plugin {} disconnected", self.name);
    }

    async fn step(&mut self) -> Ending {
        tokio::select! {
            received = self.incoming.message() => match received {
                Ok(Some(message)) => self.forward(message),
                Ok(None) => ControlFlow::Break(None),
                Err(status) => {
                    tracing::debug!(plugin = %self.name, %status, "plugin stream failed");
                    ControlFlow::Break(None)
                }
            },
            sent = self.from_core.recv() => match sent {
                Some(message) => self.reply(message),
                None => ControlFlow::Break(None),
            },
            () = self.replies.closed() => ControlFlow::Break(None),
            () = stopped(&mut self.stop) => ControlFlow::Break(None),
        }
    }

    fn forward(&self, message: v1::FromPlugin) -> Ending {
        let mut warnings = Vec::new();
        match convert::from_plugin(message, &mut warnings) {
            Ok(message) => {
                let _ = self.inbound.send(Inbound::PluginMessage {
                    connection: self.connection,
                    message,
                });
            }
            Err(error) => tracing::warn!("Dropped a message from plugin {}: {error}", self.name),
        }
        for warning in warnings {
            self.reply(ToPlugin::Warning(warning))?;
        }
        ControlFlow::Continue(())
    }

    fn reply(&self, message: ToPlugin) -> Ending {
        if self.replies.capacity() <= 1 {
            tracing::warn!(
                "Plugin {} is not reading its stream, dropping it",
                self.name
            );
            return ControlFlow::Break(Some(Status::resource_exhausted(
                "the plugin is not reading its stream",
            )));
        }
        let replaced = matches!(message, ToPlugin::Replaced);
        if self
            .replies
            .try_send(Ok(convert::to_plugin(message)))
            .is_err()
        {
            return ControlFlow::Break(None);
        }
        if replaced {
            return ControlFlow::Break(Some(Status::already_exists(
                "replaced by a newer connection",
            )));
        }
        ControlFlow::Continue(())
    }
}
