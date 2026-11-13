# CHANGELOG — MakerCraft

## 2026-11-13 — Bench instruments write a capture file

- `instrument.sources` lists the seven repos folded into this app: oscilloscope, logic-analyzer, multimeter, function-generator, spectrum-analyzer, jtag-swd, bus-pirate.
- `instrument.capture` writes one JSON file. `demo` is true and `live` is false. A function-generator file records `output_driven: false`. A jtag-swd file records `flashed: false`. An unknown source writes nothing. The directory is `--dir`, otherwise `MAKERCRAFT_CAPTURE_DIR`, otherwise the temp folder `makercraft-captures`.
- CLI and MCP: `makercraft-mcp instrument-sources` and `makercraft-mcp instrument-capture <source>`. The catalog is 17 tools. `makercraft-cli` is unchanged.
- The Machines tab lists the seven sources and names the live command when one exists.
- The Python servers are not deleted. Live scope, logic, and Bus Pirate probes stay on `scope.usb`, `logic.usb`, and `bus.probe`.
- Recount, same method as the old 3,570 (physical lines in the `.rs` files): 3,796. tokei code lines: 3,399. `cargo test --workspace --offline` passed 38 (15 core, 16 hal, 7 mcp). `test_dispatch_status_lists_all_commands` expects 17.

## 2026-11-06 — GRBL probe hardened via grbl-mcp oracle

- `cnc.probe` now drains the DTR-reset boot banner before asking, normalizes
  `Hold:x`/`Door:x` sub-states, and reports `machine_pos`/`work_pos` plus
  `$I` firmware identity (best-effort, never fails the probe).
- Cross-checked against the `grbl-mcp 0.1.0` crate source (state table + MPos
  parsing adopted; its `anyhow`/`unwrap`/motion-command architecture NOT adopted).

- 2 new Commands/MCP tools (15 total): `scope.usb` (USBTMC `*IDN?` + Rigol
  waveform over `rusb`, Demo when missing) and `logic.usb` (sigrok-cli capture,
  `engine: down` receipt when the binary is absent — Goliath has none).
- Decision: USB scopes/analyzers live in MakerCraft HAL. No `benchcraft` repo:
  one bench, one binary; a split would duplicate GUI+CLI+MCP for no seam benefit.
- Windows USB = WinUSB via Zadig per device, else `scope.usb` names the fix.
- 36 tests green (14 core + 15 hal + 7 mcp); `usbtmc 0.1.0` rejected (empty stub).

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
