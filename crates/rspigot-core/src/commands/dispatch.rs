use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::{Component, NamedColor};
use rspigot_domain::world::Difficulty;
use rspigot_domain::{BlockPos, ItemId, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSender {
    Console,
    Player(PlayerId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOutcome {
    Success,
    Failure,
    Unknown,
    Denied,
}

pub trait CommandContext {
    fn reply(&mut self, sender: CommandSender, message: Component);
    fn broadcast(&mut self, message: Component);
    fn is_operator(&self, sender: CommandSender) -> bool;
    fn sender_name(&self, sender: CommandSender) -> String;
    fn find_player(&self, name: &str) -> Option<PlayerId>;
    fn player_name(&self, player: PlayerId) -> Option<String>;
    fn player_names(&self) -> Vec<String>;
    fn max_players(&self) -> u32;
    fn player_position(&self, player: PlayerId) -> Option<Vec3>;
    fn set_game_mode(&mut self, player: PlayerId, mode: GameMode);
    fn teleport(&mut self, player: PlayerId, position: Vec3);
    fn day_time(&self) -> i64;
    fn game_time(&self) -> i64;
    fn set_day_time(&mut self, time: i64);
    fn give(&mut self, player: PlayerId, item: ItemId, count: u32) -> bool;
    fn clear_inventory(&mut self, player: PlayerId) -> u32;
    fn kick(&mut self, player: PlayerId, reason: Component);
    fn set_operator(&mut self, name: &str, operator: bool) -> bool;
    fn seed(&self) -> i64;
    fn difficulty(&self) -> Difficulty;
    fn set_difficulty(&mut self, difficulty: Difficulty);
    fn set_weather(&mut self, weather: Weather);
    fn set_world_spawn(&mut self, pos: BlockPos);
    fn save_all(&mut self) -> usize;
    fn tps(&self) -> [f64; 3];
    fn stop(&mut self);
}

const ADMIN_COMMANDS: [&str; 16] = [
    "say",
    "gamemode",
    "tp",
    "teleport",
    "time",
    "give",
    "clear",
    "kick",
    "op",
    "deop",
    "difficulty",
    "weather",
    "setworldspawn",
    "save-all",
    "stop",
    "defaultgamemode",
];

pub fn execute<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    line: &str,
) -> CommandOutcome {
    let line = line.trim().trim_start_matches('/');
    let mut parts = line.split_whitespace();
    let Some(label) = parts.next() else {
        return CommandOutcome::Unknown;
    };
    let label = label.to_ascii_lowercase();
    let label = label
        .strip_prefix("minecraft:")
        .or_else(|| label.strip_prefix("bukkit:"))
        .unwrap_or(&label)
        .to_owned();
    let args: Vec<&str> = parts.collect();
    if ADMIN_COMMANDS.contains(&label.as_str()) && !ctx.is_operator(sender) {
        ctx.reply(
            sender,
            Component::text("I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error.")
                .color(NamedColor::Red),
        );
        return CommandOutcome::Denied;
    }
    let rest = line
        .split_once(char::is_whitespace)
        .map_or("", |(_, r)| r.trim());
    let outcome = match label.as_str() {
        "help" | "?" => help(ctx, sender),
        "list" => list(ctx, sender),
        "seed" => {
            let seed = ctx.seed();
            ctx.reply(
                sender,
                Component::translatable(
                    "commands.seed.success",
                    vec![Component::text(format!("[{seed}]")).color(NamedColor::Green)],
                ),
            );
            CommandOutcome::Success
        }
        "version" | "ver" | "about" => {
            ctx.reply(
                sender,
                Component::text(format!(
                    "This server is running rspigot version {} (MC: {})",
                    env!("CARGO_PKG_VERSION"),
                    rspigot_data::VERSION_NAME
                )),
            );
            CommandOutcome::Success
        }
        "plugins" | "pl" => {
            ctx.reply(
                sender,
                Component::text("Plugins (0): ").color(NamedColor::White),
            );
            CommandOutcome::Success
        }
        "tps" => {
            let [a, b, c] = ctx.tps();
            ctx.reply(
                sender,
                Component::text(format!("TPS from last 1m, 5m, 15m: {a:.1}, {b:.1}, {c:.1}"))
                    .color(NamedColor::Gold),
            );
            CommandOutcome::Success
        }
        "me" => {
            let name = ctx.sender_name(sender);
            ctx.broadcast(Component::text(format!("* {name} {rest}")));
            CommandOutcome::Success
        }
        "say" => {
            let name = ctx.sender_name(sender);
            ctx.broadcast(Component::text(format!("[{name}] {rest}")));
            CommandOutcome::Success
        }
        "msg" | "tell" | "w" => private_message(ctx, sender, &args),
        "gamemode" => gamemode(ctx, sender, &args),
        "tp" | "teleport" => teleport(ctx, sender, &args),
        "time" => time(ctx, sender, &args),
        "give" => give(ctx, sender, &args),
        "clear" => clear(ctx, sender, &args),
        "kick" => kick(ctx, sender, &args),
        "op" | "deop" => operator(ctx, sender, &args, label == "op"),
        "difficulty" => difficulty(ctx, sender, &args),
        "weather" => weather(ctx, sender, &args),
        "setworldspawn" => set_world_spawn(ctx, sender, &args),
        "save-all" => {
            let count = ctx.save_all();
            ctx.reply(
                sender,
                Component::text(format!("Saving the game ({count} chunks)")),
            );
            CommandOutcome::Success
        }
        "stop" => {
            ctx.reply(sender, Component::text("Stopping the server"));
            ctx.stop();
            CommandOutcome::Success
        }
        _ => CommandOutcome::Unknown,
    };
    if outcome == CommandOutcome::Unknown {
        ctx.reply(
            sender,
            Component::text("Unknown command. Type \"/help\" for help."),
        );
    }
    outcome
}

fn usage<C: CommandContext>(ctx: &mut C, sender: CommandSender, text: &str) -> CommandOutcome {
    ctx.reply(
        sender,
        Component::text(format!("Usage: {text}")).color(NamedColor::Red),
    );
    CommandOutcome::Failure
}

fn not_found<C: CommandContext>(ctx: &mut C, sender: CommandSender) -> CommandOutcome {
    ctx.reply(
        sender,
        Component::translatable("argument.entity.notfound.player", Vec::new())
            .color(NamedColor::Red),
    );
    CommandOutcome::Failure
}

fn help<C: CommandContext>(ctx: &mut C, sender: CommandSender) -> CommandOutcome {
    let mut lines = vec![
        "/help",
        "/list",
        "/me <action>",
        "/msg <player> <message>",
        "/seed",
        "/tps",
        "/version",
        "/plugins",
    ];
    if ctx.is_operator(sender) {
        lines.extend([
            "/say <message>",
            "/gamemode <mode> [player]",
            "/tp <x> <y> <z> | <player> [target]",
            "/time set|add|query <value>",
            "/give <player> <item> [count]",
            "/clear [player]",
            "/kick <player> [reason]",
            "/op <player>",
            "/deop <player>",
            "/difficulty [level]",
            "/weather <clear|rain|thunder>",
            "/setworldspawn [x y z]",
            "/save-all",
            "/stop",
        ]);
    }
    ctx.reply(
        sender,
        Component::text("--------- Help ---------").color(NamedColor::Yellow),
    );
    for line in lines {
        ctx.reply(sender, Component::text(line).color(NamedColor::Gold));
    }
    CommandOutcome::Success
}

fn list<C: CommandContext>(ctx: &mut C, sender: CommandSender) -> CommandOutcome {
    let names = ctx.player_names();
    let max = ctx.max_players();
    ctx.reply(
        sender,
        Component::translatable(
            "commands.list.players",
            vec![
                Component::text(names.len().to_string()),
                Component::text(max.to_string()),
                Component::text(names.join(", ")),
            ],
        ),
    );
    CommandOutcome::Success
}

fn private_message<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    args: &[&str],
) -> CommandOutcome {
    if args.len() < 2 {
        return usage(ctx, sender, "/msg <player> <message>");
    }
    let Some(target) = ctx.find_player(args[0]) else {
        return not_found(ctx, sender);
    };
    let from = ctx.sender_name(sender);
    let to = ctx.player_name(target).unwrap_or_default();
    let message = args[1..].join(" ");
    ctx.reply(
        CommandSender::Player(target),
        Component::text(format!("{from} whispers to you: {message}"))
            .color(NamedColor::Gray)
            .italic(true),
    );
    ctx.reply(
        sender,
        Component::text(format!("You whisper to {to}: {message}"))
            .color(NamedColor::Gray)
            .italic(true),
    );
    CommandOutcome::Success
}

fn target_or_self<C: CommandContext>(
    ctx: &C,
    sender: CommandSender,
    arg: Option<&&str>,
) -> Option<PlayerId> {
    match arg {
        Some(name) if *name == "@s" => match sender {
            CommandSender::Player(id) => Some(id),
            CommandSender::Console => None,
        },
        Some(name) => ctx.find_player(name),
        None => match sender {
            CommandSender::Player(id) => Some(id),
            CommandSender::Console => None,
        },
    }
}

fn mode_key(mode: GameMode) -> String {
    format!("gameMode.{}", mode.name())
}

fn gamemode<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    args: &[&str],
) -> CommandOutcome {
    let Some(mode) = args.first().and_then(|m| GameMode::from_name(m)) else {
        return usage(
            ctx,
            sender,
            "/gamemode <survival|creative|adventure|spectator> [player]",
        );
    };
    let Some(target) = target_or_self(ctx, sender, args.get(1)) else {
        return not_found(ctx, sender);
    };
    ctx.set_game_mode(target, mode);
    let mode_name = Component::translatable(mode_key(mode), Vec::new());
    if CommandSender::Player(target) == sender {
        ctx.reply(
            sender,
            Component::translatable("commands.gamemode.success.self", vec![mode_name]),
        );
    } else {
        let name = ctx.player_name(target).unwrap_or_default();
        ctx.reply(
            CommandSender::Player(target),
            Component::translatable("gameMode.changed", vec![mode_name.clone()]),
        );
        ctx.reply(
            sender,
            Component::translatable(
                "commands.gamemode.success.other",
                vec![Component::text(name), mode_name],
            ),
        );
    }
    CommandOutcome::Success
}

