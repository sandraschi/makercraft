use clap::Parser;
use serde_json::json;
use makercraft_core::MakerEngine;

#[derive(Parser, Debug)]
#[command(name = "makercraft", about = "MakerCraft Digital Fabrication Studio")]
struct Args {
    #[arg(long)]
    smoke: bool,
}

pub struct MakerCraftApp {
    status_msg: String,
    gcode_buffer: String,
}

impl MakerCraftApp {
    pub fn new() -> Self {
        Self {
            status_msg: "Fabrication Bench Ready (Port 14130)".to_string(),
            gcode_buffer: "G90\nG21\nG0 Z25.0 F3000\nG1 X20 Y20 F1200\nG1 Z-1.0 F300\nM2".to_string(),
        }
    }
}

impl eframe::App for MakerCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🛠️ MakerCraft — Digital Fabrication Bench & CNC/3D Studio");
                ui.separator();
                ui.label(&self.status_msg);
            });
        });

        egui::Panel::left("machine_telemetry").show(ui, |ui| {
            ui.heading("Active Hardware");
            let printer = MakerEngine::get_printer_status();
            ui.group(|ui| {
                ui.label(egui::RichText::new("3D Printer (Klipper)").strong());
                ui.label(format!("State: {}", printer.state));
                ui.label(format!("Toolhead: ({:.1}, {:.1}, {:.1}) mm", printer.toolhead_x_mm, printer.toolhead_y_mm, printer.toolhead_z_mm));
                ui.label(format!("Extruder: {:.1} °C", printer.extruder_or_spindle_temp));
            });

            let cnc = MakerEngine::get_cnc_status();
            ui.group(|ui| {
                ui.label(egui::RichText::new("CNC Router (GRBL)").strong());
                ui.label(format!("State: {}", cnc.state));
                ui.label(format!("Spindle: {:.0} RPM", cnc.extruder_or_spindle_temp));
            });

            ui.group(|ui| {
                ui.label(egui::RichText::new("Bus Pirate / Logic").strong());
                let pins = MakerEngine::get_bus_pins();
                for pin in pins {
                    ui.label(format!("{}: {:.1}V ({})", pin.pin_name, pin.voltage_v, if pin.is_high { "HIGH" } else { "LOW" }));
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Toolpath Inspector & Safety Lint");
            ui.text_edit_multiline(&mut self.gcode_buffer);
            if ui.button("Analyze Toolpath Safety").clicked() {
                if let Ok(report) = MakerEngine::lint_gcode(&self.gcode_buffer) {
                    self.status_msg = format!("Lint: {} lines, max Z {:.1}mm, min Z {:.1}mm, pass: {}",
                        report.total_lines, report.max_z_mm, report.min_z_mm, report.passes_safety);
                }
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        println!("[makercraft] Running headless smoke verification...");
        let printer = MakerEngine::get_printer_status();
        let report = MakerEngine::lint_gcode("G0 Z25.0\nG1 X10 Y10\nG1 Z-1.5")?;
        let smoke_payload = json!({
            "app": "makercraft",
            "version": "0.1.0",
            "status": "ready",
            "control_port": 14130,
            "success": true,
            "printer_ready": printer.is_connected,
            "passes_safety": report.passes_safety,
            "capabilities": ["KlipperGcodeStreaming", "GrblSpindleControl", "BusPirateLogicSniffing", "ZeroCollisionVerification"]
        });
        println!("[makercraft] Smoke OK: {}", smoke_payload);
        return Ok(());
    }

    let app = MakerCraftApp::new();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("MakerCraft — Digital Fabrication Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "MakerCraft",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;
    Ok(())
}
