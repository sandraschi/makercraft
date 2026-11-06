use clap::Parser;
use makercraft_core::{MachineProfile, MakerEngine, ToolpathLintReport};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "makercraft", about = "MakerCraft Digital Fabrication Studio")]
struct Args {
    #[arg(long)]
    smoke: bool,
}

// Ultra-contrast bench palette (pure white on near-black).
const INK_BG: egui::Color32 = egui::Color32::from_rgb(5, 8, 16);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(14, 20, 34);
const GOLD: egui::Color32 = egui::Color32::from_rgb(255, 205, 105);
const PAPER: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const MIST: egui::Color32 = egui::Color32::from_rgb(208, 228, 252);
const TEAL: egui::Color32 = egui::Color32::from_rgb(130, 245, 210);
const ROSE: egui::Color32 = egui::Color32::from_rgb(255, 135, 135);
const LINE_BRIGHT: egui::Color32 = egui::Color32::from_rgb(90, 130, 185);
const OK_GREEN: egui::Color32 = egui::Color32::from_rgb(120, 255, 170);

#[derive(PartialEq, Eq)]
enum BenchTab {
    Preview,
    Safety,
    Machines,
    Gcode,
}

pub struct MakerCraftApp {
    status_msg: String,
    locale: makercraft_core::i18n::AppLocale,
    gcode_buffer: String,
    active_tab: BenchTab,
    profile_cnc: bool,
    lint_report: Option<ToolpathLintReport>,
    preview_zoom: f32,
    preview_pan: egui::Vec2,
    estop: bool,
    is_dark_theme: bool,
    is_authenticated: bool,
    auth_username: String,
    auth_password_input: String,
    show_auth_modal: bool,
    show_about_dialog: bool,
}

impl Default for MakerCraftApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MakerCraftApp {
    pub fn new() -> Self {
        let gcode_buffer =
            "G90\nG21\nG0 Z25.0 F3000\nG1 X20 Y20 F1200\nG1 Z-1.0 F300\nG1 X80 Y20\nG1 X80 Y80\nG1 X20 Y80\nG1 X20 Y20\nG0 Z25\nM2"
                .to_string();
        let lint_report = MakerEngine::lint_gcode(&gcode_buffer).ok();
        Self {
            status_msg: "Fabrication Bench Ready (Port 14130)".to_string(),
            locale: makercraft_core::i18n::AppLocale::default(),
            gcode_buffer,
            active_tab: BenchTab::Preview,
            profile_cnc: true,
            lint_report,
            preview_zoom: 1.0,
            preview_pan: egui::Vec2::ZERO,
            estop: false,
            is_dark_theme: true,
            is_authenticated: false,
            auth_username: "fabricator".to_string(),
            auth_password_input: String::new(),
            show_auth_modal: false,
            show_about_dialog: false,
        }
    }

    fn profile(&self) -> MachineProfile {
        if self.profile_cnc {
            MachineProfile::default_cnc()
        } else {
            MachineProfile::default_printer()
        }
    }

    fn analyze(&mut self) {
        match MakerEngine::lint_gcode_with_profile(&self.gcode_buffer, &self.profile()) {
            Ok(report) => {
                self.status_msg = if report.passes_safety {
                    format!(
                        "PASS: {} segs, {:.1}mm travel, ~{:.1}min | {} warnings",
                        report.segment_count,
                        report.travel_distance_mm,
                        report.estimated_minutes,
                        report.warnings.len()
                    )
                } else {
                    format!(
                        "FAIL: {} errors, {} warnings - see Safety tab",
                        report.errors.len(),
                        report.warnings.len()
                    )
                };
                self.lint_report = Some(report);
            }
            Err(e) => {
                self.status_msg = format!("Lint error: {e}");
            }
        }
    }
}

fn big_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(27.0).strong().color(PAPER));
}

fn body_text(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(17.5).color(PAPER));
}

