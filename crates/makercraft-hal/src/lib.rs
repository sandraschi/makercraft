//! MakerCraft Hardware Abstraction Layer
//!
//! Provides unified protocol interfaces for Klipper / Moonraker 3D printers,
//! GRBL CNC milling routers, and Bus Pirate / Logic Analyzer instrumentation.
//!
//! L1 HAL only: mock oracles plus v2 read-only live bridges (see [`live`]).
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

pub mod live;
pub mod usb;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MachineType {
    Klipper3DPrinter,
    GrblCncRouter,
    BusPirateProtocolBridge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineStatus {
    pub machine_type: MachineType,
    pub is_connected: bool,
    pub state: String, // "READY", "PRINTING", "MILLING", "ESTOP"
    pub toolhead_x_mm: f64,
    pub toolhead_y_mm: f64,
    pub toolhead_z_mm: f64,
    pub feedrate_mm_min: f64,
    pub extruder_or_spindle_temp: f64,
    pub target_temp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusPiratePinState {
    pub pin_name: String,
    pub is_high: bool,
    pub pullup_enabled: bool,
    pub voltage_v: f64,
}

pub struct HardwareAdapter;

impl HardwareAdapter {
    pub fn mock_klipper_printer() -> MachineStatus {
        MachineStatus {
            machine_type: MachineType::Klipper3DPrinter,
            is_connected: true,
            state: "READY".to_string(),
            toolhead_x_mm: 150.0,
            toolhead_y_mm: 150.0,
            toolhead_z_mm: 0.2,
            feedrate_mm_min: 3600.0,
            extruder_or_spindle_temp: 215.0,
            target_temp: 215.0,
        }
    }

    pub fn mock_grbl_cnc() -> MachineStatus {
        MachineStatus {
            machine_type: MachineType::GrblCncRouter,
            is_connected: true,
            state: "READY".to_string(),
            toolhead_x_mm: 0.0,
            toolhead_y_mm: 0.0,
            toolhead_z_mm: 5.0,
            feedrate_mm_min: 1200.0,
            extruder_or_spindle_temp: 18000.0, // Spindle RPM
            target_temp: 18000.0,
        }
    }

    pub fn mock_bus_pirate() -> Vec<BusPiratePinState> {
        vec![
            BusPiratePinState {
                pin_name: "MOSI".to_string(),
                is_high: true,
                pullup_enabled: true,
                voltage_v: 3.3,
            },
            BusPiratePinState {
                pin_name: "MISO".to_string(),
                is_high: false,
                pullup_enabled: true,
                voltage_v: 0.1,
            },
            BusPiratePinState {
                pin_name: "CLK".to_string(),
                is_high: false,
                pullup_enabled: false,
                voltage_v: 0.0,
            },
            BusPiratePinState {
                pin_name: "CS".to_string(),
                is_high: true,
                pullup_enabled: false,
                voltage_v: 3.3,
            },
        ]
    }
}
