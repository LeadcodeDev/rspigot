# rspigot

Spigot 26.3 compatible Minecraft server in Rust. Spec and workstream tracking: EPIC issue #11 (one sub-issue per crate). `docs/` is local only and never committed.

Layers: `rspigot-blocks`, `rspigot-data`, `rspigot-nbt` < `rspigot-domain` < `rspigot-core` (ports + use cases) < adapters (`rspigot-network`, `rspigot-anvil`, `rspigot-worldgen`, `rspigot-auth`) < `apps/rspigot` (composition root). `rspigot-protocol` is the wire format, used only by `rspigot-network`.

Blocks: one generated struct per block in `crates/rspigot-blocks/src/blocks/` (never edit by hand). Regenerate with `tools/block-dump/run.sh` then `python3 -I tools/gen_blocks.py`. Behaviour lives in hand-written families (`crates/rspigot-blocks/src/family/`), mapped from vanilla classes in `crates/rspigot-blocks/families.txt`.

Rules: no `async-trait`, no comments in code, static dispatch first, `thiserror` per crate, `anyhow` only in `apps/rspigot`. Stacked PRs, one per crate, in dependency order.

Verify: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
Run: `cargo run --release -p rspigot` (listens on 25565, files in the working directory).
