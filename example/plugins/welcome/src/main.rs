use rspigot_plugin::prelude::*;

#[derive(Plugin, Default)]
struct Welcome;

#[listeners]
impl Welcome {
    async fn on_player_join(&self, event: &PlayerJoin) {
        event
            .player
            .send_message(format!("Welcome {}!", event.player.name));
    }

    #[priority(High)]
    async fn on_player_move(&self, event: &mut PlayerMove) {
        event.cancel();
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    rspigot_plugin::run(Welcome).await
}
