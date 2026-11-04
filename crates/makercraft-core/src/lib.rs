//! MakerCraft Core Engine
//!
//! Real G-code parsing, collision detection, toolpath safety linting,
//! and digital manufacturing job estimation.
//!
//! Reference rig (everything is linted against this unless a custom
//! profile is supplied):
//! - 3D printer: Ender-3 class, Klipper/Moonraker, 220 x 220 x 250 mm
//! - CNC router: 3018 class, GRBL 1.1h, 300 x 180 x 45 mm, 20k RPM spindle
//! - Logic: Bus Pirate v4, 3.3 V pullups

pub mod i18n;

use makercraft_hal::{BusPiratePinState, HardwareAdapter, MachineStatus};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Documented reference rig. Declare it so hardware behavior stays
/// testable instead of theoretical (fleet KIBITZ rule).
pub const REFERENCE_RIG: &str = "Ender-3 class Klipper/Moonraker 220x220x250 + 3018 class GRBL 1.1h 300x180x45 + Bus Pirate v4 @3.3V";

#[derive(Error, Debug)]
pub enum MakerError {
    #[error("Safety violation: {0}")]
    Safety(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileKind {
    Printer,
    Cnc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineProfile {
    pub name: String,
    pub kind: ProfileKind,
    /// Work envelope in mm.
    pub bed_x_mm: f64,
    pub bed_y_mm: f64,
    pub max_z_mm: f64,
    pub max_feedrate_mm_min: f64,
    /// Rapids at or above this Z are assumed crash-free.
    pub safe_z_mm: f64,
    /// CNC only: deepest allowed cut below stock top (positive mm).
    pub max_cut_depth_mm: f64,
    /// Printer only: hotend safety ceiling.
    pub max_nozzle_temp_c: f64,
}

impl MachineProfile {
    pub fn default_printer() -> Self {
        Self {
            name: "Ender-3 class (Klipper)".to_string(),
            kind: ProfileKind::Printer,
            bed_x_mm: 220.0,
            bed_y_mm: 220.0,
            max_z_mm: 250.0,
            max_feedrate_mm_min: 6000.0,
            safe_z_mm: 10.0,
            max_cut_depth_mm: 0.0,
            max_nozzle_temp_c: 300.0,
        }
    }

    pub fn default_cnc() -> Self {
        Self {
            name: "3018 class (GRBL 1.1h)".to_string(),
            kind: ProfileKind::Cnc,
            bed_x_mm: 300.0,
            bed_y_mm: 180.0,
            max_z_mm: 45.0,
            max_feedrate_mm_min: 2000.0,
            safe_z_mm: 5.0,
            max_cut_depth_mm: 12.0,
            max_nozzle_temp_c: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathSegment {
    pub line_no: usize,
    pub rapid: bool,
    pub from_x_mm: f64,
    pub from_y_mm: f64,
    pub from_z_mm: f64,
    pub to_x_mm: f64,
    pub to_y_mm: f64,
    pub to_z_mm: f64,
    pub feed_mm_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedToolpath {
    pub segments: Vec<ToolpathSegment>,
    pub min_x_mm: f64,
    pub max_x_mm: f64,
    pub min_y_mm: f64,
    pub max_y_mm: f64,
    pub min_z_mm: f64,
    pub max_z_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathLintReport {
    pub total_lines: usize,
    pub travel_distance_mm: f64,
    pub max_z_mm: f64,
    pub min_z_mm: f64,
    pub passes_safety: bool,
    pub warnings: Vec<String>,
    /// Hard failures (out of envelope, rapid plunge, over-temp).
    pub errors: Vec<String>,
    pub estimated_minutes: f64,
    pub segment_count: usize,
}

/// Parse G-code into motion segments. Handles `;` and `(...)` comments,
/// G90/G91 absolute/relative, G20/G21 units, modal G0/G1, G2/G3 arcs
/// (approximated as straight moves with a warning flag via `arc`).
fn parse_toolpath(gcode: &str) -> Result<(ParsedToolpath, Vec<String>, bool, bool), MakerError> {
    let mut segments = Vec::new();
    let mut notes = Vec::new();
    let mut absolute = true;
    let mut seen_units = false;
    let mut inches = false;
    let mut seen_motion = false;
    let mut last_motion_rapid = false;
    let mut pos = (0.0_f64, 0.0_f64, 0.0_f64);
    let mut feed = 1200.0_f64;
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;

    for (idx, raw) in gcode.lines().enumerate() {
        let line_no = idx + 1;
        // Strip ; comments and (...) comments.
        let mut line = raw.split(';').next().unwrap_or("").to_string();
        while let Some(s) = line.find('(') {
            if let Some(e) = line[s..].find(')') {
                line.replace_range(s..s + e + 1, " ");
            } else {
                line.truncate(s);
                break;
            }
        }
        let line = line.trim().to_uppercase();
        if line.is_empty() {
            continue;
        }
        let mut code: Option<(char, i32)> = None;
        let mut params: Vec<(char, f64)> = Vec::new();
        for tok in line.split_whitespace() {
            let (head, tail) = tok.split_at(1.min(tok.len()));
            let h = head
                .chars()
                .next()
                .ok_or_else(|| MakerError::Parse(format!("line {line_no}: empty token")))?;
            if !h.is_ascii_alphabetic() {
                return Err(MakerError::Parse(format!(
                    "line {line_no}: bad token '{tok}'"
                )));
            }
            let v: f64 = tail
                .parse()
                .map_err(|_| MakerError::Parse(format!("line {line_no}: bad number '{tok}'")))?;
            if code.is_none() && (h == 'G' || h == 'M') {
                code = Some((h, v as i32));
            } else {
                params.push((h, v));
            }
        }
        let param = |p: char| params.iter().find(|(k, _)| *k == p).map(|(_, v)| *v);
        if let Some(('G', n)) = code {
            match n {
                90 => absolute = true,
                91 => absolute = false,
                20 => {
                    inches = true;
                    seen_units = true;
                }
                21 => {
                    inches = false;
                    seen_units = true;
                }
                0 | 1 => {
                    seen_motion = true;
                    last_motion_rapid = n == 0;
                    let unit = if inches { 25.4 } else { 1.0 };
                    let nx = param('X').map(|v| v * unit);
                    let ny = param('Y').map(|v| v * unit);
                    let nz = param('Z').map(|v| v * unit);
                    if let Some(f) = param('F') {
                        feed = f * unit;
                    }
                    if nx.is_none() && ny.is_none() && nz.is_none() {
                        continue;
                    }
                    let target = (
                        nx.map(|v| if absolute { v } else { pos.0 + v })
                            .unwrap_or(pos.0),
                        ny.map(|v| if absolute { v } else { pos.1 + v })
                            .unwrap_or(pos.1),
                        nz.map(|v| if absolute { v } else { pos.2 + v })
                            .unwrap_or(pos.2),
                    );
                    segments.push(ToolpathSegment {
                        line_no,
                        rapid: n == 0,
                        from_x_mm: pos.0,
                        from_y_mm: pos.1,
                        from_z_mm: pos.2,
                        to_x_mm: target.0,
                        to_y_mm: target.1,
                        to_z_mm: target.2,
                        feed_mm_min: feed,
                    });
                    let (tx, ty, tz) = target;
                    min_x = min_x.min(tx);
                    max_x = max_x.max(tx);
                    min_y = min_y.min(ty);
                    max_y = max_y.max(ty);
                    min_z = min_z.min(tz);
                    max_z = max_z.max(tz);
                    pos = target;
                }
                2 | 3 => {
                    // Arc: approximate endpoint like a linear move, warn once.
                    seen_motion = true;
                    let unit = if inches { 25.4 } else { 1.0 };
                    let nx = param('X').map(|v| v * unit).unwrap_or(pos.0);
                    let ny = param('Y').map(|v| v * unit).unwrap_or(pos.1);
                    segments.push(ToolpathSegment {
                        line_no,
                        rapid: false,
                        from_x_mm: pos.0,
                        from_y_mm: pos.1,
                        from_z_mm: pos.2,
                        to_x_mm: nx,
                        to_y_mm: ny,
                        to_z_mm: pos.2,
                        feed_mm_min: feed,
                    });
                    notes.push(format!(
                        "line {line_no}: G{n} arc approximated as straight move"
                    ));
                    pos.0 = nx;
                    pos.1 = ny;
                }
                _ => {}
            }
        } else if nx_yz_only(&params) && seen_motion {
            // Modal motion: bare coordinates repeat the last G0/G1.
            let unit = if inches { 25.4 } else { 1.0 };
            let nx = param('X').map(|v| v * unit);
            let ny = param('Y').map(|v| v * unit);
            let nz = param('Z').map(|v| v * unit);
            if let Some(f) = param('F') {
                feed = f * unit;
            }
            let target = (
                nx.map(|v| if absolute { v } else { pos.0 + v })
                    .unwrap_or(pos.0),
                ny.map(|v| if absolute { v } else { pos.1 + v })
                    .unwrap_or(pos.1),
                nz.map(|v| if absolute { v } else { pos.2 + v })
                    .unwrap_or(pos.2),
            );
            segments.push(ToolpathSegment {
                line_no,
                rapid: last_motion_rapid,
                from_x_mm: pos.0,
                from_y_mm: pos.1,
                from_z_mm: pos.2,
                to_x_mm: target.0,
                to_y_mm: target.1,
                to_z_mm: target.2,
                feed_mm_min: feed,
            });
            min_x = min_x.min(target.0);
            max_x = max_x.max(target.0);
            min_y = min_y.min(target.1);
            max_y = max_y.max(target.1);
            min_z = min_z.min(target.2);
            max_z = max_z.max(target.2);
            pos = target;
        }
    }

    if segments.is_empty() {
        min_x = 0.0;
        max_x = 0.0;
        min_y = 0.0;
        max_y = 0.0;
        min_z = 0.0;
        max_z = 0.0;
    }
    Ok((
        ParsedToolpath {
            segments,
            min_x_mm: min_x,
            max_x_mm: max_x,
            min_y_mm: min_y,
            max_y_mm: max_y,
            min_z_mm: min_z,
            max_z_mm: max_z,
        },
        notes,
        seen_units,
        inches,
    ))
}

fn nx_yz_only(params: &[(char, f64)]) -> bool {
    !params.is_empty()
        && params
            .iter()
            .all(|(k, _)| matches!(*k, 'X' | 'Y' | 'Z' | 'F'))
}

pub struct MakerEngine;

impl MakerEngine {
    pub fn get_printer_status() -> MachineStatus {
        HardwareAdapter::mock_klipper_printer()
    }

    pub fn get_cnc_status() -> MachineStatus {
        HardwareAdapter::mock_grbl_cnc()
    }

    pub fn get_bus_pins() -> Vec<BusPiratePinState> {
        HardwareAdapter::mock_bus_pirate()
    }

    pub fn reference_rig() -> &'static str {
        REFERENCE_RIG
    }

    /// Lint with the default 3018-class CNC profile (back-compat entry point).
    pub fn lint_gcode(gcode: &str) -> Result<ToolpathLintReport, MakerError> {
        Self::lint_gcode_with_profile(gcode, &MachineProfile::default_cnc())
    }

    /// Full lint against an explicit machine profile.
    pub fn lint_gcode_with_profile(
        gcode: &str,
        profile: &MachineProfile,
    ) -> Result<ToolpathLintReport, MakerError> {
        let total_lines = gcode.lines().count();
        let (path, mut warnings, seen_units, inches) = parse_toolpath(gcode)?;
        let mut errors: Vec<String> = Vec::new();

        if total_lines == 0 || gcode.trim().is_empty() {
            return Ok(ToolpathLintReport {
                total_lines: 0,
                travel_distance_mm: 0.0,
                max_z_mm: 0.0,
                min_z_mm: 0.0,
                passes_safety: false,
                warnings: vec![],
                errors: vec!["empty program: nothing to verify".to_string()],
                estimated_minutes: 0.0,
                segment_count: 0,
            });
        }
        if !seen_units {
            warnings.push("no G21/G20: assuming millimeters".to_string());
        }
        if path.segments.is_empty() {
            errors.push("no motion (G0/G1) commands found".to_string());
        }
        if inches {
            warnings.push("G20 inches detected: all bounds checked after conversion".to_string());
        }

        let mut travel = 0.0_f64;
        let mut seconds = 0.0_f64;
        let mut spindle_on = false;
        // Re-scan raw lines for spindle/temperature words the motion parser skips.
        for (idx, raw) in gcode.lines().enumerate() {
            let line_no = idx + 1;
            let code = raw.split(';').next().unwrap_or("").trim().to_uppercase();
            if code.starts_with("M3") || code.starts_with("M4") {
                spindle_on = true;
                if !code.contains('S') && profile.kind == ProfileKind::Cnc {
                    warnings.push(format!("line {line_no}: spindle on without S speed"));
                }
            }
            if code.starts_with("M5") {
                spindle_on = false;
            }
            if code.starts_with("M104") || code.starts_with("M109") {
                let s = code
                    .split_whitespace()
                    .find(|t| t.starts_with('S'))
                    .and_then(|t| t[1..].parse::<f64>().ok())
                    .unwrap_or(0.0);
                if profile.kind == ProfileKind::Printer && s > profile.max_nozzle_temp_c {
                    errors.push(format!(
                        "line {line_no}: nozzle {s}C exceeds {t}C ceiling",
                        t = profile.max_nozzle_temp_c
                    ));
                }
            }
        }

        for seg in &path.segments {
            let dx = seg.to_x_mm - seg.from_x_mm;
            let dy = seg.to_y_mm - seg.from_y_mm;
            let dz = seg.to_z_mm - seg.from_z_mm;
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            travel += len;
            let feed = seg.feed_mm_min.max(60.0);
            seconds += len / feed * 60.0;

            // Envelope.
            for (axis, v, lo, hi) in [
                ("X", seg.to_x_mm, 0.0, profile.bed_x_mm),
                ("Y", seg.to_y_mm, 0.0, profile.bed_y_mm),
            ] {
                if v < lo - 1e-6 || v > hi + 1e-6 {
                    errors.push(format!(
                        "line {}: {axis}={v:.2}mm outside 0..{hi:.0}mm envelope ({})",
                        seg.line_no, profile.name
                    ));
                }
            }
            if seg.to_z_mm > profile.max_z_mm + 1e-6 {
                errors.push(format!(
                    "line {}: Z={:.2}mm above Z-max {:.0}mm",
                    seg.line_no, seg.to_z_mm, profile.max_z_mm
                ));
            }
            match profile.kind {
                ProfileKind::Printer => {
                    if seg.to_z_mm < -1e-6 {
                        errors.push(format!(
                            "line {}: Z={:.2}mm below bed (printer cannot cut)",
                            seg.line_no, seg.to_z_mm
                        ));
                    }
                }
                ProfileKind::Cnc => {
                    if seg.to_z_mm < -profile.max_cut_depth_mm - 1e-6 {
                        errors.push(format!(
                            "line {}: Z={:.2}mm deeper than -{:.1}mm single-pass limit",
                            seg.line_no, seg.to_z_mm, profile.max_cut_depth_mm
                        ));
                    }
                    if seg.rapid && seg.to_z_mm < 0.0 && dz < -1e-9 {
                        errors.push(format!(
                            "line {}: rapid (G0) plunge below stock top - use G1 feed move",
                            seg.line_no
                        ));
                    }
                    if !seg.rapid && seg.to_z_mm < 0.0 {
                        warnings.push(format!(
                            "line {}: sub-zero plunge - verify stock thickness",
                            seg.line_no
                        ));
                    }
                }
            }
            if seg.feed_mm_min > profile.max_feedrate_mm_min + 1e-6 {
                warnings.push(format!(
                    "line {}: feed {:.0} exceeds {:.0}mm/min profile max",
                    seg.line_no, seg.feed_mm_min, profile.max_feedrate_mm_min
                ));
            }
            let _ = spindle_on;
        }

        // Cap noise: keep first 50 findings.
        warnings.truncate(50);
        errors.truncate(50);
        let passes_safety = errors.is_empty();
        Ok(ToolpathLintReport {
            total_lines,
            travel_distance_mm: (travel * 10.0).round() / 10.0,
            max_z_mm: (path.max_z_mm * 10.0).round() / 10.0,
            min_z_mm: (path.min_z_mm * 10.0).round() / 10.0,
            passes_safety,
            warnings,
            errors,
            estimated_minutes: (seconds / 60.0 * 10.0).round() / 10.0,
            segment_count: path.segments.len(),
        })
    }

    /// Motion segments for the 2D toolpath preview (no profile needed).
    pub fn preview_segments(gcode: &str) -> Result<ParsedToolpath, MakerError> {
        Ok(parse_toolpath(gcode)?.0)
    }
}
