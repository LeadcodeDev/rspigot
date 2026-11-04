# rspigot

A Minecraft Java Edition 26.3 server (protocol 777) written in Rust. It reads and writes Spigot's configuration files and world layout.

## Run

```
cargo build --release -p rspigot
./target/release/rspigot
```

The server creates `server.properties`, `spigot.yml`, `bukkit.yml`, `ops.json`, `banned-players.json` and `banned-ips.json` in the working directory on first start, then listens on port 25565. For local testing, run it from `example/server`, which git ignores. Like vanilla 26.3, `white-list` defaults to `true`: run `whitelist add <name>` in the console (or set `white-list=false`) before joining. Operators from `ops.json` always get in. `online-mode=false` skips Mojang authentication.

Options: `--port <n>`, `--world-dir <dir>`, `--online-mode <true|false>`. Type `help` in the console for commands; `stop` or Ctrl-C saves the world and exits.

## Plugins

A plugin is a separate program, never loaded into the server. It connects to the server's plugin port when it starts, so it can crash, restart or be redeployed while the server keeps running. Plugins are written in Rust with the `rspigot-plugin` crate; [`example/plugins/welcome`](example/plugins/welcome) is a complete one. The `plugins` console command lists the connected plugins.

The server creates `plugins.yml` in the working directory on first start:

| Key | Default | Meaning |
|---|---|---|
| `host` | `127.0.0.1` | Address the plugin port listens on |
| `port` | `25580` | The plugin port, separate from the Minecraft port |
| `token` | `''` | Secret every plugin must send; `RSPIGOT_PLUGIN_TOKEN` overrides it |
| `decision-ticks` | `1` | Ticks a move waits for the deciding plugins (1 to 20) |
| `late-cancel-ticks` | `20` | Ticks after a move during which a late cancel still sends the player back (0 to 100) |
| `order` | `[]` | Plugin names; on equal priority the first listed runs first |
| `priorities` | `{}` | Overrides a handler's `#[priority]`, for a whole plugin (`welcome: high`) or one event (`welcome.player_move: lowest`) |

The server refuses to start if `host` is not a loopback address and no token is set. A plugin finds the server through `RSPIGOT_URL` (default `http://127.0.0.1:25580`) and sends `RSPIGOT_TOKEN` when the server has a token.

A handler that takes `&Event` observes: the server does not wait for it and tells it the settled outcome. A handler that takes `&mut Event` decides: it may call `event.cancel()`, and the server holds the move until a decider cancels, all of them answer or `decision-ticks` pass. A cancel that misses that deadline, from a decider that had not answered yet, sends the player back if it arrives within `late-cancel-ticks`; after that it is ignored and the plugin gets a warning.

Handlers take `&self`, so plugin state lives in fields behind a `Mutex` or atomics.
`rspigot-plugin` is not published: depend on it with a path or git dependency on `crates/rspigot-plugin`.

A plugin handles its events one at a time, in the order they arrive. Any `.await` in a handler, observers included, delays the plugin's later decisions: a cancel that misses the deadline no longer prevents the move, at best it rolls it back.

A handler reaches the server through handles. `event.player` is a `Player`. `player.inventory()`, `player.server()` and `player.world()` give their `Inventory`, the `Server` and the `World`. `world.block(pos)` gives a `Block`. An `Entity` is a mob, a dropped item or any other non-player entity: `world.entities.fetch()` lists them, and `world.spawn` and `world.drop_item` wait and return the new one. A player is never an `Entity`: `player.target_entity` returns a `Target`, which is either a `Player` or an `Entity`. Going from one handle to another (`player.world()`, `world.block(pos)`) costs nothing, and a handle's fields (`player.name`, `player.position`) are a snapshot taken when it was made. Methods carry the Rust name of the Spigot method: `setHealth` is `set_health`, `getTargetBlock` is `target_block`.

Most calls are orders (`send_message`, `teleport`, `set_health`, `give_exp`, `kick`, `inventory.add`, `block.set`, `server.broadcast`): they return nothing and the plugin does not wait. The server applies them in the order the plugin sent them, before its next tick, with the code of the matching vanilla command or gameplay, so a plugin teleport behaves like `/tp`. An order about a player, entity or inventory that is gone is dropped, and so is an order sent while the plugin is not connected. An order the server cannot apply as given changes nothing, and the plugin logs one warning: `set_food_level(21)` logs `rspigot: player.set_food_level was not applied: food level 21 is outside 0..=20`. Values Spigot refuses are refused this way, not clamped.

Questions are `async` and cost one round trip. `resolve()` reads one thing: `player.resolve()` returns a `PlayerState` (health, food, experience, effects, ping, address and more), and inventories, blocks, entities, the server and the world have their own. `fetch()` reads a list: `server.players.fetch()`, `server.bans.fetch()`, `world.entities.near(pos, 8.0).of(EntityType::Zombie).fetch()`. `player.target_block(5.0)` and `player.target_entity(5.0)` return what the player looks at, up to 120 blocks away. A question about something that is gone returns `Error::NotFound`, one asked while the plugin is not connected `Error::Disconnected`, and one without an answer after 10 seconds `Error::Timeout`. Orders and questions travel on one stream, so a question sees the effect of every order sent before it. `world.spawn(EntityType::Chicken, pos)` and `world.drop_item(pos, item)` wait like a question; when the server refuses, they return `Error::Rejected` with its reason, for example a monster on peaceful or a chunk that is not loaded.