fn parse_coordinate(raw: &str, base: f64) -> Option<f64> {
    match raw.strip_prefix('~') {
        Some("") => Some(base),
        Some(offset) => offset.parse::<f64>().ok().map(|o| base + o),
        None => raw.parse::<f64>().ok(),
    }
}

fn parse_position(args: &[&str], base: Vec3) -> Option<Vec3> {
    if args.len() != 3 {
        return None;
    }
    let x = parse_coordinate(args[0], base.x)?;
    let y = parse_coordinate(args[1], base.y)?;
    let z = parse_coordinate(args[2], base.z)?;
    let center = |v: f64, raw: &str| {
        if !raw.contains('.') && !raw.starts_with('~') {
            v + 0.5
        } else {
            v
        }
    };
    Some(Vec3::new(center(x, args[0]), y, center(z, args[2])))
}

fn teleport<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    args: &[&str],
) -> CommandOutcome {
    let base = match sender {
        CommandSender::Player(id) => ctx.player_position(id).unwrap_or(Vec3::ZERO),
        CommandSender::Console => Vec3::ZERO,
    };
    let (target, destination) = match args.len() {
        1 => {
            let Some(dest) = ctx.find_player(args[0]) else {
                return not_found(ctx, sender);
            };
            (target_or_self(ctx, sender, None), ctx.player_position(dest))
        }
        2 => {
            let (Some(who), Some(dest)) = (ctx.find_player(args[0]), ctx.find_player(args[1]))
            else {
                return not_found(ctx, sender);
            };
            (Some(who), ctx.player_position(dest))
        }
        3 => (
            target_or_self(ctx, sender, None),
            parse_position(args, base),
        ),
        4 => {
            let who = ctx.find_player(args[0]);
            let base = who.and_then(|w| ctx.player_position(w)).unwrap_or(base);
            (who, parse_position(&args[1..], base))
        }
        _ => return usage(ctx, sender, "/tp <x> <y> <z> | <player> [target]"),
    };
    let (Some(target), Some(destination)) = (target, destination) else {
        return usage(ctx, sender, "/tp <x> <y> <z> | <player> [target]");
    };
    if !destination.is_finite()
        || destination.x.abs() > 30_000_000.0
        || destination.z.abs() > 30_000_000.0
    {
        return usage(ctx, sender, "/tp <x> <y> <z> | <player> [target]");
    }
    ctx.teleport(target, destination);
    let name = ctx.player_name(target).unwrap_or_default();
    ctx.reply(
        sender,
        Component::translatable(
            "commands.teleport.success.location.single",
            vec![
                Component::text(name),
                Component::text(format!("{:.2}", destination.x)),
                Component::text(format!("{:.2}", destination.y)),
                Component::text(format!("{:.2}", destination.z)),
            ],
        ),
    );
    CommandOutcome::Success
}

