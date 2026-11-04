use std::future::Future;
use std::time::Duration;

use rspigot_plugin_protocol::PROTOCOL_VERSION;
use rspigot_plugin_protocol::v1::plugin_host_client::PluginHostClient;
use rspigot_plugin_protocol::v1::{self, from_plugin, to_plugin};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::transport::Endpoint;
use tonic::{Code, Status, Streaming};

use crate::Error;
use crate::event::Received;
use crate::link::{ANSWER_TIMEOUT, Link, Outbound, message};
use crate::listener::{HandlerSpec, Listeners, Outcome, Plugin, Verdict};
use crate::server::Server;

const DEFAULT_URL: &str = "http://127.0.0.1:25580";
const FIRST_RETRY: Duration = Duration::from_secs(1);
const LAST_RETRY: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(10);
const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(20);

enum Queued {
    Event {
        event: Box<v1::Event>,
        reply: mpsc::WeakUnboundedSender<v1::FromPlugin>,
    },
    Enable(Link),
    Disable,
}

type Queue = mpsc::UnboundedSender<Queued>;

enum Ended {
    Refused(Status),
    Replaced,
    Lost(String),
}

pub async fn run<P: Plugin>(plugin: P) -> Result<(), Error> {
    let url = std::env::var("RSPIGOT_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| DEFAULT_URL.to_owned());
    let token = std::env::var("RSPIGOT_TOKEN").ok();
    run_with(plugin, url, token).await
}

pub async fn run_with<P: Plugin>(
    plugin: P,
    url: String,
    token: Option<String>,
) -> Result<(), Error> {
    let endpoint = endpoint(&url)?;
    let link = Link::default();
    let (queue, queued) = mpsc::unbounded_channel();
    let hello = v1::Hello {
        protocol_version: PROTOCOL_VERSION,
        name: P::NAME.to_owned(),
        version: P::VERSION.to_owned(),
        token: token.unwrap_or_default(),
        handlers: P::handlers()
            .into_iter()
            .map(HandlerSpec::to_wire)
            .collect(),
    };
    let (error, ()) = tokio::join!(
        keep_connected(endpoint, url, hello, &link, queue),
        handle_events(&plugin, &link, queued),
    );
    Err(error)
}

fn endpoint(url: &str) -> Result<Endpoint, Error> {
    let endpoint = Endpoint::from_shared(url.to_owned()).map_err(|_| Error::Url(url.to_owned()))?;
    if !matches!(endpoint.uri().scheme_str(), Some("http" | "https")) {
        return Err(Error::Url(url.to_owned()));
    }
    Ok(endpoint
        .connect_timeout(CONNECT_TIMEOUT)
        .http2_keep_alive_interval(KEEP_ALIVE_INTERVAL)
        .keep_alive_timeout(KEEP_ALIVE_TIMEOUT)
        .keep_alive_while_idle(true))
}

async fn keep_connected(
    endpoint: Endpoint,
    url: String,
    hello: v1::Hello,
    link: &Link,
    queue: Queue,
) -> Error {
    let mut backoff = Backoff::default();
    loop {
        let reason = match connect(&endpoint, &hello, link, &queue, &mut backoff).await {
            Ended::Refused(status) => return Error::Refused(status),
            Ended::Replaced => return Error::Replaced,
            Ended::Lost(reason) => reason,
        };
        let delay = backoff.delay();
        tracing::warn!(
            "Not connected to rspigot at {url} ({reason}), retrying in {} s",
            delay.as_secs()
        );
        tokio::time::sleep(delay).await;
    }
}

async fn connect(
    endpoint: &Endpoint,
    hello: &v1::Hello,
    link: &Link,
    queue: &Queue,
    backoff: &mut Backoff,
) -> Ended {
    let channel = match endpoint.connect().await {
        Ok(channel) => channel,
        Err(error) => return Ended::Lost(causes(&error)),
    };
    let (outbound, said) = mpsc::unbounded_channel();
    let _ = outbound.send(message(from_plugin::Body::Hello(hello.clone())));
    let mut hears = match PluginHostClient::new(channel)
        .connect(UnboundedReceiverStream::new(said))
        .await
    {
        Ok(response) => response.into_inner(),
        Err(status) => return ended(status),
    };
    if let Err(ended) = accepted(hears.message()).await {
        return ended;
    }
    backoff.reset();
    tracing::info!("Connected to rspigot as {} {}", hello.name, hello.version);
    let mut reading = JoinSet::new();
    reading.spawn(read(hears, link.clone(), outbound, queue.clone()));
    match reading.join_next().await {
        Some(Ok(ended)) => ended,
        Some(Err(error)) => Ended::Lost(error.to_string()),
        None => Ended::Lost("the stream reader vanished".to_owned()),
    }
}

async fn accepted(
    first: impl Future<Output = Result<Option<v1::ToPlugin>, Status>>,
) -> Result<(), Ended> {
    match tokio::time::timeout(ANSWER_TIMEOUT, first).await {
        Ok(Ok(Some(v1::ToPlugin {
            body: Some(to_plugin::Body::Accepted(_)),
        }))) => Ok(()),
        Ok(Ok(Some(_))) => Err(Ended::Lost(
            "rspigot spoke before accepting the plugin".to_owned(),
        )),
        Ok(Ok(None)) => Err(Ended::Lost("rspigot closed the stream".to_owned())),
        Ok(Err(status)) => Err(ended(status)),
        Err(_) => Err(Ended::Lost(
            "rspigot did not accept the plugin in time".to_owned(),
        )),
    }
}

async fn read(
    hears: Streaming<v1::ToPlugin>,
    link: Link,
    outbound: Outbound,
    queue: Queue,
) -> Ended {
    let reply = outbound.downgrade();
    let _attached = link.attach(outbound);
    if queue.send(Queued::Enable(link.clone())).is_err() {
        return Ended::Lost("the plugin stopped handling events".to_owned());
    }
    let ended = listen(hears, &link, &reply, &queue).await;
    let _ = queue.send(Queued::Disable);
    ended
}

async fn listen(
    mut hears: Streaming<v1::ToPlugin>,
    link: &Link,
    reply: &mpsc::WeakUnboundedSender<v1::FromPlugin>,
    queue: &Queue,
) -> Ended {
    loop {
        let body = match hears.message().await {
            Ok(Some(message)) => message.body,
            Ok(None) => return Ended::Lost("rspigot closed the stream".to_owned()),
            Err(status) => return ended(status),
        };
        match body {
            Some(to_plugin::Body::Event(event)) => {
                let queued = Queued::Event {
                    event: Box::new(event),
                    reply: reply.clone(),
                };
                if queue.send(queued).is_err() {
                    return Ended::Lost("the plugin stopped handling events".to_owned());
                }
            }
            Some(to_plugin::Body::Answer(answer)) => link.answer(answer),
            Some(to_plugin::Body::Warning(warning)) => {
                tracing::warn!("rspigot: {}", warning.message);
            }
            Some(to_plugin::Body::Replaced(_)) => return Ended::Replaced,
            Some(to_plugin::Body::Accepted(_)) | None => {}
        }
    }
}

fn causes(error: &tonic::transport::Error) -> String {
    let mut causes: Vec<String> =
        std::iter::successors(std::error::Error::source(error), |cause| cause.source())
            .map(ToString::to_string)
            .collect();
    causes.dedup();
    if causes.is_empty() {
        error.to_string()
    } else {
        causes.join(": ")
    }
}

fn ended(status: Status) -> Ended {
    match status.code() {
        Code::Unauthenticated | Code::InvalidArgument | Code::FailedPrecondition => {
            Ended::Refused(status)
        }
        Code::AlreadyExists => Ended::Replaced,
        code => Ended::Lost(format!("{code:?}: {}", status.message())),
    }
}

async fn handle_events<P: Listeners>(
    plugin: &P,
    link: &Link,
    mut queued: mpsc::UnboundedReceiver<Queued>,
) {
    while let Some(next) = queued.recv().await {
        match next {
            Queued::Event { event, reply } => handle(plugin, link, *event, reply).await,
            Queued::Enable(connected) => plugin.enable(&Server::new(connected)).await,
            Queued::Disable => plugin.disable().await,
        }
    }
}

async fn handle<P: Listeners>(
    plugin: &P,
    link: &Link,
    mut event: v1::Event,
    reply: mpsc::WeakUnboundedSender<v1::FromPlugin>,
) {
    let id = event.id;
    let handlers = std::mem::take(&mut event.handlers);
    if event.phase() != v1::Phase::Decide {
        match Received::from_wire(event, link) {
            Ok(received) => observe(plugin, &received, &handlers).await,
            Err(error) => tracing::warn!("Skipped event {id}: {error}"),
        }
        return;
    }
    let Some(reply) = reply.upgrade() else {
        tracing::debug!("Dropped event {id}: its connection is gone");
        return;
    };
    let verdicts = match Received::from_wire(event, link) {
        Ok(received) => decide(plugin, &received, &handlers).await,
        Err(error) => {
            tracing::warn!("Let event {id} through: {error}");
            Vec::new()
        }
    };
    let _ = reply.send(message(from_plugin::Body::Decision(v1::Decision {
        event: id,
        verdicts,
    })));
}

async fn observe<P: Listeners>(plugin: &P, received: &Received, handlers: &[u32]) {
    for &handler in handlers {
        plugin.dispatch(received.call(handler)).await;
    }
}

async fn decide<P: Listeners>(
    plugin: &P,
    received: &Received,
    handlers: &[u32],
) -> Vec<v1::Verdict> {
    let mut verdicts = Vec::with_capacity(handlers.len());
    for &handler in handlers {
        let verdict = match plugin.dispatch(received.call(handler)).await {
            Outcome::Decided(verdict) => verdict,
            Outcome::Observed => Verdict::default(),
        };
        let cancelled = verdict.cancel;
        verdicts.push(verdict.into_wire(handler));
        if cancelled {
            break;
        }
    }
    verdicts
}

struct Backoff(Duration);

impl Default for Backoff {
    fn default() -> Self {
        Self(FIRST_RETRY)
    }
}

impl Backoff {
    fn delay(&mut self) -> Duration {
        let delay = self.0;
        self.0 = (delay * 2).min(LAST_RETRY);
        delay
    }

    const fn reset(&mut self) {
        self.0 = FIRST_RETRY;
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Instant;

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn no_acceptance_within_ten_seconds_is_a_lost_connection() {
        let started = Instant::now();

        let accepted = accepted(std::future::pending()).await;

        assert!(matches!(accepted, Err(Ended::Lost(_))));
        assert!(started.elapsed() >= ANSWER_TIMEOUT);
    }

    #[tokio::test]
    async fn a_failed_connection_says_why() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let endpoint = Endpoint::from_shared(format!("http://{address}")).unwrap();
        let (queue, _queued) = mpsc::unbounded_channel();

        let ended = connect(
            &endpoint,
            &v1::Hello::default(),
            &Link::default(),
            &queue,
            &mut Backoff::default(),
        )
        .await;

        let Ended::Lost(reason) = ended else {
            panic!("expected a lost connection");
        };
        assert!(reason.contains("refused"), "{reason}");
    }

    #[test]
    fn the_retry_delay_doubles_up_to_thirty_seconds_and_starts_over_once_accepted() {
        let mut backoff = Backoff::default();
        let delays: Vec<u64> = (0..7).map(|_| backoff.delay().as_secs()).collect();
        assert_eq!(delays, [1, 2, 4, 8, 16, 30, 30]);
        backoff.reset();
        assert_eq!(backoff.delay(), FIRST_RETRY);
    }
}
