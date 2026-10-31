use tonic::Status;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("`{0}` is not a valid rspigot URL")]
    Url(String),
    #[error("rspigot refused the plugin: {}", .0.message())]
    Refused(Status),
    #[error("replaced by a newer connection under the same plugin name")]
    Replaced,
    #[error("not connected to rspigot")]
    Disconnected,
    #[error("rspigot did not answer in time")]
    Timeout,
    #[error("rspigot does not know it")]
    NotFound,
    #[error("rspigot sent {0}")]
    Malformed(String),
}
