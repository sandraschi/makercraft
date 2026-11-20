# CHANGELOG — MakerCraft

## 2026-11-20 — v0.3.0: 3D Orbit/STL Viewer, Multi-Printer Profiles, Buying Matrices & Expanded Help

- **Interactive 3D Orbit & STL Viewer (`viewer3d.rs`, `makercraft-core`):** Added rotatable 3D build-volume viewer (`3D Orbit` / `2D XY` toggle), ASCII/binary `.stl` mesh parser + drag-and-drop `.gcode`/`.stl` loading, Z-layer scrubber, and `G2/G3` circular arc tessellation.
- **Expanded 3D Printer Suite:** Added Prusa MK4S, Bambu Lab X1C/P1S, and Voron 2.4 350 profiles alongside Ender-3 Klipper and 3018 GRBL CNC, plus PLA/PETG/ABS/TPU material presets and `E`-word filament length/weight/cost estimation.
- **Polish & Kanban:** Industrial Slate palette, G-code syntax highlighting (`LayoutJob`), telemetry sparklines, animated Workshop oscilloscope/LEDs + DUT PCB Bezier cables, and 1-click Kanban `Run` actions.
- **Device Acquisition Matrices & 10-Item Help Menu:** Interactive fit-scored matrices for 3D printers, display-less & benchtop oscilloscopes, logic analyzers, and desktop CNCs; expanded Help menu with `Getting Started...`, `Usage...`, `Tips & Tricks...`, `Common Pitfalls & Safety...`, `Planned Extensions...`, and `Bundled Skills (8)...`.

## 2026-11-18 — Tiers 4–7: Craft Menu Bar, Pictorial Studio, Skills & AI Chat, Vienna & Buying Guide

- **Tier 4 (Menu Bar & Help Suite):** Implemented the universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `egui::MenuBar` `switch_on_hover`, wired hotkeys (`Ctrl+K`, `Ctrl+J`, `F1`, `F5`, `Ctrl+N/O/S/Z/Y`, `Esc` E-stop), and the 5-item Help suite (`Command Palette...`, `Keyboard Shortcuts...`, `Project Files & Docs...`, `Licenses & Notices...`, `About MakerCraft`) backed by `HELP.md`.
- **Tier 5 (Pictorial Hardware & Workshop Studio):** Added `hardware_viz.rs` rendering vector technical diagrams via `egui::Painter` for all 9 devices (Ender-3 Printer, 3018 CNC, Oscilloscope, Logic Analyzer, Bus Pirate v4, Digital Multimeter, Function Generator, Spectrum Analyzer, JTAG/SWD Probe) across the `Machines` and `Instruments` tabs, plus an interactive **Workshop & ESD Workbench Panorama** (`Workshop` tab) with clickable stations and E-stop mushroom button wired to `Command::dispatch`.
- **Tier 6 (Bundled Skills & AI Chat Dock):** Added 8 `SKILL.md` guides under `skills/` embedded in `makercraft-core::skills`, exposed via MCP `resources/list` + `resources/read` (`skill://`), `makercraft_skill_read` (18 tools total), CLI `makercraft-mcp skills list|show|export`, and `Help -> Project Files & Docs -> Skills`. Added `chat_dock.rs` (`Ctrl+J`) for local-first Ollama / LM Studio / OpenAI-compatible chat with zero fake fallback text.
- **Tier 7 (Vienna Makerspaces & Buying Guide):** Added `reference_views.rs` and the `Vienna & Buying` tab covering Metalab, Happylab, and Maker Austria plus practical hardware buying advice.
- Verified with `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --offline` (40 tests green: 1 core unit + 15 core integration + 16 hal + 8 mcp), `cargo fmt --all`, and `makercraft.exe --smoke`.

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
