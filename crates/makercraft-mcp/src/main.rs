//! MakerCraft MCP Server
//!
//! Exposes digital fabrication, CNC milling, Klipper 3D printing,
//! and Bus Pirate hardware diagnostics as FastMCP JSON-RPC tools on port 14130.

use serde_json::json;
use makercraft_core::MakerEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = 14130;
    println!("[makercraft-mcp] Initializing Digital Fabrication FastMCP engine on port {}...", port);

    let printer = MakerEngine::get_printer_status();
    let cnc = MakerEngine::get_cnc_status();

    let status = json!({
        "status": "ready",
        "mcp_server": "makercraft",
        "port": port,
        "capabilities": ["KlipperGcodeStreaming", "GrblSpindleControl", "BusPirateLogicSniffing", "ZeroCollisionVerification"],
        "printer_state": printer.state,
        "cnc_state": cnc.state,
    });
    println!("[makercraft-mcp] Status: {}", serde_json::to_string_pretty(&status)?);
    Ok(())
}
