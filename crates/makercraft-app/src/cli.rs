use clap::{Parser, Subcommand};
use makercraft_core::{MachineProfile, MakerEngine};

#[derive(Parser)]
#[command(
    name = "makercraft-cli",
    about = "MakerCraft Digital Fabrication Bench CLI"
)]
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
        /// Machine profile: cnc (3018 class, default) or printer (Ender-3 class)
        #[arg(long, default_value = "cnc")]
        profile: String,
    },
    /// Check a file and exit non-zero when unsafe (CI gate)
    Check {
        path: String,
        #[arg(long, default_value = "cnc")]
        profile: String,
    },
    /// Estimate time + travel for a G-code file
    Estimate {
        path: String,
        #[arg(long, default_value = "cnc")]
        profile: String,
    },
}

fn pick_profile(name: &str) -> MachineProfile {
    match name.to_lowercase().as_str() {
        "printer" | "ender-3" | "ender3" => MachineProfile::default_printer(),
        _ => MachineProfile::default_cnc(),
    }
}

fn read_or_sample(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_| "G0 Z25.0\nG1 X10 Y10\nG1 Z-1.5".to_string())
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
        Commands::Lint { path, profile } => {
            let content = read_or_sample(&path);
            let report = MakerEngine::lint_gcode_with_profile(&content, &pick_profile(&profile))?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Check { path, profile } => {
            let content = read_or_sample(&path);
            let report = MakerEngine::lint_gcode_with_profile(&content, &pick_profile(&profile))?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passes_safety {
                std::process::exit(2);
            }
        }
        Commands::Estimate { path, profile } => {
            let content = read_or_sample(&path);
            let report = MakerEngine::lint_gcode_with_profile(&content, &pick_profile(&profile))?;
            let out = serde_json::json!({
                "profile": pick_profile(&profile).name,
                "segment_count": report.segment_count,
                "travel_distance_mm": report.travel_distance_mm,
                "estimated_minutes": report.estimated_minutes,
                "passes_safety": report.passes_safety,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}
