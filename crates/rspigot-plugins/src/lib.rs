mod config;
mod convert;
mod outbox;
mod service;

use std::time::Duration;

use crossbeam_channel::Sender;
use rspigot_core::events::Inbound;
use rspigot_core::ports::ClientLink;
use rspigot_plugin_protocol::v1::plugin_host_server::PluginHostServer;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tonic_health::ServingStatus;

pub use config::PluginHostConfig;

use crate::service::{Health, Host};

const INTRODUCTION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum PluginHostError {
    #[error(
        "the plugin port listens beyond loopback, so it needs a token (`token` in plugins.yml or RSPIGOT_PLUGIN_TOKEN)"
    )]
    TokenRequired,
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
}

pub async fn run<L: ClientLink>(
    config: PluginHostConfig,
    inbound: Sender<Inbound<L>>,
    stop: watch::Receiver<bool>,
) -> Result<(), PluginHostError> {
    config.validate()?;
    let listener = TcpListener::bind((config.host, config.port)).await?;
    serve(listener, config.token, inbound, stop).await
}

pub async fn serve<L: ClientLink>(
    listener: TcpListener,
    token: Option<String>,
    inbound: Sender<Inbound<L>>,
    stop: watch::Receiver<bool>,
) -> Result<(), PluginHostError> {
    serve_with(listener, token, inbound, stop, INTRODUCTION_TIMEOUT).await
}

async fn serve_with<L: ClientLink>(
    listener: TcpListener,
    token: Option<String>,
    inbound: Sender<Inbound<L>>,
    stop: watch::Receiver<bool>,
    introduction_timeout: Duration,
) -> Result<(), PluginHostError> {
    tracing::info!("Starting plugin host on {}", listener.local_addr()?);
    let (reporter, health_service) = tonic_health::server::health_reporter();
    let health = Health::new(reporter.clone());
    let host = Host::new(token, inbound, health, stop.clone(), introduction_timeout);
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));
    let mut stopped = stop;
    Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(Duration::from_secs(20)))
        .add_service(health_service)
        .add_service(PluginHostServer::new(host))
        .serve_with_incoming_shutdown(incoming, async move {
            service::stopped(&mut stopped).await;
            reporter
                .set_service_status("", ServingStatus::NotServing)
                .await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rspigot_core::events::ClientEvent;
    use rspigot_domain::text::Component;
    use rspigot_plugin_protocol::v1;
    use rspigot_plugin_protocol::v1::plugin_host_client::PluginHostClient;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;
    use tonic::Code;
    use tonic::transport::Endpoint;

    use super::*;

    struct NoLink;

    impl ClientLink for NoLink {
        fn send(&self, _event: ClientEvent) {}

        fn disconnect(&self, _reason: Component) {}

        fn is_connected(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn a_silent_client_is_refused_after_ten_seconds() {
        assert_eq!(INTRODUCTION_TIMEOUT, Duration::from_secs(10));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (inbound, inbox) = crossbeam_channel::unbounded::<Inbound<NoLink>>();
        let (_stop, stopped) = watch::channel(false);
        let wait = Duration::from_millis(200);
        tokio::spawn(serve_with(listener, None, inbound, stopped, wait));
        let channel = Endpoint::from_shared(format!("http://{address}"))
            .unwrap()
            .connect()
            .await
            .unwrap();
        let (_says, said) = mpsc::channel::<v1::FromPlugin>(1);
        let started = Instant::now();

        let refused = tokio::time::timeout(
            Duration::from_secs(5),
            PluginHostClient::new(channel).connect(ReceiverStream::new(said)),
        )
        .await
        .expect("timed out")
        .unwrap_err();
        assert_eq!(refused.code(), Code::DeadlineExceeded);
        assert_eq!(refused.message(), "no introduction");
        assert!(started.elapsed() >= wait);
        assert!(inbox.try_recv().is_err());
    }
}
