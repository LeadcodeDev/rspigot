mod config;
mod convert;
mod outbox;
mod service;

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
    tracing::info!("Starting plugin host on {}", listener.local_addr()?);
    let (reporter, health_service) = tonic_health::server::health_reporter();
    let health = Health::new(reporter.clone());
    let host = Host::new(token, inbound, health, stop.clone());
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));
    let mut stopped = stop;
    Server::builder()
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
