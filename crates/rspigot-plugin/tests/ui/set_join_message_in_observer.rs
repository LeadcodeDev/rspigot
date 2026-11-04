use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

#[listeners]
impl Guard {
    async fn on_player_join(&self, event: &PlayerJoin) {
        event.set_join_message(None);
    }
}

fn main() {}
