use std::io::BufRead;

use crossbeam_channel::Sender;
use rspigot_core::events::Inbound;
use rspigot_network::NetLink;

pub fn spawn(inbound: Sender<Inbound<NetLink>>) {
    let spawned = std::thread::Builder::new()
        .name("console".to_owned())
        .spawn(move || {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                let Ok(line) = line else {
                    break;
                };
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if inbound.send(Inbound::Console(line.to_owned())).is_err() {
                    break;
                }
            }
        });
    if let Err(error) = spawned {
        tracing::error!(%error, "console thread failed to start");
    }
}