fn parse_time(raw: &str) -> Option<i64> {
    let (number, unit) = match raw.chars().last()? {
        'd' => (&raw[..raw.len() - 1], 24000.0),
        's' => (&raw[..raw.len() - 1], 20.0),
        't' => (&raw[..raw.len() - 1], 1.0),
        _ => (raw, 1.0),
    };
    let value: f64 = number.parse().ok()?;
    (value >= 0.0).then(|| (value * unit).round() as i64)
}

fn time<C: CommandContext>(ctx: &mut C, sender: CommandSender, args: &[&str]) -> CommandOutcome {
    match (args.first().copied(), args.get(1).copied()) {
        (Some("set"), Some(value)) => {
            let ticks = match value {
                "day" => Some(1000),
                "noon" => Some(6000),
                "night" => Some(13000),
                "midnight" => Some(18000),
                other => parse_time(other),
            };
            let Some(ticks) = ticks else {
                return usage(ctx, sender, "/time set <day|noon|night|midnight|ticks>");
            };
            ctx.set_day_time(ticks);
            ctx.reply(
                sender,
                Component::translatable(
                    "commands.time.set",
                    vec![Component::text(ticks.to_string())],
                ),
            );
            CommandOutcome::Success
        }
        (Some("add"), Some(value)) => {
            let Some(ticks) = parse_time(value) else {
                return usage(ctx, sender, "/time add <ticks>");
            };
            let now = ctx.day_time() + ticks;
            ctx.set_day_time(now);
            ctx.reply(
                sender,
                Component::translatable(
                    "commands.time.set",
                    vec![Component::text(now.to_string())],
                ),
            );
            CommandOutcome::Success
        }
        (Some("query"), Some(kind)) => {
            let value = match kind {
                "daytime" => ctx.day_time().rem_euclid(24000),
                "gametime" => ctx.game_time(),
                "day" => ctx.day_time().div_euclid(24000),
                _ => return usage(ctx, sender, "/time query <daytime|gametime|day>"),
            };
            ctx.reply(
                sender,
                Component::translatable(
                    "commands.time.query",
                    vec![Component::text(value.to_string())],
                ),
            );
            CommandOutcome::Success
        }
        _ => usage(ctx, sender, "/time <set|add|query> <value>"),
    }
}

