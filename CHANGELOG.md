# CHANGELOG — MakerCraft

## 2026-11-06 — v2 live bridges (read-only) + bench instruments

- 5 new Commands/MCP tools (13 total): `printer.live` (Moonraker HTTP),
  `cnc.probe` (GRBL `?` serial), `bus.probe` (Bus Pirate identity),
  `scope.idn` (SCPI `*IDN?` over TCP — oscilloscopes/PSUs/DMMs),
  `logic.capture` (4-pin Demo pattern until SUMP v3).
- Every live call is sync with timeouts and Demo fallback (`"live": false`);
  `cargo test` needs no hardware (27 tests green: 13 core + 7 hal + 7 mcp).
- Env precedence: explicit arg > `MOONRAKER_URL`/`GRBL_PORT`/`BUS_PIRATE_PORT`/`SCOPE_ADDR` > Demo.
- No motion, no heaters, no pin drive — motion/heat/pin-drive stay confirm-gated non-goals.

## 2026-11-06 — Command bus + stdio MCP (Tier 1 closing)

- Workspace `deny(unwrap_used, expect_used, panic, todo, unimplemented)` walls on.
- L1 `Command::dispatch` (8 commands) — CLI, MCP, and GUI share one code path.
- L2 `makercraft-mcp` is CLI-first + hand-rolled MCP 2024-11-05 stdio; stdout 100% JSON-RPC.
- `gcode.check` exits 2 when unsafe (CI gate); stdio probe green (initialize/tools-list/tools-call).
- 18 tests green (12 core incl. dispatch parity + 6 mcp incl. underscore/dotted aliases).
- Dropped unused `databasecraft-*` deps from the workspace (leaner lockfile, faster build).
- Hygiene: `justfile`, `AGENTS.md`, `docs/PRD.md`, current-user `installer/installer.nsi`.
