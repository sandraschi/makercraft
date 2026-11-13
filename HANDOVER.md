---
updated: 2026-11-13T08:05:00+01:00
status: current
tags: [makercraft, bench, instruments, capture]
---

# Handover — MakerCraft

## What Was Done
- `instrument.sources` lists oscilloscope, logic-analyzer, multimeter, function-generator, spectrum-analyzer, jtag-swd, and bus-pirate.
- `instrument.capture` writes one JSON file per source. `demo` is true and `live` is false. A function-generator file records `output_driven: false`. A jtag-swd file records `flashed: false`. An unknown source writes nothing.
- The command is on `makercraft-mcp` (`instrument-sources`, `instrument-capture`). It is not on `makercraft-cli`.
- The Machines tab lists the seven sources. Scope, logic, and Bus Pirate still have their own live commands. The Python servers are still on disk.

## Last Concrete Action
- Recounted on 13 November 2026. Nine `.rs` files are 3,796 physical lines. tokei reports 3,399 Rust code lines. `cargo test --workspace --offline` passed 38 tests (15 core, 16 hal, 7 mcp). The status test now expects 17 commands.

## Next Step
- Put a live reading into the capture file for one of the four sources that are files only: multimeter, function-generator, spectrum-analyzer, jtag-swd. Until that reading exists, the file stays a demo.