fn give<C: CommandContext>(ctx: &mut C, sender: CommandSender, args: &[&str]) -> CommandOutcome {
    if args.len() < 2 {
        return usage(ctx, sender, "/give <player> <item> [count]");
    }
    let Some(target) = target_or_self(ctx, sender, args.first()) else {
        return not_found(ctx, sender);
    };
    let item_name = if args[1].contains(':') {
        args[1].to_owned()
    } else {
        format!("minecraft:{}", args[1])
    };
    let Some(item) = rspigot_data::items::id(&item_name) else {
        ctx.reply(
            sender,
            Component::translatable("argument.item.id.invalid", vec![Component::text(args[1])])
                .color(NamedColor::Red),
        );
        return CommandOutcome::Failure;
    };
    let count = args
        .get(2)
        .and_then(|c| c.parse::<u32>().ok())
        .unwrap_or(1)
        .clamp(1, 6400);
    if !ctx.give(target, ItemId(item), count) {
        return CommandOutcome::Failure;
    }
    let name = ctx.player_name(target).unwrap_or_default();
    ctx.reply(
        sender,
        Component::translatable(
            "commands.give.success.single",
            vec![
                Component::text(count.to_string()),
                Component::text(format!("[{item_name}]")),
                Component::text(name),
            ],
        ),
    );
    CommandOutcome::Success
}

fn clear<C: CommandContext>(ctx: &mut C, sender: CommandSender, args: &[&str]) -> CommandOutcome {
    let Some(target) = target_or_self(ctx, sender, args.first()) else {
        return not_found(ctx, sender);
    };
    let removed = ctx.clear_inventory(target);
    let name = ctx.player_name(target).unwrap_or_default();
    ctx.reply(
        sender,
        Component::translatable(
            "commands.clear.success.single",
            vec![Component::text(removed.to_string()), Component::text(name)],
        ),
    );
    CommandOutcome::Success
}

fn kick<C: CommandContext>(ctx: &mut C, sender: CommandSender, args: &[&str]) -> CommandOutcome {
    let Some(target) = args.first().and_then(|n| ctx.find_player(n)) else {
        return not_found(ctx, sender);
    };
    let reason = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "Kicked by an operator".to_owned()
    };
    let name = ctx.player_name(target).unwrap_or_default();
    ctx.kick(target, Component::text(reason.clone()));
    ctx.reply(
        sender,
        Component::translatable(
            "commands.kick.success",
            vec![Component::text(name), Component::text(reason)],
        ),
    );
    CommandOutcome::Success
}

