use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rspigot_core::events::ClientEvent;
use rspigot_core::ports::ClientLink;
use rspigot_domain::text::Component;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum Outgoing {
    Event(ClientEvent),
    Disconnect(Component),
    Pong(i64),
}

#[derive(Clone)]
pub struct NetLink {
    sender: UnboundedSender<Outgoing>,
    connected: Arc<AtomicBool>,
}

impl NetLink {
    pub fn new(sender: UnboundedSender<Outgoing>, connected: Arc<AtomicBool>) -> Self {
        Self { sender, connected }
    }

    pub fn pong(&self, payload: i64) {
        let _ = self.sender.send(Outgoing::Pong(payload));
    }
}

impl ClientLink for NetLink {
    fn send(&self, event: ClientEvent) {
        if self.connected.load(Ordering::Acquire)
            && self.sender.send(Outgoing::Event(event)).is_err()
        {
            self.connected.store(false, Ordering::Release);
        }
    }

    fn disconnect(&self, reason: Component) {
        if self.sender.send(Outgoing::Disconnect(reason)).is_err() {
            self.connected.store(false, Ordering::Release);
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }
}
