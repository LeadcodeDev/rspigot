# Test server

Run the server from this folder to try it locally. Everything it writes here (`server.properties`, `spigot.yml`, `bukkit.yml`, `ops.json`, `whitelist.json`, the world, the logs) stays out of git; only this file is tracked.

```
cd example/server
cargo run --release -p rspigot
```

Example plugins live in `example/plugins/<name>`.
