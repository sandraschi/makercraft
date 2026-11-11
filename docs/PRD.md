# MakerCraft PRD — Digital Fabrication Bench

## Problem

Hobby fabrication runs three disconnected worlds: Klipper/Moonraker printers,
GRBL CNC routers, and Bus Pirate logic probes. Each has its own sender, its own
units, its own foot-guns (rapid plunge, over-temp, envelope overrun). No single
local-first bench lints a toolpath *before* it touches stock.

## What it does

- **Parse:** real G-code — modal `G0/G1`, `G90/G91`, `G20/G21`, `G2/G3` arc
  notice, `;` + `(...)` comments. Bare coords repeat the modal motion.
- **Lint with teeth:** envelope gates per profile, rapid-plunge hard error,
  single-pass depth limit (CNC), below-bed error (printer), nozzle-temp
  ceiling, feedrate warnings. `passes_safety` is real; `check` exits 2 for CI.
- **Estimate:** travel distance + time from feedrates for job planning.
- **Preview:** 2D top-down XY (feed vs rapid), bed outline, start/end dots.
- **Machines:** Klipper (live via Moonraker or Demo) + GRBL (`?` probe or Demo)
  + Bus Pirate identity (or Demo) + SCPI `*IDN?` bench instruments
  + USB scopes via USBTMC (or Demo) + USB logic via sigrok-cli (or DOWN)
  + 4-pin logic sampler.
- **Agent surface:** 15 MCP tools over stdio, same `dispatch` as the CLI.

## Bench hardware lives here (no benchcraft repo)

One bench, one binary. USB oscilloscopes and USB analyzers are MakerCraft
HAL drivers, not a new repo: the Command bus scales by variant, and a second
repo would duplicate the GUI, CLI, MCP framing, and reference rig for zero
separation benefit. The `benchcraft` name is retired — if it ever returns,
it will be for something that is not a bench instrument (e.g. inventory).

Windows USB prerequisite: every USB instrument needs a WinUSB driver —
install with Zadig per device (Rigol 1ab1:04ce, Siglent f4ec:*). Without it
`rusb` cannot open the device and `scope.usb` says so in `"error"`.

## HAL traits (v2.1 — landed 2026-11-06)

## HAL traits (v2 — landed 2026-11-06)

- `printer.live`: Moonraker HTTP (`/server/info` + `/printer/info`, 3 s timeout,
  `MOONRAKER_URL` default `http://127.0.0.1:7125`) with Demo fallback. Toolhead
  XYZ stays the Demo oracle until `/printer/objects` lands (v2.1).
- `cnc.probe`: GRBL `?` status over serial 115200 8N1 (`GRBL_PORT`), read-only,
  Demo fallback. Never sends motion.
- `bus.probe`: Bus Pirate text-mode `i` identity (`BUS_PIRATE_PORT`), read-only,
  Demo fallback. Binary-mode ADC lands in v3.
- `scope.idn`: generic SCPI `*IDN?` over TCP — covers oscilloscopes, bench PSUs,
  DMMs with LAN (`SCOPE_ADDR=ip:5025`), Demo 8-sample sine when unreachable.
- `logic.capture`: 4-pin (MOSI/MISO/CLK/CS) pattern sampler, `hz`/`samples`
  clamped, `samples_source: demo` until SUMP firmware lands (v3).
- `scope.usb`: USBTMC `*IDN?` + best-effort Rigol `:WAV:PRE?`/`:WAV:DATA?`
  waveform in volts (IEEE 488.2 block parsed, preamble-scaled), Demo when the
  device is missing. Hand-rolled on `rusb`; the `usbtmc 0.1.0` crate is an
  empty stub and is deliberately NOT a dependency.
- `logic.usb`: sigrok-cli subprocess capture (`--driver`, `--config
  samplerate=`, `--samples`, `--output-format csv` parsed to channels).
  Missing binary = `engine: down` receipt with labeled Demo pattern, never a
  fake green. Empty driver = usage hint.

Bench hardware that is NOT in scope: Saleae/sigrok USB analyzers, USBTMC/GPIB
oscilloscope waveform fetch, spectrum analyzers. They want a `benchcraft`
neighbor repo; MakerCraft only speaks SCPI `*IDN?` until someone files that issue.

## Sim-first

No hardware in CI. All lint bounds tested against the reference rig.
Live Moonraker/serial bridges land behind `--confirm` and are never required
for `cargo test` to pass.

## Non-goals

- No heaters/spindles/motion without an explicit confirm gate.
- No cloud account, no slicer engine, no CAD modeling (see solidcraft).
- No database: job files stay files; no SQLite until a job queue is specified.
- No HTTP sidecar: the GUI calls core in-process; MCP is stdio only.

## Boundary

Neighbor `databasecraft` owns inventory/part catalogs; MakerCraft links to it,
never reimplements it. Neighbor `solidcraft` owns CAD; MakerCraft consumes
G-code, never models.

## UI/UX Architecture & Menu Bar Standard

In accordance with [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md):
- **Pattern 2 (Viewport-Centric Craft Workstation):** Persistent G-code / XY toolpath canvas and digital fabrication bench surrounded by machine monitors. Top quick button row acts as accelerators into the `View` menu. To add a new hardware monitor or tool:
  1. Register a visibility toggle in the `View` or `Window` menu.
  2. Add an entry in the sidebar Machine / Probe toolbox.
  3. Wire the action directly into the universal `Command::dispatch` bus.
- **Top Menu Bar Hierarchy:**
  - `File`: New Job (`Ctrl+N`), Open G-code / NC File (`Ctrl+O`), Open Recent ▸, Save Job (`Ctrl+S`), Save As... (`Ctrl+Shift+S`), Export Toolpath (`Ctrl+E`), Close Job (`Ctrl+W`), Quit (`Ctrl+Q`).
  - `Edit`: Undo (`Ctrl+Z`), Redo (`Ctrl+Y`), Cut, Copy, Paste, Select All, Preferences (`Ctrl+,`).
  - `View`: XY Toolpath Canvas, Machine Status HUD, G-code Terminal, Oscilloscope / Logic Analyzer, Density (`Compact`/`Standard`), Theme (`Dark`/`Light`).
  - `Insert`: Add Toolpath Layer, Insert Pause / Tool Change, Add Probe Point, Load Fixture Profile.
  - `Modify`: Scale / Shift Toolpath, Invert Z-Axis, Optimize Rapids, Adjust Feedrate Override.
  - `Commands`: Lint Toolpath (Safety Envelope), Run Dry Run / Simulation, Probe Machine Status, Home All Axes, Emergency Stop.
  - `Window`: Reset Layout, Toolpath Viewport, G-code Editor, Scope Telemetry, Machine Inspector.
  - `Help` (Mandatory 5-Dialog Suite): `Command Palette...` (`Ctrl+K`), `Keyboard Shortcuts...` (`F1`), `Project Files & Docs...`, `Licenses & Notices...`, `About MakerCraft`.
- **Behavioral Mechanics:** Smooth `switch_on_hover` between top-level menus, zero-unsafe immediate-mode rendering, and headless `--smoke` parity.
