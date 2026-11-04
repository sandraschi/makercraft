use clap::{Parser, Subcommand};
use makercraft_core::MakerEngine;

#[derive(Parser)]
#[command(name = "makercraft-cli", about = "MakerCraft Digital Fabrication Bench CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect connected machines (printer, CNC, bus pirate)
    Status,
    /// Lint G-code toolpath for collision hazards
    Lint {
        path: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let printer = MakerEngine::get_printer_status();
            let cnc = MakerEngine::get_cnc_status();
            let pins = MakerEngine::get_bus_pins();

            let payload = serde_json::json!({
                "printer": printer,
                "cnc": cnc,
                "bus_pirate_pins": pins,
            });
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        Commands::Lint { path } => {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| "G0 Z25.0\nG1 X10 Y10\nG1 Z-1.5".to_string());
            let report = MakerEngine::lint_gcode(&content)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}
