---
name: makercraft-workflow
description: Core MakerCraft workflow for G-code parsing, safety linting, 2D toolpath preview, and CLI/MCP bench operations.
---

# MakerCraft Core Workflow

MakerCraft is a local-first digital fabrication bench and instrument studio. One `Command::dispatch` engine powers the GUI (`makercraft`), CLI (`makercraft-cli` / `makercraft-mcp`), and MCP stdio server.

## Daily Workflow
1. Select a machine profile: **3018 CNC** (300x180x45 mm, GRBL 1.1h) or **Ender-3 Printer** (220x220x250 mm, Klipper/Moonraker).
2. Load or paste G-code in the **G-code** tab (or `File -> Open G-code...`).
3. Run `gcode_lint` (or press `F5` / `Analyze Toolpath Safety`) before sending any job to hardware.
4. Inspect the **2D Preview** tab: blue/teal lines are feed moves (`G1`), dashed/gold lines are rapids (`G0`).
5. Check `gcode_estimate` for total travel distance (mm) and estimated runtime (min).

## Safety Gates
- `gcode_check` exits with code `2` when `passes_safety` is `false` (use in CI or pre-flight scripts).
- Hard errors block execution: empty program, X/Y/Z envelope violations, negative-Z printer moves, and rapid plunges (`G0` downward below safe Z).
- Heaters, spindles, and motion stay behind confirm gates. Use `Emergency Stop` (`Esc` or `Machines -> Emergency Kill All Spindles`) to latch E-stop in the UI.

## MCP & CLI Commands
- `status`: bench capabilities and available commands.
- `gcode_lint`, `gcode_check`, `gcode_estimate`, `gcode_preview`: toolpath verification.
- `instrument_sources`: list the 7 folded bench instruments.
- `instrument_capture`: write a labelled demo capture JSON (`"demo": true`).
- `makercraft_skill_read`: read any bundled MakerCraft skill.
