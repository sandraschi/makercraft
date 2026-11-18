---
name: makercraft-klipper-printer
description: Operating Klipper and Moonraker 3D printers safely, checking temperatures, bed envelope, and first-layer diagnostics.
---

# Klipper / Moonraker 3D Printing

Reference profile: **Ender-3 class** (220 x 220 x 250 mm, Klipper + Moonraker HTTP API).

## Telemetry & Probing
- Use `printer_status` for deterministic reference-rig telemetry.
- Use `printer_live` with a Moonraker URL (for example `http://printer.local:7125`) to query live extruder temperature, heated bed temperature, print progress, and state. If unreachable, MakerCraft returns a labelled `"demo"` oracle without panicking.

## Safety Rules for FDM Printing
- **Bed envelope:** X and Y must stay within `0..220` mm and Z within `0..250` mm. Negative Z (`Z < 0`) is rejected as a hard error by `gcode_lint` on the `printer` profile.
- **Thermal sanity:** Watch nozzle targets (>260 C triggers a high-temp warning for PTFE-lined hotends) and verify bed thermistor readings before preheating.
- **First layer checklist:** Clean PEI/glass bed with IPA, verify Z-offset on a slow skirt (`F600` to `F1200`), and check cooling fan is `0%` on layer 1 for PLA/PETG.

## Common Klipper Tuning Notes
- **Pressure Advance (`SET_PRESSURE_ADVANCE`):** Typically `0.02-0.06` for direct drive, `0.25-0.65` for Bowden.
- **Input Shaper:** Run ADXL345 resonance testing after any belt tension or frame change; never exceed max_accel without checking stepper skipped-step margins.
- **Emergency stop:** In Klipper, `M112` halts immediately and requires `FIRMWARE_RESTART`.
