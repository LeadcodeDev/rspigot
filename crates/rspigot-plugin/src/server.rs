use rspigot_plugin_protocol::v1::{self, answer, from_plugin};

use crate::Error;
use crate::link::Link;
use crate::player::Player;

#[derive(Debug, Clone)]
pub struct Server {
    pub players: Players,
}

impl Server {
    pub(crate) const fn new(link: Link) -> Self {
        Self {
            players: Players { link },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Players {
    link: Link,
}

impl Players {
    pub async fn fetch(&self) -> Result<Vec<Player>, Error> {
        let answer = self
            .link
            .ask(|request| from_plugin::Body::FetchPlayers(v1::FetchPlayers { request }))
            .await?;
        match answer {
            answer::Body::Players(list) => list
                .players
                .into_iter()
                .map(|player| Player::from_wire(Some(player), &self.link))
                .collect(),
            answer::Body::NotFound(_) => Err(Error::NotFound),
            answer::Body::Player(_) => Err(Error::Malformed(
                "one player where a player list was expected".to_owned(),
            )),
        }
    }
}
