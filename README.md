# MakerCraft

Local-first digital fabrication bench and electronics instrument studio connecting 3D printers, CNC routers, and 7 bench instruments into one pure-Rust desktop app, CLI, and MCP server.

![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Rust 1.90+](https://img.shields.io/badge/rust-1.90%2B-orange?logo=rust)
![MCP stdio](https://img.shields.io/badge/MCP-stdio-purple)
![ArtCraft harness](https://img.shields.io/badge/ArtCraft-harness-green)

## What You Can Do

- **3D Orbit & STL Viewer (`Preview` tab)**: Drag-to-orbit 3D build-volume viewer (`3D Orbit` / `2D XY`), Z-layer scrubber, and ASCII/binary `.stl` part preview (drag-and-drop `.gcode` or `.stl` directly onto the window).
- **Real G-code Parser & Safety Linter (`makercraft-core`)**: Modal `G0/G1`, `G2/G3` circular arc tessellation, `G90/G91`, `G20/G21`, envelope gates, rapid-plunge hard errors, depth limits, and feedrate/nozzle-temp checks (`gcode_check` exits `2` for CI).
- **Multi-Printer & CNC Profiles**: `3018 CNC` (300x180x45 mm, GRBL 1.1h), `Ender-3` (Klipper), `Prusa MK4S`, `Bambu X1C/P1S`, and `Voron 2.4` (350x350x330 mm), plus PLA/PETG/ABS/TPU presets and filament weight/cost estimation.
- **Interactive Workshop Panorama (`Workshop` tab)**: Clickable 2-bay workshop (Digital Fabrication Bench + ESD Electronics Workbench) with live E-stop mushroom button and per-station probing.
- **7 Consolidated Bench Instruments (`Instruments` tab)**: Oscilloscope (`scope_idn`, `scope_usb`), Logic Analyzer (`logic_capture`, `logic_usb`), Bus Pirate v4 (`bus_probe`, `bus_pins`), Multimeter, Function Generator, Spectrum Analyzer, and JTAG/SWD (`instrument_capture` demo JSON).
- **Dashboard, Handholding & Project Kanban**: Readiness KPIs, step-by-step CNC/Printer/Bench guides, SQLite project Kanban, and **Device Acquisition Decision Matrices** (including display-less USB oscilloscopes) + **Vienna Makerspaces** guide (`Metalab`, `Happylab`, `Maker Austria`).
- **8 Bundled Skills & Local-First AI Chat (`Ctrl+J`)**: Embedded `SKILL.md` guides served over MCP `resources/list` + `makercraft_skill_read`, CLI `skills`, and used as the preprompt for the in-app Ollama/LM Studio/OpenAI-compatible chat dock.

## Install

1. **Windows Installer (Recommended):** Download `makercraft-0.3.0-setup.exe` from [GitHub Releases](https://github.com/sandraschi/makercraft/releases) (per-user install to `%LOCALAPPDATA%\Programs\MakerCraft`, Start Menu shortcuts, clean uninstaller).
2. **Portable / MCP Bundle:** Grab `makercraft-0.3.0-windows-x64-portable.zip` or `makercraft-v0.3.0.mcpb` from Releases, or run the local helper script:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-craft.ps1
```

Installs the Windows app and the CLI (Start Menu shortcut, CLI on the user PATH). `-RegisterClients` also registers the MCP exe with the IDEs that are installed. `-Clients cursor,claude-desktop` limits that list. `-Mcpb` installs the Claude Desktop extension bundle.

## Workspace Architecture

1. `crates/makercraft-hal`: Moonraker HTTP, GRBL serial, Bus Pirate, SCPI/USBTMC scopes, and sigrok logic analyzers.
2. `crates/makercraft-core`: G-code parser/arc tessellator, safety linter, STL parser, filament estimator, and 8 embedded skills.
3. `crates/makercraft-mcp`: CLI + MCP `2024-11-05` stdio server (`18` tools + `skill://` resources).
4. `crates/makercraft-app`: Native `egui 0.37` workstation (`makercraft.exe`) and CI CLI (`makercraft-cli.exe`).

## Verification & CLI

```powershell
cargo test --workspace --offline
makercraft.exe --smoke
makercraft-cli.exe check sample.gcode --profile cnc
makercraft-mcp.exe skills list
```

## Docs

- [HELP.md](HELP.md) — User manual (Get Started, Functions, Usage, Tips, Pitfalls & Safety, Planned Extensions)
- [docs/PRD.md](docs/PRD.md) | [docs/TIERED_PLAN.md](docs/TIERED_PLAN.md) | [CHANGELOG.md](CHANGELOG.md)
