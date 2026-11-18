# MakerCraft — Tiered Execution Plan

**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  
**Architecture Model:** 4-Tier ArtCraft Incremental Delivery  
**Status:** Active  

---

## Architecture Flow

```
Tier 1: Core Engine ──> Tier 2: MCP / CLI ──> Tier 3: Desktop UI ──> Tier 4: Menu Bar, Help & Packaging
(makercraft-core)       (makercraft-mcp)       (makercraft-app)       (Hardened Workstation)
```

---

## Tier Specifications

### Tier 1 — Core Toolpath & Machine Engine (`makercraft-core`)
- [x] G-code modal parser (`G0/G1`, `G90/G91`, `G20/G21`, `G2/G3`).
- [x] Deterministic safety envelope linter (rapid plunge, CNC depth, over-temp).
- [x] Travel distance and time estimator.
- [x] Bench HAL drivers: Moonraker HTTP, GRBL serial, Bus Pirate, USBTMC scope, sigrok logic.
- [x] `#![forbid(unsafe_code)]` and zero-panic clippy lints.

### Tier 2 — Headless Driver & stdio MCP (`makercraft-mcp`)
- [x] Unified CLI `makercraft-cli` with `--json` output.
- [x] Stdio server communicating over JSON-RPC 2.0 (17 tools, including `instrument.sources` and `instrument.capture`).
- [x] Hardware simulation and demo fallback testing without real benches in CI.
- [x] Headless `--smoke` verification without window initialization.

### Tier 3 — Desktop Workstation UI (`makercraft-app`)
- [x] Immediate-mode `egui` 0.37 / `wgpu` workstation.
- [x] 2D XY toolpath visualizer (feed vs rapid, bed outline, start/end).
- [x] Hardware status cards, scope waveform viewer, and logic trace HUD.
- [x] Machines tab lists the seven bench sources. The capture command is the CLI, not a button.

### Tier 4 — Craft Menu Bar Standard, Help Suite & Hardened Verification
- [x] Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `MenuBar` `switch_on_hover`.
- [x] Mandatory 5-dialog Help suite (`Command Palette...`, `Keyboard Shortcuts...`, `Project Files & Docs...`, `Licenses & Notices...`, `About MakerCraft`) + `HELP.md`.
- [x] Headless `--smoke` parity validation.
- [ ] Standalone release binary packaging and NSIS installer (re-cut after Tier 7).

### Tier 5 — Pictorial Hardware & Instrument Studio (`hardware_viz.rs`)
- [x] Vector technical diagrams rendered via `egui::Painter` for Ender-3 Printer, 3018 CNC, Oscilloscope, Logic Analyzer, Bus Pirate v4, Digital Multimeter, Function Generator, Spectrum Analyzer, and JTAG/SWD Probe.
- [x] Interactive `Machines` and `Instruments` tabs wired to `Command::dispatch` (`printer_live`, `cnc_probe`, `bus_probe`, `scope_idn`, `logic_capture`, `instrument_capture`).

### Tier 6 — Bundled Skills (`skills/`) & Local-First AI Chat Dock (`Ctrl+J`)
- [x] 8 embedded `SKILL.md` guides compiled into `makercraft-core::skills` (`workflow`, `klipper-printer`, `grbl-cnc`, `oscilloscope`, `logic-analyzer`, `bus-pirate`, `vienna-makerspaces`, `buying-guide`).
- [x] Served over MCP `resources/list` + `resources/read` (`skill://`), `makercraft_skill_read`, CLI `makercraft-mcp skills`, and `Help -> Project Files & Docs -> Skills`.
- [x] Local-first AI Chat Dock (`Ctrl+J`) supporting Ollama, LM Studio, and OpenAI-compatible endpoints with zero fake fallback text.

### Tier 7 — Vienna Makerspaces Reference & Machine Buying Guide
- [x] Dedicated `Vienna & Buying` tab with verified Vienna makerspaces (Metalab, Happylab, Maker Austria), prep checklist, and hardware buying guide.
