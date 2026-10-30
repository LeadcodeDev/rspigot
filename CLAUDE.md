# rspigot

Spigot 26.3 compatible Minecraft server in Rust. Spec and workstream tracking: EPIC issue #11 (one sub-issue per crate). `docs/` is local only and never committed.

Layers: `rspigot-blocks`, `rspigot-items`, `rspigot-effects`, `rspigot-entities`, `rspigot-data`, `rspigot-nbt` < `rspigot-domain` < `rspigot-core` (ports + use cases) < adapters (`rspigot-network`, `rspigot-anvil`, `rspigot-worldgen`, `rspigot-auth`) < `apps/rspigot` (composition root). `rspigot-protocol` is the wire format, used only by `rspigot-network`.

Game objects: one generated struct per block, item, mob effect and entity type (`crates/rspigot-{blocks,items,effects,entities}`), never edited by hand. Regenerate with `tools/<kind>-dump/run.sh` then `python3 -I tools/gen_<kinds>.py` (block, item, effect, entity). Behaviour lives in hand-written families (`src/family/`), mapped from vanilla classes in each crate's `families.txt`. Loot tables and registries: `python3 -I tools/gen_data.py`.

Rules: no `async-trait`, no comments in code, static dispatch first, `thiserror` per crate, `anyhow` only in `apps/rspigot`. Stacked PRs, one per crate, in dependency order.

Verify: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
Run: `cargo run --release -p rspigot` (listens on 25565, files in the working directory).
