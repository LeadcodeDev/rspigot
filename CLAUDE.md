# rspigot

Spigot 26.3 compatible Minecraft server in Rust. Spec: `docs/specs/rspigot.md`. Workstreams: `docs/chantiers/rspigot.md`.

Layers: `rspigot-data`, `rspigot-nbt` < `rspigot-domain` < `rspigot-core` (ports + use cases) < adapters (`rspigot-network`, `rspigot-anvil`, `rspigot-worldgen`, `rspigot-auth`) < `apps/rspigot` (composition root). `rspigot-protocol` is the wire format, used only by `rspigot-network`.

Rules: no `async-trait`, no comments in code, static dispatch first, `thiserror` per crate, `anyhow` only in `apps/rspigot`.

Verify: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
Run: `cargo run --release -p rspigot` (listens on 25565, files in the working directory).
