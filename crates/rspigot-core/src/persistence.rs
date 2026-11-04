use rspigot_domain::player::GameProfile;

use crate::player::Player;
use crate::player_data::PlayerSave;
use crate::player_io::PlayerIo;
use crate::ports::ClientLink;
use crate::server::Server;
use rspigot_domain::player::PlayerId;

impl<L: ClientLink> Server<L> {
    pub(crate) fn load_saved_player(&self, profile: &GameProfile) -> Option<PlayerSave> {
        let io: &PlayerIo = self.player_io.as_ref()?;
        let data = match io.load(profile.id) {
            Ok(data) => data?,
            Err(error) => {
                tracing::warn!("Failed to load player data for {}: {error}", profile.name);
                return None;
            }
        };
        match PlayerSave::from_nbt(&data) {
            Ok(save) => Some(save),
            Err(error) => {
                tracing::warn!("Ignoring player data for {}: {error}", profile.name);
                None
            }
        }
    }

    pub(crate) fn persist_player(&self, player: &Player<L>) {
        if let Some(io) = &self.player_io {
            io.save(player.id, PlayerSave::capture(player).to_nbt());
        }
    }

    pub(crate) fn persist_all_players(&self) {
        for player in self.players.values() {
            self.persist_player(player);
        }
    }

    pub(crate) fn announce_restored(&mut self, id: PlayerId, save: &PlayerSave) {
        self.send_inventory(id);
        for effect in &save.effects {
            self.apply_effect(id, *effect, None);
        }
    }
}
