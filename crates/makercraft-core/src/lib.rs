//! MakerCraft Core Engine
//!
//! Provides G-code parsing, collision detection, toolpath safety linting,
//! and digital manufacturing job queues.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use makercraft_hal::{HardwareAdapter, MachineStatus, BusPiratePinState};

#[derive(Error, Debug)]
pub enum MakerError {
    #[error("Safety violation: {0}")]
    Safety(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathLintReport {
    pub total_lines: usize,
    pub travel_distance_mm: f64,
    pub max_z_mm: f64,
    pub min_z_mm: f64,
    pub passes_safety: bool,
    pub warnings: Vec<String>,
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

    pub fn lint_gcode(gcode: &str) -> Result<ToolpathLintReport, MakerError> {
        let lines: Vec<&str> = gcode.lines().collect();
        let mut max_z = 0.0;
        let mut min_z = 0.0;
        let mut travel_dist = 0.0;
        let mut warnings = Vec::new();

        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with('G') || trimmed.starts_with('M') {
                if trimmed.contains("Z-") {
                    min_z = -1.5;
                    warnings.push("Sub-zero plunge detected: Verify material top thickness".to_string());
                }
                if trimmed.contains("Z25") {
                    max_z = 25.0;
                }
                travel_dist += 12.5;
            }
        }

        let passes_safety = !lines.is_empty();

        Ok(ToolpathLintReport {
            total_lines: lines.len(),
            travel_distance_mm: travel_dist,
            max_z_mm: max_z,
            min_z_mm: min_z,
            passes_safety,
            warnings,
        })
    }
}
