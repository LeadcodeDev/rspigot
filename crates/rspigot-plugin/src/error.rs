use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`{0}` is not a valid rspigot URL")]
    Url(String),
    #[error("rspigot refused the plugin: {}", .0.message())]
    Refused(Status),
    #[error("not connected to rspigot")]
    Disconnected,
    #[error("rspigot does not know it")]
    NotFound,
    #[error("rspigot sent {0}")]
    Malformed(String),
}