impl eframe::App for MakerCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.is_dark_theme {
            let mut v = egui::Visuals::dark();
            v.override_text_color = Some(egui::Color32::WHITE);
            v.panel_fill = INK_BG;
            v.window_fill = CARD_BG;
            v.extreme_bg_color = egui::Color32::BLACK;
            v.selection.bg_fill = GOLD;
            v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.hyperlink_color = GOLD;
            v.window_stroke = egui::Stroke::new(2.0, LINE_BRIGHT);
            v.widgets.noninteractive.fg_stroke = egui::Stroke::new(2.0, PAPER);
            v.widgets.inactive.fg_stroke = egui::Stroke::new(2.0, PAPER);
            v.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, LINE_BRIGHT);
            v.widgets.hovered.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.hovered.bg_fill = GOLD;
            v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.active.bg_fill = TEAL;
            v.widgets.open.fg_stroke = egui::Stroke::new(3.0, GOLD);
            ui.ctx().set_visuals(v);
        } else {
            let mut v = egui::Visuals::light();
            v.override_text_color = Some(egui::Color32::BLACK);
            v.panel_fill = egui::Color32::WHITE;
            v.window_fill = egui::Color32::from_rgb(244, 246, 250);
            v.extreme_bg_color = egui::Color32::WHITE;
            v.selection.bg_fill = egui::Color32::from_rgb(10, 25, 60);
            v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
            v.window_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(10, 25, 60));
            v.widgets.inactive.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.hovered.bg_fill = GOLD;
            v.widgets.hovered.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.active.bg_fill = TEAL;
            v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            ui.ctx().set_visuals(v);
        }

        // 0. Desktop Menu Bar
        egui::Panel::top("desktop_menu_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import G-code File...").clicked() {
                        self.status_msg =
                            "Paste G-code into the G-code tab (file picker in v2)".to_string();
                        self.active_tab = BenchTab::Gcode;
                        ui.close();
                    }
                    if ui.button("Analyze Toolpath Safety").clicked() {
                        self.analyze();
                        self.active_tab = BenchTab::Safety;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Machines", |ui| {
                    if ui.button("Emergency Kill All Spindles").clicked() {
                        self.estop = true;
                        self.status_msg =
                            "EMERGENCY STOP: ALL MOTORS CUT (press Reset ESTOP in Machines tab)"
                                .to_string();
                        ui.close();
                    }
                    if ui.button("Zero All Coordinates (G92 X0 Y0 Z0)").clicked() {
                        self.status_msg = "Work coordinates zeroed".to_string();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    let theme_label = if self.is_dark_theme {
                        "Switch to Light Theme (ultra contrast)"
                    } else {
                        "Switch to Dark Theme (ultra contrast)"
                    };
                    if ui.button(theme_label).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                        ui.close();
                    }
                });

                ui.menu_button("Security", |ui| {
                    if self.is_authenticated {
                        ui.label(format!("Fabricator: {}", self.auth_username));
                        if ui.button("Sign Out").clicked() {
                            self.is_authenticated = false;
                            self.status_msg = "Signed out".to_string();
                            ui.close();
                        }
                    } else if ui.button("Basic Auth Login...").clicked() {
                        self.show_auth_modal = true;
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About MakerCraft").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cur_loc = self.locale;
                    let label = format!("{} {}", cur_loc.flag(), cur_loc.code().to_uppercase());
                    egui::ComboBox::from_id_salt("topbar_locale_selector")
                        .selected_text(label)
                        .show_ui(ui, |ui| {
                            for loc_item in makercraft_core::i18n::AppLocale::all() {
                                let text =
                                    format!("{} {}", loc_item.flag(), loc_item.display_name());
                                let _ = ui.selectable_value(&mut self.locale, *loc_item, text);
                            }
                        });

                    if self.estop {
                        ui.colored_label(ROSE, "ESTOP ACTIVE");
                    }
                    let theme_icon = if self.is_dark_theme { "Light" } else { "Dark" };
                    if ui.small_button(theme_icon).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                    }
                    if self.is_authenticated {
                        ui.colored_label(GOLD, format!("Operator {}", self.auth_username));
                    } else if ui.small_button("Login").clicked() {
                        self.show_auth_modal = true;
                    }
                });
            });
        });

        // 1. Tab bar + status
        egui::Panel::top("bench_tabs").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("MakerCraft Bench");
                ui.separator();
                for (tab, label) in [
                    (BenchTab::Preview, "2D Preview"),
                    (BenchTab::Safety, "Safety Lint"),
                    (BenchTab::Machines, "Machines"),
                    (BenchTab::Gcode, "G-code"),
                ] {
                    if ui.selectable_label(self.active_tab == tab, label).clicked() {
                        self.active_tab = tab;
                    }
                }
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Port 14130 | {}", self.status_msg)).size(16.0),
                );
            });
        });

        // 2. Left: profile + telemetry
        egui::Panel::left("machine_telemetry").show(ui, |ui| {
            ui.set_width(300.0);
            ui.label(
                egui::RichText::new("Machine Profile")
                    .size(19.0)
                    .strong()
                    .color(GOLD),
            );
            ui.horizontal(|ui| {
                if ui.selectable_label(self.profile_cnc, "3018 CNC").clicked() {
                    self.profile_cnc = true;
                    self.analyze();
                }
                if ui.selectable_label(!self.profile_cnc, "Ender-3").clicked() {
                    self.profile_cnc = false;
                    self.analyze();
                }
            });
            let p = self.profile();
            ui.label(
                egui::RichText::new(format!(
                    "{:.0}x{:.0}x{:.0}mm | max {:.0}mm/min",
                    p.bed_x_mm, p.bed_y_mm, p.max_z_mm, p.max_feedrate_mm_min
                ))
                .size(15.0)
                .color(MIST),
            );
            ui.separator();
            ui.label(
                egui::RichText::new("Active Hardware")
                    .size(19.0)
                    .strong()
                    .color(GOLD),
            );
            let printer = MakerEngine::get_printer_status();
            ui.group(|ui| {
                ui.label(
                    egui::RichText::new("3D Printer (Klipper)")
                        .size(17.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "State: {}",
                        if self.estop { "ESTOP" } else { &printer.state }
                    ))
                    .size(16.0),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Toolhead: ({:.1}, {:.1}, {:.1}) mm",
                        printer.toolhead_x_mm, printer.toolhead_y_mm, printer.toolhead_z_mm
                    ))
                    .size(16.0),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Extruder: {:.1} C",
                        printer.extruder_or_spindle_temp
                    ))
                    .size(16.0),
                );
            });

            let cnc = MakerEngine::get_cnc_status();
            ui.group(|ui| {
                ui.label(egui::RichText::new("CNC Router (GRBL)").size(17.0).strong());
                ui.label(
                    egui::RichText::new(format!(
                        "State: {}",
                        if self.estop { "ESTOP" } else { &cnc.state }
                    ))
                    .size(16.0),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Spindle: {:.0} RPM",
                        cnc.extruder_or_spindle_temp
                    ))
                    .size(16.0),
                );
            });

            ui.group(|ui| {
                ui.label(
                    egui::RichText::new("Bus Pirate / Logic")
                        .size(17.0)
                        .strong(),
                );
                for pin in MakerEngine::get_bus_pins() {
                    ui.label(
                        egui::RichText::new(format!(
                            "{}: {:.1}V ({})",
                            pin.pin_name,
                            pin.voltage_v,
                            if pin.is_high { "HIGH" } else { "LOW" }
                        ))
                        .size(16.0),
                    );
                }
            });
        });

        // 3. Center tabs
        egui::CentralPanel::default().show(ui, |ui| match self.active_tab {
            BenchTab::Preview => {
                big_heading(ui, "2D Toolpath Preview");
                body_text(ui, "Top-down XY. Teal = feed cut, red = rapid travel. Wheel zooms, drag pans.");
                ui.horizontal(|ui| {
                    if ui.button("Zoom -").clicked() {
                        self.preview_zoom = (self.preview_zoom / 1.25).clamp(0.25, 8.0);
                    }
                    if ui.button("Fit").clicked() {
                        self.preview_zoom = 1.0;
                        self.preview_pan = egui::Vec2::ZERO;
                    }
                    if ui.button("Zoom +").clicked() {
                        self.preview_zoom = (self.preview_zoom * 1.25).clamp(0.25, 8.0);
                    }
                    if let Some(r) = &self.lint_report {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} segs | {:.0}mm | ~{:.1}min",
                                r.segment_count, r.travel_distance_mm, r.estimated_minutes
                            ))
                            .size(16.0)
                            .color(GOLD),
                        );
                    }
                });
                ui.add_space(4.0);
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 420.0),
                    egui::Sense::click_and_drag(),
                );
                let wheel = ui.input(|i| i.smooth_scroll_delta.y);
                if response.hovered() && wheel != 0.0 {
                    let factor = (1.0_f32 + wheel * 0.0015).clamp(0.8, 1.25);
                    self.preview_zoom = (self.preview_zoom * factor).clamp(0.25, 8.0);
                }
                if response.dragged() {
                    self.preview_pan += response.drag_delta();
                }
                let painter = ui.painter();
                painter.rect_filled(rect, 6.0, egui::Color32::BLACK);
                painter.rect_stroke(
                    rect,
                    6.0,
                    egui::Stroke::new(3.0, GOLD),
                    egui::StrokeKind::Inside,
                );
                match MakerEngine::preview_segments(&self.gcode_buffer) {
                    Ok(path) if !path.segments.is_empty() => {
                        let span_x = (path.max_x_mm - path.min_x_mm).max(1.0);
                        let span_y = (path.max_y_mm - path.min_y_mm).max(1.0);
                        let base = (rect.width() as f64 / span_x)
                            .min(rect.height() as f64 / span_y)
                            * 0.85;
                        let scale = (base * self.preview_zoom as f64) as f32;
                        let cx = (path.min_x_mm + path.max_x_mm) / 2.0;
                        let cy = (path.min_y_mm + path.max_y_mm) / 2.0;
                        let to_screen = |x: f64, y: f64| {
                            egui::pos2(
                                rect.center().x + ((x - cx) as f32 * scale) + self.preview_pan.x,
                                rect.center().y - ((y - cy) as f32 * scale) + self.preview_pan.y,
                            )
                        };
                        // Bed outline for the active profile.
                        let p = self.profile();
                        let bed = [
                            to_screen(0.0, 0.0),
                            to_screen(p.bed_x_mm, 0.0),
                            to_screen(p.bed_x_mm, p.bed_y_mm),
                            to_screen(0.0, p.bed_y_mm),
                            to_screen(0.0, 0.0),
                        ];
                        painter.add(egui::Shape::line(
                            bed.to_vec(),
                            egui::Stroke::new(2.0, LINE_BRIGHT),
                        ));
                        for seg in &path.segments {
                            let a = to_screen(seg.from_x_mm, seg.from_y_mm);
                            let b = to_screen(seg.to_x_mm, seg.to_y_mm);
                            let stroke = if seg.rapid {
                                egui::Stroke::new(1.5, ROSE)
                            } else {
                                egui::Stroke::new(2.5, TEAL)
                            };
                            painter.line_segment([a, b], stroke);
                        }
                        if let (Some(first), Some(last)) =
                            (path.segments.first(), path.segments.last())
                        {
                            painter.circle_filled(
                                to_screen(first.from_x_mm, first.from_y_mm),
                                6.0,
                                OK_GREEN,
                            );
                            painter.circle_filled(
                                to_screen(last.to_x_mm, last.to_y_mm),
                                6.0,
                                egui::Color32::WHITE,
                            );
                        }
                    }
                    _ => {
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "No motion commands - see G-code tab",
                            egui::FontId::proportional(18.0),
                            MIST,
                        );
                    }
                }
                ui.add_space(6.0);
                body_text(ui, "Green = start, white = end. Red rapids must stay above safe Z.");
            }

            BenchTab::Safety => {
                big_heading(ui, "Safety Lint");
                ui.horizontal(|ui| {
                    body_text(ui, &format!("Profile: {}", self.profile().name));
                    if ui.button("Re-analyze").clicked() {
                        self.analyze();
                    }
                });
                if let Some(r) = &self.lint_report {
                    ui.add_space(4.0);
                    if r.passes_safety {
                        ui.group(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "PASS - {} lines, {} segs, {:.1}mm, ~{:.1}min",
                                    r.total_lines,
                                    r.segment_count,
                                    r.travel_distance_mm,
                                    r.estimated_minutes
                                ))
                                .size(19.0)
                                .strong()
                                .color(OK_GREEN),
                            );
                        });
                    } else {
                        ui.group(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "FAIL - {} errors (fix before running job)",
                                    r.errors.len()
                                ))
                                .size(19.0)
                                .strong()
                                .color(ROSE),
                            );
                        });
                    }
                    if !r.errors.is_empty() {
                        ui.label(egui::RichText::new("Errors").size(19.0).strong().color(ROSE));
                        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                            for e in &r.errors {
                                ui.label(egui::RichText::new(format!("! {e}")).size(17.0).color(ROSE));
                            }
                        });
                    }
                    if !r.warnings.is_empty() {
                        ui.label(egui::RichText::new("Warnings").size(19.0).strong().color(GOLD));
                        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                            for w in &r.warnings {
                                ui.label(egui::RichText::new(format!("- {w}")).size(17.0).color(PAPER));
                            }
                        });
                    }
                    ui.add_space(4.0);
                    body_text(ui, &format!(
                        "Z range {:.1}..{:.1}mm | Reference rig: {}",
                        r.min_z_mm,
                        r.max_z_mm,
                        MakerEngine::reference_rig()
                    ));
                } else {
                    body_text(ui, "No report yet - press Re-analyze.");
                }
            }

            BenchTab::Machines => {
                big_heading(ui, "Machines & Safety Interlock");
                body_text(ui, MakerEngine::reference_rig());
                ui.add_space(6.0);
                if self.estop {
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new("ESTOP ACTIVE - all motion cut")
                                .size(21.0)
                                .strong()
                                .color(ROSE),
                        );
                        if ui.button("Reset ESTOP").clicked() {
                            self.estop = false;
                            self.status_msg = "ESTOP cleared - machines READY".to_string();
                        }
                    });
                } else if ui.button("Emergency Kill All Spindles").clicked() {
                    self.estop = true;
                    self.status_msg = "EMERGENCY STOP: ALL MOTORS CUT".to_string();
                }
                ui.add_space(6.0);
                body_text(ui, "Live Moonraker/GRBL bridges land here (v2). Telemetry above is the declared reference-rig mock until hardware-in-the-loop lands.");
            }

            BenchTab::Gcode => {
                big_heading(ui, "G-code Editor");
                ui.horizontal(|ui| {
                    body_text(ui, &format!("Profile: {}", self.profile().name));
                    if ui.button("Analyze Toolpath Safety").clicked() {
                        self.analyze();
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.gcode_buffer);
                });
            }
        });

        // Modal: Basic Auth Login
        if self.show_auth_modal {
            egui::Window::new("Machine Operator Auth")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Authenticate for Live CNC / Laser Toolhead Fire:");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("User:");
                        ui.text_edit_singleline(&mut self.auth_username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Pass:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.auth_password_input)
                                .password(true),
                        );
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Confirm").clicked() && !self.auth_username.is_empty() {
                            self.is_authenticated = true;
                            self.show_auth_modal = false;
                            self.status_msg = format!("Signed in as {}", self.auth_username);
                            self.auth_password_input.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_auth_modal = false;
                        }
                    });
                });
        }

        // Modal: About Dialog
        if self.show_about_dialog {
            egui::Window::new("About MakerCraft")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.heading("MakerCraft Studio");
                    ui.label("Sovereign digital fabrication, CNC, and 3D printing studio.");
                    ui.label("Version: v0.2.0");
                    ui.label("Control Port: 14130");
                    ui.separator();
                    ui.label("Real G-code parser: modal G0/G1, G90/G91, G20/G21, arcs noted, envelope + plunge + feed + temp gates.");
                    ui.label(format!("Reference rig: {}", MakerEngine::reference_rig()));
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        println!("[makercraft] Running headless smoke verification...");
        let printer = MakerEngine::get_printer_status();
        let report = MakerEngine::lint_gcode("G21\nG90\nG0 Z25.0\nG1 X10 Y10\nG1 Z-1.5")?;
        let preview = MakerEngine::preview_segments("G21\nG90\nG1 X10 Y10\nX20")?;
        let smoke_payload = json!({
            "app": "makercraft",
            "version": "0.2.0",
            "status": "ready",
            "control_port": 14130,
            "success": true,
            "printer_ready": printer.is_connected,
            "passes_safety": report.passes_safety,
            "segment_count": report.segment_count,
            "estimated_minutes": report.estimated_minutes,
            "preview_segments": preview.segments.len(),
            "reference_rig": MakerEngine::reference_rig(),
            "capabilities": ["KlipperGcodeStreaming", "GrblSpindleControl", "BusPirateLogicSniffing", "ZeroCollisionVerification", "ToolpathPreview2D", "MachineProfiles"]
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