fn operator<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    args: &[&str],
    grant: bool,
) -> CommandOutcome {
    let Some(name) = args.first() else {
        return usage(
            ctx,
            sender,
            if grant {
                "/op <player>"
            } else {
                "/deop <player>"
            },
        );
    };
    if !ctx.set_operator(name, grant) {
        let key = if grant {
            "commands.op.failed"
        } else {
            "commands.deop.failed"
        };
        ctx.reply(
            sender,
            Component::translatable(key, Vec::new()).color(NamedColor::Red),
        );
        return CommandOutcome::Failure;
    }
    let key = if grant {
        "commands.op.success"
    } else {
        "commands.deop.success"
    };
    ctx.reply(
        sender,
        Component::translatable(key, vec![Component::text(*name)]),
    );
    CommandOutcome::Success
}

fn difficulty<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    args: &[&str],
) -> CommandOutcome {
    let Some(raw) = args.first() else {
        let current = ctx.difficulty();
        ctx.reply(
            sender,
            Component::translatable(
                "commands.difficulty.query",
                vec![Component::translatable(difficulty_key(current), Vec::new())],
            ),
        );
        return CommandOutcome::Success;
    };
    let Some(level) = Difficulty::from_name(raw) else {
        return usage(ctx, sender, "/difficulty <peaceful|easy|normal|hard>");
    };
    ctx.set_difficulty(level);
    ctx.reply(
        sender,
        Component::translatable(
            "commands.difficulty.success",
            vec![Component::translatable(difficulty_key(level), Vec::new())],
        ),
    );
    CommandOutcome::Success
}

fn difficulty_key(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Peaceful => "options.difficulty.peaceful",
        Difficulty::Easy => "options.difficulty.easy",
        Difficulty::Normal => "options.difficulty.normal",
        Difficulty::Hard => "options.difficulty.hard",
    }
}

fn weather<C: CommandContext>(ctx: &mut C, sender: CommandSender, args: &[&str]) -> CommandOutcome {
    let (weather, key) = match args.first().copied() {
        Some("clear") => (Weather::Clear, "commands.weather.set.clear"),
        Some("rain") => (Weather::Rain, "commands.weather.set.rain"),
        Some("thunder") => (Weather::Thunder, "commands.weather.set.thunder"),
        _ => return usage(ctx, sender, "/weather <clear|rain|thunder>"),
    };
    ctx.set_weather(weather);
    ctx.reply(sender, Component::translatable(key, Vec::new()));
    CommandOutcome::Success
}

