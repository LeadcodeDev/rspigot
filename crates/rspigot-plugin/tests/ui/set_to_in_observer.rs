use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

#[listeners]
impl Guard {
    async fn on_player_move(&self, event: &PlayerMove) {
        event.set_to(event.from);
    }
}

fn main() {}
