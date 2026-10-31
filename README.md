# rspigot

A Minecraft Java Edition 26.3 server (protocol 777) written in Rust. It reads and writes Spigot's configuration files and world layout.

## Run

```
cargo build --release -p rspigot
./target/release/rspigot
```

The server creates `server.properties`, `spigot.yml`, `bukkit.yml` and `ops.json` in the working directory on first start, then listens on port 25565. Like vanilla 26.3, `white-list` defaults to `true`: run `whitelist add <name>` in the console (or set `white-list=false`) before joining. Operators from `ops.json` always get in. `online-mode=false` skips Mojang authentication.

Options: `--port <n>`, `--world-dir <dir>`, `--online-mode <true|false>`. Type `help` in the console for commands; `stop` or Ctrl-C saves the world and exits.

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