fn set_world_spawn<C: CommandContext>(
    ctx: &mut C,
    sender: CommandSender,
    args: &[&str],
) -> CommandOutcome {
    let base = match sender {
        CommandSender::Player(id) => ctx.player_position(id).unwrap_or(Vec3::ZERO),
        CommandSender::Console => Vec3::ZERO,
    };
    let position = if args.is_empty() {
        Some(base)
    } else {
        parse_position(args, base)
    };
    let Some(position) = position else {
        return usage(ctx, sender, "/setworldspawn [x y z]");
    };
    let pos = position.block();
    ctx.set_world_spawn(pos);
    ctx.reply(
        sender,
        Component::translatable(
            "commands.setworldspawn.success",
            vec![
                Component::text(pos.x.to_string()),
                Component::text(pos.y.to_string()),
                Component::text(pos.z.to_string()),
            ],
        ),
    );
    CommandOutcome::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeContext {
        replies: Vec<(CommandSender, String)>,
        broadcasts: Vec<String>,
        operator: bool,
        modes: Vec<(PlayerId, GameMode)>,
        teleports: Vec<(PlayerId, Vec3)>,
        day_time: i64,
        stopped: bool,
    }

    fn alice() -> PlayerId {
        PlayerId::offline("Alice")
    }

    impl CommandContext for FakeContext {
        fn reply(&mut self, sender: CommandSender, message: Component) {
            self.replies.push((sender, message.plain_text()));
        }
        fn broadcast(&mut self, message: Component) {
            self.broadcasts.push(message.plain_text());
        }
        fn is_operator(&self, _: CommandSender) -> bool {
            self.operator
        }
        fn sender_name(&self, sender: CommandSender) -> String {
            match sender {
                CommandSender::Console => "Server".to_owned(),
                CommandSender::Player(_) => "Alice".to_owned(),
            }
        }
        fn find_player(&self, name: &str) -> Option<PlayerId> {
            name.eq_ignore_ascii_case("alice").then(alice)
        }
        fn player_name(&self, _: PlayerId) -> Option<String> {
            Some("Alice".to_owned())
        }
        fn player_names(&self) -> Vec<String> {
            vec!["Alice".to_owned()]
        }
        fn max_players(&self) -> u32 {
            20
        }
        fn player_position(&self, _: PlayerId) -> Option<Vec3> {
            Some(Vec3::new(10.0, 64.0, 10.0))
        }
        fn set_game_mode(&mut self, player: PlayerId, mode: GameMode) {
            self.modes.push((player, mode));
        }
        fn teleport(&mut self, player: PlayerId, position: Vec3) {
            self.teleports.push((player, position));
        }
        fn day_time(&self) -> i64 {
            self.day_time
        }
        fn game_time(&self) -> i64 {
            0
        }
        fn set_day_time(&mut self, time: i64) {
            self.day_time = time;
        }
        fn give(&mut self, _: PlayerId, _: ItemId, _: u32) -> bool {
            true
        }
        fn clear_inventory(&mut self, _: PlayerId) -> u32 {
            0
        }
        fn kick(&mut self, _: PlayerId, _: Component) {}
        fn set_operator(&mut self, _: &str, _: bool) -> bool {
            true
        }
        fn seed(&self) -> i64 {
            42
        }
        fn difficulty(&self) -> Difficulty {
            Difficulty::Easy
        }
        fn set_difficulty(&mut self, _: Difficulty) {}
        fn set_weather(&mut self, _: Weather) {}
        fn set_world_spawn(&mut self, _: BlockPos) {}
        fn save_all(&mut self) -> usize {
            0
        }
        fn tps(&self) -> [f64; 3] {
            [20.0; 3]
        }
        fn stop(&mut self) {
            self.stopped = true;
        }
    }

    #[test]
    fn non_operator_cannot_change_game_mode() {
        let mut ctx = FakeContext::default();
        assert_eq!(
            execute(
                &mut ctx,
                CommandSender::Player(alice()),
                "/gamemode creative"
            ),
            CommandOutcome::Denied
        );
        assert!(ctx.modes.is_empty());
    }

    #[test]
    fn operator_changes_own_game_mode() {
        let mut ctx = FakeContext {
            operator: true,
            ..FakeContext::default()
        };
        assert_eq!(
            execute(&mut ctx, CommandSender::Player(alice()), "gamemode c"),
            CommandOutcome::Success
        );
        assert_eq!(ctx.modes, vec![(alice(), GameMode::Creative)]);
    }

    #[test]
    fn relative_teleport_uses_sender_position() {
        let mut ctx = FakeContext {
            operator: true,
            ..FakeContext::default()
        };
        execute(&mut ctx, CommandSender::Player(alice()), "/tp ~ ~5 ~-2");
        assert_eq!(ctx.teleports, vec![(alice(), Vec3::new(10.0, 69.0, 8.0))]);
        execute(&mut ctx, CommandSender::Player(alice()), "/tp 1 2 3");
        assert_eq!(ctx.teleports[1].1, Vec3::new(1.5, 2.0, 3.5));
    }

    #[test]
    fn time_set_and_add() {
        let mut ctx = FakeContext {
            operator: true,
            ..FakeContext::default()
        };
        execute(&mut ctx, CommandSender::Console, "time set night");
        assert_eq!(ctx.day_time, 13000);
        execute(&mut ctx, CommandSender::Console, "time add 1d");
        assert_eq!(ctx.day_time, 37000);
    }

    #[test]
    fn unknown_command_replies_with_spigot_message() {
        let mut ctx = FakeContext::default();
        assert_eq!(
            execute(&mut ctx, CommandSender::Console, "/frobnicate"),
            CommandOutcome::Unknown
        );
        assert_eq!(
            ctx.replies.last().map(|r| r.1.as_str()),
            Some("Unknown command. Type \"/help\" for help.")
        );
    }

    #[test]
    fn stop_stops() {
        let mut ctx = FakeContext {
            operator: true,
            ..FakeContext::default()
        };
        execute(&mut ctx, CommandSender::Console, "stop");
        assert!(ctx.stopped);
    }
}
