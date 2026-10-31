use rspigot_plugin::prelude::*;

#[derive(Plugin)]
struct Guard;

#[listeners]
impl Guard {
    async fn on_player_join(&self, event: &PlayerJoin) {
        event.player.send_message(self.greeting());
    }

    fn greeting(&self) -> &'static str {
        "Welcome!"
    }
}

fn main() {}
