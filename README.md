# MakerCraft

**MakerCraft** is an ArtCraft-tier sovereign digital fabrication bench and instrument studio connecting 3D printers, CNC routers, and hardware protocol debuggers into a unified desktop and agentic interface.

Features:
- **Klipper / Moonraker 3D Printing**: Direct feedrate, temperature, and kinematic monitoring.
- **GRBL CNC Router Control**: Toolhead coordinate tracking, spindle speed control, and sub-zero plunge safety linting.
- **Bus Pirate Protocol Bridge**: Real-time pin logic level and pullup voltage monitoring.
- **FastMCP Protocol**: Exposed locally on port `14130` for autonomous fabrication agents.

---

## Workspace Architecture

1. `crates/makercraft-hal`: Hardware abstraction interfaces for Klipper, GRBL, and Bus Pirate.
2. `crates/makercraft-core`: G-code parser, safety linter, and queue manager.
3. `crates/makercraft-mcp`: FastMCP JSON-RPC server on port `14130`.
4. `crates/makercraft-app`: Native GUI (`eframe`/`egui`) and CLI tool (`makercraft-cli`).

---

## Verification & Usage

```powershell
# Run test suite
cargo test

# Headless smoke test
./target/debug/makercraft.exe --smoke

# CLI Telemetry
./target/debug/makercraft-cli.exe status
./target/debug/makercraft-cli.exe lint sample.gcode
```
