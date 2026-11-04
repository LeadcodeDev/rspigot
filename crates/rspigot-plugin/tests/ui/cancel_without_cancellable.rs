use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

#[listeners]
impl Guard {
    async fn on_player_join(&self, event: &mut PlayerJoin) {
        event.set_join_message(None);
        event.cancel();
    }

    async fn on_player_death(&self, event: &mut PlayerDeath) {
        event.set_keep_inventory(true);
        event.cancel();
    }
}

fn main() {}
