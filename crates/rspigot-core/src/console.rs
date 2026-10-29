use rspigot_domain::text::Component;

fn english(key: &str) -> Option<&'static str> {
    Some(match key {
        "commands.seed.success" => "Seed: %s",
        "multiplayer.player.joined" => "%s joined the game",
        "multiplayer.player.left" => "%s left the game",
        "argument.entity.notfound.player" => "No player was found",
        "commands.gamemode.success.self" => "Set own game mode to %s",
        "commands.gamemode.success.other" => "Set %s's game mode to %s",
        "gameMode.changed" => "Your game mode has been updated to %s",
        "commands.teleport.success.location.single" => "Teleported %s to %s, %s, %s",
        "commands.time.set" => "Set the time to %s",
        "commands.time.query" => "The time is %s",
        "commands.give.success.single" => "Gave %s %s to %s",
        "argument.item.id.invalid" => "Unknown item '%s'",
        "commands.clear.success.single" => "Removed %s item(s) from player %s",
        "commands.kick.success" => "Kicked %s: %s",
        "commands.op.failed" => "Nothing changed. The player is already an operator",
        "commands.deop.failed" => "Nothing changed. The player is not an operator",
        "commands.op.success" => "Made %s a server operator",
        "commands.deop.success" => "Made %s no longer a server operator",
        "commands.difficulty.query" => "The difficulty is %s",
        "commands.difficulty.success" => "The difficulty has been set to %s",
        "commands.weather.set.clear" => "Set the weather to clear",
        "commands.weather.set.rain" => "Set the weather to rain",
        "commands.weather.set.thunder" => "Set the weather to rain and thunder",
        "commands.setworldspawn.success" => "Set the world spawn point to %s, %s, %s [%s]",
        "commands.list.players" => "There are %s of a max of %s players online: %s",
        "options.difficulty.peaceful" => "Peaceful",
        "options.difficulty.easy" => "Easy",
        "options.difficulty.normal" => "Normal",
        "options.difficulty.hard" => "Hard",
        "gameMode.survival" => "Survival Mode",
        "gameMode.creative" => "Creative Mode",
        "gameMode.adventure" => "Adventure Mode",
        "gameMode.spectator" => "Spectator Mode",
        "multiplayer.disconnect.duplicate_login" => "You logged in from another location",
        "multiplayer.disconnect.server_full" => "The server is full!",
        "multiplayer.disconnect.server_shutdown" => "Server closed",
        "multiplayer.disconnect.invalid_player_movement" => "Invalid move player packet received",
        "multiplayer.disconnect.illegal_characters" => "Illegal characters in chat",
        "multiplayer.disconnect.transfers_disabled" => "This server does not accept transfers",
        "multiplayer.disconnect.invalid_player_data" => "Invalid player data",
        "multiplayer.disconnect.unverified_username" => "Failed to verify username!",
        "disconnect.timeout" => "Timed out",
        _ => return None,
    })
}

pub fn console_text(component: &Component) -> String {
    component.render(&english)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_message_reads_like_vanilla_console() {
        let joined =
            Component::translatable("multiplayer.player.joined", vec![Component::text("Probe")]);
        assert_eq!(console_text(&joined), "Probe joined the game");
    }
}
