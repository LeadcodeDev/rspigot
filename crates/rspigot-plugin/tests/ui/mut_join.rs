use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

#[listeners]
impl Guard {
    async fn on_player_join(&self, event: &mut PlayerJoin) {
        event.player.send_message("Welcome!");
    }
}

fn main() {}
