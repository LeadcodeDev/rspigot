# welcome

An example plugin: it says "Welcome!" to every player who joins and cancels every move, so players stay where they are.

Start the server in one terminal (it writes its files, `plugins.yml` included, in the directory it runs from):

```
cargo run --release -p rspigot
```

and the plugin in another:

```
cargo run -p welcome
```

With the defaults both sides use `127.0.0.1:25580`. The plugin logs `Connected to rspigot as welcome 0.1.0` once the server accepts it, and `Not connected to rspigot at <url> (<reason>), retrying in <n> s` while it cannot reach it. In the server console, `plugins` shows `Plugins (1): welcome`. Join with a client: the message shows and every step snaps back. Stop the plugin with Ctrl-C and players move again.

To reach another server, set `RSPIGOT_URL` (for example `RSPIGOT_URL=http://10.0.0.5:25580`). If the server has a token, set `RSPIGOT_TOKEN` to it.
