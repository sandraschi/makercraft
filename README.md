# MakerCraft

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange?logo=rust)
![MCP stdio](https://img.shields.io/badge/MCP-stdio-purple)
![ArtCraft harness](https://img.shields.io/badge/ArtCraft-harness-green)

## Install

Tell Cursor, Claude, Antigravity, or OpenCode: **Install this.**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-craft.ps1
```

Installs the Windows app and the CLI (Start Menu shortcut, CLI on the user PATH). `-RegisterClients` also registers the MCP exe with the IDEs that are installed. `-Clients cursor,claude-desktop` limits that list. `-Mcpb` installs the Claude Desktop extension bundle.

**MakerCraft** is an ArtCraft-tier local-first digital fabrication bench and instrument studio connecting 3D printers, CNC routers, and hardware protocol debuggers into a unified desktop and agentic interface.

Features:
- **Real G-code Parser**: Modal G0/G1, G90/G91 absolute/relative, G20/G21 units, G2/G3 arc notice, `;` + `(...)` comments.
- **Safety Linter with Teeth**: Envelope gates, rapid-plunge errors, depth limits, feedrate + nozzle-temp gates. `passes_safety` is real.
- **Machine Profiles**: 3018-class CNC (300x180x45, GRBL 1.1h) and Ender-3-class printer (220x220x250, Klipper) - the declared reference rig.
- **2D Toolpath Preview**: Top-down XY, feed vs rapid coloring, wheel zoom + drag pan, bed outline, start/end dots.
- **Klipper / Moonraker 3D Printing**: Direct feedrate, temperature, and kinematic monitoring.
- **GRBL CNC Router Control**: Toolhead coordinates, spindle speed, sub-zero plunge safety linting.
- **Bus Pirate Protocol Bridge**: Real-time pin logic level and pullup voltage monitoring.
- **Bench capture**: `makercraft-mcp instrument-capture` writes a labelled demo file for the oscilloscope, logic analyzer, multimeter, function generator, spectrum analyzer, JTAG/SWD adapter, and Bus Pirate. It does not drive hardware.
- **Pictorial Hardware & Workshop Studio**: Interactive **Workshop** panorama (`Workshop` tab) depicting the 2-bay Maker's Workshop (Fabrication Bench with Ender-3, 3018 CNC, HEPA vac, and clickable E-stop + ESD Electronics Workbench with pegboard, scope, function generator, spectrum analyzer, logic analyzer, Bus Pirate, DMM, and JTAG/SWD), plus individual device cards in **Machines** and **Instruments** wired to `Command::dispatch`.
- **Bundled Skills & Local-First AI Chat (`Ctrl+J`)**: 8 embedded `SKILL.md` guides (`workflow`, `klipper-printer`, `grbl-cnc`, `oscilloscope`, `logic-analyzer`, `bus-pirate`, `vienna-makerspaces`, `buying-guide`) served over MCP `resources/list`, `makercraft_skill_read`, CLI `skills`, and used as the preprompt for the in-app AI Chat Dock (Ollama / LM Studio / OpenAI-compatible).
- **Vienna Makerspaces & Buying Guide**: Dedicated **Vienna & Buying** tab covering Metalab, Happylab, and Maker Austria plus practical hardware buying advice.
- **Craft Menu Bar & 5-Item Help Suite**: Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `Command Palette...` (`Ctrl+K`), `Keyboard Shortcuts...` (`F1`), `Project Files & Docs...` (with Skills viewer & `HELP.md`), `Licenses & Notices...`, and `About MakerCraft`.
- **MCP**: `makercraft-mcp mcp` speaks JSON-RPC 2.0 on stdin and stdout (18 tools + `skill://` resources). The window label `14130` is not a listening server.

Reference rig (all lint bounds are tested against this):
`Ender-3 class Klipper/Moonraker 220x220x250 + 3018 class GRBL 1.1h 300x180x45 + Bus Pirate v4 @3.3V`

---

## Workspace Architecture

1. `crates/makercraft-hal`: Hardware abstraction interfaces for Klipper, GRBL, Bus Pirate, SCPI/USBTMC scopes, and sigrok logic analyzers.
2. `crates/makercraft-core`: Real G-code parser, safety linter, machine profiles, job estimation, and 8 embedded skills (`skills.rs`).
3. `crates/makercraft-mcp`: CLI plus the stdio MCP server (`makercraft-mcp`, 18 tools + `skill://` resources).
4. `crates/makercraft-app`: Native GUI (`eframe`/`egui`, tabs: Workshop / Preview / Safety / Machines / Instruments / G-code / Vienna & Buying, plus `Ctrl+J` AI Chat Dock) and CLI (`makercraft-cli`).

---

## Verification & Usage

```powershell
# Run test suite
cargo test

# Headless smoke test
./target/debug/makercraft.exe --smoke

# CLI Telemetry + lint (CNC default, --profile printer for Ender-3)
./target/debug/makercraft-cli.exe status
./target/debug/makercraft-cli.exe lint sample.gcode
./target/debug/makercraft-cli.exe check sample.gcode   # exit 2 when unsafe (CI gate)
./target/debug/makercraft-cli.exe estimate sample.gcode --profile printer
./target/debug/makercraft-mcp.exe instrument-sources
./target/debug/makercraft-mcp.exe instrument-capture oscilloscope --dir $env:TEMP\makercraft-captures
./target/debug/makercraft-mcp.exe skills list
./target/debug/makercraft-mcp.exe skills export --to .cursor\skills
```

## Status - 18 November 2026

**Does today:** G-code lint, printer and CNC probes, scope and logic probes, demo capture files for seven bench sources, Universal 8-menu bar + 5-item Help suite (`HELP.md`), pictorial vector diagrams for all 9 hardware devices, 8 bundled skills (`skill://` MCP resources + `makercraft_skill_read`), local-first AI Chat Dock (`Ctrl+J`), and the Vienna Makerspaces + Buying Guide tab.

**Still open:** live hardware readings inside the capture file for the meter, function generator, spectrum analyzer, and JTAG adapter; re-cutting the release NSIS installer after Tiers 4–7.

**Non-goals:** no hardware in CI. Heaters, spindles, and motion stay behind a confirm gate. `instrument-capture` does not drive an output or flash a chip.

## Lineage - the Storytold craft-app pattern

Storytold ArtCraft ([storytold/artcraft](https://github.com/storytold/artcraft) - "the IDE for artists") was the starting point. The 16 craft apps in the Storytold stable each ship one binary that is both the human CLI and the agent MCP server (stdio), headless-first, with `run.json` + report artifacts. That modus operandi and harness is what this repo implements; the code here is independent, the pattern is shared.
