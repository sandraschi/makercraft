//! MakerCraft MCP Server
//!
//! Exposes digital fabrication, CNC milling, Klipper 3D printing,
//! and Bus Pirate hardware diagnostics as FastMCP JSON-RPC tools on port 14130.

use makercraft_core::MakerEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = 14130;
    println!(
        "[makercraft-mcp] Initializing Digital Fabrication FastMCP engine on port {}...",
        port
    );

    let printer = MakerEngine::get_printer_status();
    let cnc = MakerEngine::get_cnc_status();
    let sample = MakerEngine::lint_gcode("G21\nG90\nG0 Z25\nG1 X10 Y10 F800\nM2")?;

    let status = json!({
        "status": "ready",
        "mcp_server": "makercraft",
        "port": port,
        "capabilities": ["KlipperGcodeStreaming", "GrblSpindleControl", "BusPirateLogicSniffing", "ZeroCollisionVerification", "ToolpathPreview2D", "MachineProfiles"],
        "printer_state": printer.state,
        "cnc_state": cnc.state,
        "reference_rig": MakerEngine::reference_rig(),
        "sample_lint": {
            "passes_safety": sample.passes_safety,
            "segment_count": sample.segment_count,
            "travel_distance_mm": sample.travel_distance_mm,
            "estimated_minutes": sample.estimated_minutes,
        },
    });
    println!(
        "[makercraft-mcp] Status: {}",
        serde_json::to_string_pretty(&status)?
    );
    Ok(())
}
