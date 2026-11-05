# MakerCraft

**MakerCraft** is an ArtCraft-tier local-first digital fabrication bench and instrument studio connecting 3D printers, CNC routers, and hardware protocol debuggers into a unified desktop and agentic interface.

Features:
- **Real G-code Parser**: Modal G0/G1, G90/G91 absolute/relative, G20/G21 units, G2/G3 arc notice, `;` + `(...)` comments.
- **Safety Linter with Teeth**: Envelope gates, rapid-plunge errors, depth limits, feedrate + nozzle-temp gates. `passes_safety` is real.
- **Machine Profiles**: 3018-class CNC (300x180x45, GRBL 1.1h) and Ender-3-class printer (220x220x250, Klipper) — the declared reference rig.
- **2D Toolpath Preview**: Top-down XY, feed vs rapid coloring, wheel zoom + drag pan, bed outline, start/end dots.
- **Klipper / Moonraker 3D Printing**: Direct feedrate, temperature, and kinematic monitoring.
- **GRBL CNC Router Control**: Toolhead coordinates, spindle speed, sub-zero plunge safety linting.
- **Bus Pirate Protocol Bridge**: Real-time pin logic level and pullup voltage monitoring.
- **FastMCP Protocol**: Exposed locally on port `14130`.

Reference rig (all lint bounds are tested against this):
`Ender-3 class Klipper/Moonraker 220x220x250 + 3018 class GRBL 1.1h 300x180x45 + Bus Pirate v4 @3.3V`

---

## Workspace Architecture

1. `crates/makercraft-hal`: Hardware abstraction interfaces for Klipper, GRBL, and Bus Pirate.
2. `crates/makercraft-core`: Real G-code parser, safety linter, machine profiles, job estimation.
3. `crates/makercraft-mcp`: FastMCP JSON-RPC server on port `14130`.
4. `crates/makercraft-app`: Native GUI (`eframe`/`egui`, tabs: Preview / Safety / Machines / G-code) and CLI (`makercraft-cli`).

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
```

## Status — does today / next week / non-goals

**Does today:** 4-crate Rust workspace (HAL first); fabrication-bench
positioning (Klipper/GRBL/bus-probe); 9 test fns.

**Next week:** PRD (HAL traits, sim-first, safety/confirm gates, benchcraft
boundary); toolpath fixture tests (model → G-code); toolpath-sim recipe.

**Non-goals:** no hardware in CI (`sim` receipts); heaters/spindles/motion
behind confirm gates.

## Lineage — the Storytold craft-app pattern

Storytold ArtCraft ([storytold/artcraft](https://github.com/storytold/artcraft) — "the IDE for artists") was the starting point. The 16 craft apps in the Storytold stable each ship one binary that is both the human CLI and the agent MCP server (stdio), headless-first, with `run.json` + report artifacts. That modus operandi and harness is what this repo implements; the code here is independent, the pattern is shared.
