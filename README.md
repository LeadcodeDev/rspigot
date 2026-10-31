# rspigot

A Minecraft Java Edition 26.3 server (protocol 777) written in Rust. It reads and writes Spigot's configuration files and world layout.

## Run

```
cargo build --release -p rspigot
./target/release/rspigot
```

The server creates `server.properties`, `spigot.yml`, `bukkit.yml` and `ops.json` in the working directory on first start, then listens on port 25565. Like vanilla 26.3, `white-list` defaults to `true`: run `whitelist add <name>` in the console (or set `white-list=false`) before joining. Operators from `ops.json` always get in. `online-mode=false` skips Mojang authentication.

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

A handler that takes `&Event` observes: the server does not wait for it and tells it the settled outcome. A handler that takes `&mut Event` decides: it may call `event.cancel()`, and the server holds the move until a decider cancels, all of them answer or `decision-ticks` pass. A cancel that arrives later rolls the move back.

A plugin handles its events one at a time, in the order they arrive. Any `.await` in a handler, observers included, delays the plugin's later decisions: a cancel that misses the deadline no longer prevents the move, it rolls it back.

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
