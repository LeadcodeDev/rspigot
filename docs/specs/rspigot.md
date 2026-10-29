# rspigot: a Spigot 26.3 compatible server in Rust

Goal
  A single binary that speaks the Minecraft Java Edition 26.3 protocol (protocol 777, world version 5023) well enough for an unmodified 26.3 client to join, play and see other players, built as a hexagonal Rust workspace, with faster chunk loading and unloading than Spigot.

Context
  The reference is `spigot-26.3.jar` (Spigot 26.3-R0.1-SNAPSHOT, built 2026-09-15, Java 25). Mojang ships 26.x unobfuscated, so class and method names in the jar are the real ones. Facts come from three sources: the vanilla data generator embedded in the jar (`net.minecraft.data.Main --reports`: packets, blocks, registries, commands), Ghidra 12.2 headless decompilation of the classes, and `javap -c -p` when exact field order matters. Nothing from the jar is committed: only derived tables and layout notes.

Glossary
  chunk column: the 16 x 384 x 16 column sent by `level_chunk_with_light`.
  section: a 16 x 16 x 16 cube of a column, with block states and biomes.
  chunk ticket: a reason for a chunk to stay loaded (a player view, spawn).
  streamer: the per-player component deciding which chunks to send and when.
  link: the driven port the core uses to talk to one connected client.

Invariants
  - The domain crate depends on no I/O, async runtime or wire format.  [type: crate graph]
  - Packet ids come from the generated packet table, never from hand-written numbers.  [test]
  - A section's palette never holds more entries than its bit width allows.  [type + test]
  - A chunk is sent to a client only after it is fully generated or loaded, and at most once per view entry.  [test]
  - No `async-trait`. Ports are synchronous traits; async stays inside the network adapter.  [review]
  - The game state is owned by one thread; adapters reach it through channels only.  [type: no shared &mut]

Out of scope for the first milestone
  - Java plugins (Bukkit API on a JVM). A Rust plugin trait with events replaces it later.
  - Mob AI, redstone, full vanilla terrain parity, structures, loot tables, advancements, recipes beyond the protocol sync.
  - Bedrock, proxies (BungeeCord/Velocity forwarding).

Design
  Domain (`rspigot-domain`): positions, ids, identifiers, paletted containers, sections, chunk columns, light, heightmaps, game profile, game mode, text components, item stacks.
  Data (`rspigot-data`): tables generated from the 26.3 reports and datapack: block states, items, entity types, registries, tags, packet ids.
  NBT (`rspigot-nbt`): tag model, binary read/write, network (nameless root) variant.
  Core (`rspigot-core`): ports, the game loop (20 TPS, one thread), player sessions, chunk manager, streamer, commands, chat.
  Ports (owned by core, all synchronous, static dispatch through generics):
    ChunkStore      load/save a chunk column              -> rspigot-anvil
    ChunkGenerator  produce a new chunk column            -> rspigot-worldgen
    SessionVerifier verify an online-mode join            -> rspigot-auth
    ClientLink      deliver events to one client          -> rspigot-network
  Protocol (`rspigot-protocol`): wire primitives, framing, compression, encryption, every packet of handshake, status, login, configuration and the play packets the server uses. Pure, no I/O.
  Network (`rspigot-network`): tokio listener, connection state machine, mapping between packets and core commands/events.
  Binary (`apps/rspigot`): reads `server.properties`, `bukkit.yml`, `spigot.yml`, wires adapters, runs.
  Errors: one `thiserror` enum per crate boundary; `anyhow` only in the binary.

Chunk loading design
  - Loading and generation run on a worker pool off the game thread, ordered by a priority queue keyed by distance to the nearest interested player; requests that fall out of view are cancelled before they start.
  - Encoded chunk packets are cached per chunk revision and shared between players as `Bytes`, so a chunk is serialized and compressed once, not once per viewer.
  - The streamer sends nearest chunks first and follows the client's `chunk_batch_received` rate, with a higher starting rate than vanilla and a prefetch ring one chunk past the view distance in the movement direction.
  - Unloading is deferred: a chunk without tickets stays in an eviction queue for a grace period, and dirty chunks are written back by the worker pool, not the game thread.
  - Region files stay open in an LRU cache.

Acceptance
  @spec-rspigot-1  A 26.3 client lists the server with its MOTD, version name and player count.
  @spec-rspigot-2  A 26.3 client joins in offline mode and spawns in a generated world.
  @spec-rspigot-3  With online-mode=true, a premium client joins after session verification and encryption.
  @spec-rspigot-4  Packets above the compression threshold are zlib-compressed and readable by the client.
  @spec-rspigot-5  Walking loads new chunks and forgets old ones; no chunk is sent twice while in view.
  @spec-rspigot-6  Two clients see each other move, chat, break and place blocks.
  @spec-rspigot-7  A world saved by rspigot reloads with the same blocks; an Anvil world from Spigot 26.3 loads.
  @spec-rspigot-8  Chunk throughput (load or generate, encode) is measured by a benchmark and reported against the vanilla rate.

Simulation
  The streamer and chunk manager run against an in-memory store and a fake clock to check ordering, cancellation and deferred unload without sleeping.

Verify (exit)
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
