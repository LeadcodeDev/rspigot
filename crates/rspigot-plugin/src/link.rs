use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use rspigot_plugin_protocol::v1::{self, answer, from_plugin};
use tokio::sync::{mpsc, oneshot};

use crate::Error;

pub(crate) const ANSWER_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) type Outbound = mpsc::UnboundedSender<v1::FromPlugin>;

type Waiting = oneshot::Sender<Option<answer::Body>>;

#[derive(Clone, Default)]
pub(crate) struct Link(Arc<Shared>);

#[derive(Default)]
struct Shared {
    current: Mutex<Option<Connection>>,
    last_request: AtomicU64,
}

struct Connection {
    outbound: Outbound,
    waiting: HashMap<u64, Waiting>,
}

pub(crate) struct Attached(Link);

impl Drop for Attached {
    fn drop(&mut self) {
        let gone = self.0.lock().take();
        if let Some(Connection { outbound, waiting }) = gone {
            drop(outbound);
            drop(waiting);
        }
    }
}

impl Link {
    pub(crate) fn attach(&self, outbound: Outbound) -> Attached {
        *self.lock() = Some(Connection {
            outbound,
            waiting: HashMap::new(),
        });
        Attached(self.clone())
    }

    pub(crate) fn order(&self, body: from_plugin::Body) {
        if let Some(connection) = self.lock().as_ref() {
            let _ = connection.outbound.send(message(body));
        }
    }

    pub(crate) async fn ask(
        &self,
        question: impl FnOnce(u64) -> from_plugin::Body,
    ) -> Result<answer::Body, Error> {
        let (request, answered) = self.post(question)?;
        match tokio::time::timeout(ANSWER_TIMEOUT, answered).await {
            Ok(Ok(Some(body))) => Ok(body),
            Ok(Ok(None)) => Err(Error::Malformed("an empty answer".to_owned())),
            Ok(Err(_)) => Err(Error::Disconnected),
            Err(_) => {
                self.forget(request);
                Err(Error::Timeout)
            }
        }
    }

    fn post(
        &self,
        question: impl FnOnce(u64) -> from_plugin::Body,
    ) -> Result<(u64, oneshot::Receiver<Option<answer::Body>>), Error> {
        let mut current = self.lock();
        let connection = current.as_mut().ok_or(Error::Disconnected)?;
        let request = self.0.last_request.fetch_add(1, Ordering::Relaxed) + 1;
        connection
            .outbound
            .send(message(question(request)))
            .map_err(|_| Error::Disconnected)?;
        let (waiting, answered) = oneshot::channel();
        connection.waiting.insert(request, waiting);
        Ok((request, answered))
    }

    fn forget(&self, request: u64) {
        if let Some(connection) = self.lock().as_mut() {
            connection.waiting.remove(&request);
        }
    }

    pub(crate) fn answer(&self, answer: v1::Answer) {
        let waiting = self
            .lock()
            .as_mut()
            .and_then(|connection| connection.waiting.remove(&answer.request));
        if let Some(waiting) = waiting {
            let _ = waiting.send(answer.body);
        }
    }

    fn lock(&self) -> MutexGuard<'_, Option<Connection>> {
        self.0
            .current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Link")
            .field("connected", &self.lock().is_some())
            .finish()
    }
}

pub(crate) const fn message(body: from_plugin::Body) -> v1::FromPlugin {
    v1::FromPlugin { body: Some(body) }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rspigot_plugin_protocol::v1::question;
    use tokio::time::Instant;

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn an_unanswered_question_gives_up_after_ten_seconds() {
        assert_eq!(ANSWER_TIMEOUT, Duration::from_secs(10));
        let link = Link::default();
        let (outbound, mut sent) = mpsc::unbounded_channel();
        let _attached = link.attach(outbound);
        let started = Instant::now();

        let asked = link
            .ask(|request| {
                from_plugin::Body::Question(v1::Question {
                    request,
                    question: Some(question::Question::Players(v1::Empty {})),
                })
            })
            .await;

        assert!(matches!(asked, Err(Error::Timeout)), "got {asked:?}");
        assert!(started.elapsed() >= ANSWER_TIMEOUT);
        assert!(sent.try_recv().is_ok());
        assert!(link.lock().as_ref().unwrap().waiting.is_empty());
    }
}
