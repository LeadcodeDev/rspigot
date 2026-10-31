use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

#[listeners]
impl Guard {
    #[ignore_cancelled]
    async fn on_player_move(&self, event: &mut PlayerMove) {
        event.cancel();
    }
}

fn main() {}
