use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

trait Handlers {
    async fn on_player_move(&self, event: &mut PlayerMove);
}

#[listeners]
impl Handlers for Guard {
    #[priority(High)]
    async fn on_player_move(&self, event: &mut PlayerMove) {
        event.cancel();
    }
}

fn main() {}
