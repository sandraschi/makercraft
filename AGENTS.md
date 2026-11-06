# MakerCraft — AGENTS.md

Pure-Rust ArtCraft repo (`Cargo.toml` at root). Follow
`mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md` + `rust-artcraft/CONTAINMENT_AND_CRATES.md`.
Python/FastMCP/Starlette/port-claiming rules are INVALIDATED here.

## Seam

- L1 `makercraft-core`: G-code parser, safety linter, profiles, `Command::dispatch`. Never panics.
- L1 `makercraft-hal`: Klipper/GRBL/Bus-Pirate oracles (mock today, live bridges v2).
- L2 `makercraft-mcp`: CLI-first (`status|printer-status|cnc-status|bus-pins|lint|check|estimate|preview`) + `mcp` stdio. stdout = pure JSON-RPC, logs to stderr.
- L3 `makercraft-app`: `eframe/egui` bench calling core in-process. Never spawns the MCP exe, never opens an HTTP sidecar.

## Gates (every commit)

1. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
2. `cargo test --workspace` (every fix ships a regression test)
3. `cargo fmt --all --check`
4. Stdio probe: `initialize` -> `tools/list` -> `tools/call`, stdout 100% JSON-RPC
5. `Cargo.lock` committed; no `*.bak`, `*.exe`, `target/`, `dist/` in `git status`
6. `CHANGELOG.md` checkpoint line updated

## Safety

- Heaters/spindles/motion behind confirm gates; `check` exits 2 when unsafe (CI gate).
- Reference rig: Ender-3 Klipper 220x220x250 + 3018 GRBL 300x180x45 + Bus Pirate v4 @3.3V.
- No secrets in repo. Mock until hardware-in-the-loop, labeled as mock.

<!-- FLEET-CONTEXT:BEGIN -->
## Fleet context — required reading
Before any task, read `mcp-central-docs/standards/AGENT_READING_ORDER.md`.
Never author multi-line files with Bash heredocs (use Write).
Never patch with inline Python (use Edit/Write).
Never hardcode a port. Max 10 files per commit; 3+ edits need timestamped `.bak` first.
Never restart NSSM by killing the child. Never invent dates/locations.
Prove changes by running them.
<!-- FLEET-CONTEXT:END -->
