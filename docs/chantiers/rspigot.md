# Chantier rspigot

Stacked pull requests, one per crate, in dependency order. Each branch builds on the previous one.

| # | Branch | Crate | Status |
|---|--------|-------|--------|
| 1 | feat/data | rspigot-data | pending |
| 2 | feat/nbt | rspigot-nbt | pending |
| 3 | feat/domain | rspigot-domain | pending |
| 4 | feat/protocol | rspigot-protocol | pending |
| 5 | feat/core | rspigot-core | pending |
| 6 | feat/anvil | rspigot-anvil | pending |
| 7 | feat/worldgen | rspigot-worldgen | pending |
| 8 | feat/auth | rspigot-auth | pending |
| 9 | feat/network | rspigot-network | pending |
| 10 | feat/server | apps/rspigot | pending |

Convergence points owned by the orchestrator: root `Cargo.toml`, `CLAUDE.md`, `docs/`.
Regenerated artifacts: `crates/rspigot-data/src/generated/*` from `tools/gen_data.py` over the 26.3 reports.

Decision log
  - Ports are synchronous; the game loop is a plain thread and adapters talk to it through channels. Rejected: async ports, which would need boxing or async-trait to stay object safe and add nothing for a single owner thread.
  - The protocol crate holds wire DTOs only; mapping to domain types lives in the network adapter, so the domain never sees packet types.
  - Registry entries are sent with the `minecraft:core` known pack, so the client fills registry data from its own copy and the server sends names only.
