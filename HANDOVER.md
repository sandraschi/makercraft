---
updated: 2026-11-18T03:38:00+01:00
status: current
tags: [makercraft, bench, instruments, skills, chat, menu-bar]
---

# Handover — MakerCraft

## What Was Done
- **Tier 4:** Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `switch_on_hover`, wired actions/hotkeys, and 5-item Help suite + `HELP.md`.
- **Tier 5:** Pictorial vector diagrams (`hardware_viz.rs`) for all 9 machines & bench instruments on `Machines` and `Instruments` tabs, wired to `Command::dispatch` probes and `instrument_capture`.
- **Tier 6:** 8 bundled `SKILL.md` files embedded in `makercraft-core::skills`, served via MCP `resources/list` + `resources/read` (`skill://`), `makercraft_skill_read` (18 MCP tools), CLI `makercraft-mcp skills`, and `Ctrl+J` local-first AI Chat Dock (`chat_dock.rs`).
- **Tier 7:** `Vienna & Buying` tab (`reference_views.rs`) with Metalab, Happylab, Maker Austria, prep checklists, and machine/instrument buying advice.

## Last Concrete Action
- Updated `README.md`, `CHANGELOG.md`, `HANDOVER.md`, and `docs/PRD.md` on 18 November 2026.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --offline` (40 tests: 16 core, 16 hal, 8 mcp), and `makercraft.exe --smoke` all passed.

## Next Step
- Re-cut the Windows release binary and NSIS installer (`scripts/release.ps1`) if distributing updated `dist/` artifacts, and wire live HAL readings into `instrument_capture` for multimeter, function-generator, spectrum-analyzer, and jtag-swd when hardware is attached.
