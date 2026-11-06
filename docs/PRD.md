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
- **Machines:** Klipper + GRBL + Bus Pirate telemetry (mock oracles today).
- **Agent surface:** 8 MCP tools over stdio, same `dispatch` as the CLI.

## HAL traits (v2 target)

- `KlipperBridge`: Moonraker HTTP (`printer.info`, `print.pause/cancel`, temp targets) + `Vendor::Demo` oracle.
- `GrblBridge`: serial `?` status + `$H`/`$X` interlock + `Vendor::Demo` oracle. No motion without confirm gate.
- `BusPirateBridge`: pin levels + pullups over serial; read-only, no drive.

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
