---
name: makercraft-grbl-cnc
description: Operating 3018-class GRBL 1.1h CNC routers, work coordinates, plunge safety, spindle RPM, and endmill selection.
---

# GRBL 1.1h CNC Milling

Reference profile: **3018-class CNC** (300 x 180 x 45 mm, GRBL 1.1h over USB serial).

## Probing & Status
- Use `cnc_status` for baseline telemetry or `cnc_probe` (with serial port such as `COM4` or `/dev/ttyUSB0`) to send GRBL's non-blocking `?` status query.
- Never jog or start a spindle without confirming work zero (`G54` / `G92 X0 Y0 Z0`) and clamp clearance.

## Toolpath Safety Rules
- **No rapid plunges:** `G0` downward into material (`Z < 0`) is a hard error in `gcode_lint`. Always rapid (`G0`) to a safe clearance height (e.g. `Z5.0` or `Z25.0`), then plunge with `G1 Z-1.0 F150..F300`.
- **Envelope & depth:** Keep XY inside `0..300` x `0..180` mm and Z within `-45..45` mm (or your spoilboard-adjusted limit).
- **Spindle & feed:** Match RPM (`M3 S10000`) and feedrate to cutter diameter and material:
  - Softwood / MDF (3.175 mm 2-flute): 10000-12000 RPM, 600-1000 mm/min, DOC 1.0-1.5 mm.
  - Acrylic (single-flute O-flute): 10000 RPM, 500-800 mm/min, DOC 0.5-1.0 mm (avoid melting).
  - FR4 PCB engraving (V-bit): 10000+ RPM, 150-300 mm/min, DOC 0.05-0.10 mm with height-map probing.

## Physical Safety
- Wear eye protection and use dust extraction (especially for MDF and FR4 fiberglass dust).
- Double-check ER11 collet tightness and nut seating (snap collet into nut before threading).