Text a plugin sends (messages, titles, the action bar, the tab list, kick reasons, broadcasts, item names and lore, inventory titles) uses Spigot's `§` codes. `ChatColor` writes them: `format!("{}Welcome", ChatColor::Gold)` is `"§6Welcome"`. `&` is not a code and shows as typed. As in Spigot, a color code or `§r` also resets bold, italic and the other styles, so an item name that starts with one is not italic. Item names and lore read from the server come back in the same form.

Items, blocks, effects, entity types, sounds, particles and enchantments are enums, so a typo does not compile: `Material::DiamondSword`, `BlockType::Stone`, `EffectType::Speed`, `EntityType::Zombie`, `Sound::EntityCowHurt`, `Particle::HappyVillager`, `Enchantment::Sharpness`. `key()` and `from_key()` convert to and from the namespaced key (`minecraft:diamond_sword`). `Material` and `BlockType` are separate because not every item is a block. `Particle` only lists what the server can send, with the data it needs, such as `Particle::Block(BlockType)` or `Particle::Dust { color, scale }`. The lists in `crates/rspigot-plugin/src/keys` are generated from the 26.3 server data: regenerate them with `python3 -I tools/gen_plugin_keys.py`; `--check` fails when they are stale.

`ItemStack::new(Material::Diamond, 3)` takes `.name()`, `.lore()` and `.enchant(Enchantment::Sharpness, 5)`. They are stored as the vanilla item components, so they show in the tooltip, make the enchantment glint and are saved with the item; the server does not apply enchantment effects yet. An item read from the server keeps its other components (damage, potion contents, shulker box contents, book pages) when it is written back, so moving a damaged sword keeps its damage. Player inventory slots are `InventorySlot::Hotbar(0..=8)`, `Main(0..=26)`, `Armor(ArmorSlot::Helmet)` (or `Chestplate`, `Leggings`, `Boots`) and `Offhand`. `inventory.add` works like `/give`: what does not fit drops at the player's feet.

`server.create_inventory(rows, title)` makes a menu: a chest of 1 to 6 rows where no click moves an item, in it or in the player's own inventory while it is open. `server.create_storage(rows, title)` makes an open one: players move items freely, and `resolve()` returns its contents and viewers. Fill either with `set(slot, item)` and show it with `player.open_inventory(&inventory)`. Only the plugin that made it hears its `InventoryClick` (the slot, such as `ClickedSlot::Menu(4)`, the kind of click and the item) and `InventoryClose` events. An inventory lasts as long as the connection of the plugin that made it: when the plugin disconnects, its inventories close on every client and are forgotten. A plugin holds at most 1024.

Like Spigot's `onEnable` and `onDisable`, the `#[listeners]` block may hold `async fn on_enable(&self, server: &Server)` and `async fn on_disable(&self)`. `on_enable` runs on every accepted connection, the first and each reconnect, before any event; `on_disable` runs when that connection ends. Create menus in `on_enable`: a `CustomInventory` made before a reconnect is dead, orders on it are dropped with a logged warning and its `resolve()` returns `Error::NotFound`.

`world.set_time(6000)` works like Spigot's `setTime`: it moves forward to that time of day and keeps the day count, where `/time set` sets the absolute time. `block.break_naturally()` drops the block's loot whatever tool it needs, so stone gives cobblestone, like Spigot's `breakNaturally()`; `GameRule::BlockDrops` still applies. `play_sound` and `spawn_particle` on a `Player` reach that player only; on the `World` they reach everyone nearby.

`player.ban(reason, expires)`, `server.bans` and `server.ip_bans` use the same ban lists as the `/ban`, `/ban-ip`, `/pardon`, `/pardon-ip` and `/banlist` commands. The lists are kept in `banned-players.json` and `banned-ips.json`, in the vanilla format. Banning a name the server has never seen stores an entry without a UUID; it matches that name at login and gets the UUID then. A vanilla server skips such entries. A ban reason is stored as given, as vanilla stores it, and is not parsed as `§` text; `§` codes in it still render on the client.

## Probe

`rspigot-probe` is an offline-mode client that logs in like a 26.3 client and measures chunk delivery:

```
./target/release/rspigot-probe --address 127.0.0.1:25565 --view 10 --walk 640 --speed 4
```

It prints one JSON line: time to first chunk, time until the whole view is loaded, chunks per second, duplicates, and how many chunks near the player were missing while walking.

## Layout

| Crate | Role |
|---|---|
| `rspigot-data` | Generated 26.3 tables (blocks, items, registries, tags) |
| `rspigot-nbt` | NBT model and codec |
| `rspigot-domain` | World, chunk and player model |
| `rspigot-core` | Ports, game loop, chunk loading and streaming, commands |
| `rspigot-protocol` | Wire format and packets |
| `rspigot-network` | TCP connections (tokio) |
| `rspigot-anvil` | Region file storage |
| `rspigot-worldgen` | Chunk generators |
| `rspigot-auth` | Mojang session verification |
| `apps/rspigot` | The server binary |
| `apps/rspigot-probe` | Test client |

## License

MIT, see LICENSE.
