# MakerCraft Help

## 1. Get Started
1. **Launch MakerCraft** (`makercraft.exe`): opens on the **2D Preview** tab with the default 3018 CNC square toolpath and safety lint report.
2. **Choose your machine profile** in the top tab bar (`3018 CNC` or `Ender-3`) or via `Modify -> Switch Profile`.
3. **Load or edit G-code** in the **G-code** tab, or use `File -> Open G-code...` (`Ctrl+O`) / `Insert -> Insert Sample Square`.
4. **Lint & verify safety** with `Commands -> Run Safety Lint` (`F5`) and inspect the **Safety Lint** tab for hard errors vs. warnings.
5. **Inspect hardware & instruments** in the **Machines** and **Instruments** tabs (with pictorial diagrams and live/demo telemetry), or press `Ctrl+J` to open the skill-grounded **AI Chat Dock**.

## 2. Functions
- **G-code Modal Parser (`makercraft-core`)**: Parses `G0`/`G1`, `G90`/`G91` absolute/relative, `G20`/`G21` inch/mm, `G2`/`G3` arc notices, and `;` / `(...)` comments.
- **Deterministic Safety Linter**: Enforces X/Y/Z bed envelopes, rejects rapid plunges (`G0` downward into material), blocks negative-Z moves on 3D printers, and checks feedrate/temperature bounds.
- **Machine & Bench HAL (`makercraft-hal`)**: Moonraker HTTP (`printer_live`), GRBL serial (`cnc_probe`), Bus Pirate (`bus_probe`), SCPI/USBTMC oscilloscope (`scope_idn`, `scope_usb`), sigrok/SUMP logic analyzer (`logic_usb`, `logic_capture`), plus 7 `instrument_capture` bench sources.
- **Pictorial Hardware Studio**: Vector technical illustrations for printers, CNCs, oscilloscopes, logic analyzers, Bus Pirate, multimeters, function generators, spectrum analyzers, and JTAG/SWD probes.
- **Bundled Skills & AI Chat**: 8 embedded `SKILL.md` guides served over MCP `resources/list` + `makercraft_skill_read`, and used as the preprompt for the local-first `Ctrl+J` Chat Dock.

## 3. Usage (GUI, CLI & MCP)
- **GUI (`makercraft`)**:
  - `Ctrl+K` / `Ctrl+Shift+P`: Command Palette
  - `Ctrl+J`: Toggle AI Chat Dock
  - `F1` / `Ctrl+/`: Keyboard Shortcuts
  - `F5`: Run Safety Lint
  - `Esc`: Emergency Stop (latches E-stop)
- **CLI (`makercraft-cli` / `makercraft-mcp`)**:
  - `makercraft-cli status`
  - `makercraft-cli lint sample.gcode --profile printer`
  - `makercraft-cli check sample.gcode` (exits `2` when unsafe)
  - `makercraft-mcp instrument-sources`
  - `makercraft-mcp instrument-capture oscilloscope --dir $env:TEMP\makercraft-captures`
  - `makercraft-mcp skills list` / `show <name>` / `export --to <dir>`
- **MCP (`makercraft-mcp mcp`)**:
  - Speaks JSON-RPC 2.0 on stdin/stdout (`tools/list`, `tools/call`, `resources/list`, `resources/read`).

## 4. Tips & Tricks
- Always run `gcode_check` in CI or before loading an SD card / Moonraker upload.
- On the **2D Preview** canvas, scroll the mouse wheel to zoom, drag to pan, and press `Ctrl+0` to fit the bed.
- Use the **Vienna & Buying** tab (or ask the `Ctrl+J` chat) for Vienna makerspace prep checklists (Metalab, Happylab, Maker Austria) and instrument buying advice.
- Never connect an earth-referenced oscilloscope ground clip to a non-ground-referenced node.

## 5. Other Crafts
| Craft App | Role & Handoff | Boundary |
|---|---|---|
| **CADCraft** | Parametric 3D CAD & DXF/STEP/STL geometry | Exports models/toolpaths to slice or mill in MakerCraft |
| **RobotCraft** | URDF kinematics, Yahboom/VLA robotics | Robotics actuators live in RobotCraft, not MakerCraft |
| **DevCraft** | Fleet orchestration, ports, git, worktrees | Manages repo builds; MakerCraft handles bench hardware |
| **RevCraft** | Binary reverse engineering & firmware | Inspects firmware binaries; MakerCraft probes hardware buses |
