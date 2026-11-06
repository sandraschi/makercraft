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
  + Bus Pirate identity (or Demo) + SCPI `*IDN?` bench instruments + 4-pin logic sampler.
- **Agent surface:** 13 MCP tools over stdio, same `dispatch` as the CLI.

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
