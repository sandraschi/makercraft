mod chat_dock;
mod hardware_viz;
mod reference_views;

use chat_dock::ChatDockState;
use clap::Parser;
use hardware_viz::{draw_hardware_diagram, draw_workshop_panorama, HardwareKind, WorkshopAction};
use makercraft_core::{
    dispatch, Command, MachineProfile, MakerEngine, ToolpathLintReport, INSTRUMENT_SOURCES,
};
use reference_views::{
    render_help_docs_window, render_licenses_window, render_shortcuts_window,
    render_vienna_and_buying_tab, HelpDocTab,
};
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

#[derive(PartialEq, Eq, Clone, Copy)]
enum BenchTab {
    Workshop,
    Preview,
    Safety,
    Machines,
    Instruments,
    Gcode,
    ViennaBuying,
}

pub struct MakerCraftApp {
    status_msg: String,
    locale: makercraft_core::i18n::AppLocale,
    gcode_buffer: String,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    active_tab: BenchTab,
    profile_cnc: bool,
    lint_report: Option<ToolpathLintReport>,
    preview_zoom: f32,
    preview_pan: egui::Vec2,
    show_grid: bool,
    show_rapids: bool,
    show_telemetry_panel: bool,
    estop: bool,
    is_dark_theme: bool,
    is_authenticated: bool,
    auth_username: String,
    auth_password_input: String,
    show_auth_modal: bool,
    show_about_dialog: bool,
    show_command_palette: bool,
    palette_query: String,
    show_shortcuts: bool,
    show_docs: bool,
    help_doc_tab: HelpDocTab,
    selected_skill_idx: usize,
    show_licenses: bool,
    file_modal_is_open: Option<bool>,
    file_path_input: String,
    last_probe_json: String,
    selected_station: Option<HardwareKind>,
    chat: ChatDockState,
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
            status_msg: "Fabrication Bench Ready".to_string(),
            locale: makercraft_core::i18n::AppLocale::default(),
            gcode_buffer,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_tab: BenchTab::Preview,
            profile_cnc: true,
            lint_report,
            preview_zoom: 1.0,
            preview_pan: egui::Vec2::ZERO,
            show_grid: true,
            show_rapids: true,
            show_telemetry_panel: true,
            estop: false,
            is_dark_theme: true,
            is_authenticated: false,
            auth_username: "fabricator".to_string(),
            auth_password_input: String::new(),
            show_auth_modal: false,
            show_about_dialog: false,
            show_command_palette: false,
            palette_query: String::new(),
            show_shortcuts: false,
            show_docs: false,
            help_doc_tab: HelpDocTab::GetStarted,
            selected_skill_idx: 0,
            show_licenses: false,
            file_modal_is_open: None,
            file_path_input: "sample.gcode".to_string(),
            last_probe_json: String::new(),
            selected_station: Some(HardwareKind::Printer3d),
            chat: ChatDockState::default(),
        }
    }

    fn set_gcode(&mut self, next: String) {
        if self.gcode_buffer != next {
            self.undo_stack.push(self.gcode_buffer.clone());
            self.redo_stack.clear();
            self.gcode_buffer = next;
            self.analyze();
        }
    }

    fn undo_gcode(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.gcode_buffer.clone());
            self.gcode_buffer = prev;
            self.analyze();
            self.status_msg = "Undid G-code edit".to_string();
        }
    }

    fn redo_gcode(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.gcode_buffer.clone());
            self.gcode_buffer = next;
            self.analyze();
            self.status_msg = "Redid G-code edit".to_string();
        }
    }

    fn run_command(&mut self, label: &str, cmd: Command) {
        let val = dispatch(cmd);
        self.last_probe_json =
            serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string());
        self.status_msg = format!("Executed {label}");
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
    ui.label(egui::RichText::new(text).size(27.0).strong());
}

fn body_text(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(17.5));
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
            v.weak_text_color = Some(egui::Color32::BLACK);
            v.panel_fill = egui::Color32::WHITE;
            v.window_fill = egui::Color32::WHITE;
            v.extreme_bg_color = egui::Color32::WHITE;
            v.faint_bg_color = egui::Color32::WHITE;
            v.widgets.noninteractive.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
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

        self.chat.poll();
        if self.chat.busy {
            ui.ctx().request_repaint();
        }

        // Global keyboard shortcuts
        ui.ctx().input(|i| {
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::K) {
                self.show_command_palette = !self.show_command_palette;
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::J) {
                self.chat.open = !self.chat.open;
            }
            if i.key_pressed(egui::Key::F1)
                || ((i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::Slash))
            {
                self.show_shortcuts = !self.show_shortcuts;
            }
            if i.key_pressed(egui::Key::F5) {
                self.analyze();
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::N) {
                self.set_gcode("G90\nG21\nG0 Z25.0\nM2".to_string());
                self.active_tab = BenchTab::Gcode;
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::O) {
                self.file_modal_is_open = Some(true);
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::S) {
                self.file_modal_is_open = Some(false);
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::Equals) {
                self.preview_zoom = (self.preview_zoom * 1.25).clamp(0.25, 8.0);
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::Minus) {
                self.preview_zoom = (self.preview_zoom / 1.25).clamp(0.25, 8.0);
            }
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::Num0) {
                self.preview_zoom = 1.0;
                self.preview_pan = egui::Vec2::ZERO;
            }
            if i.key_pressed(egui::Key::Escape) && !self.show_command_palette {
                self.estop = true;
                self.status_msg = "EMERGENCY STOP LATCHED (Esc)".to_string();
            }
        });

        // 0. Universal 8-Menu Bar (File, Edit, View, Insert, Modify, Commands, Window, Help)
        egui::Panel::top("desktop_menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Program (Ctrl+N)").clicked() {
                        self.set_gcode("G90\nG21\nG0 Z25.0\nM2".to_string());
                        self.active_tab = BenchTab::Gcode;
                        ui.close();
                    }
                    if ui.button("Open G-code... (Ctrl+O)").clicked() {
                        self.file_modal_is_open = Some(true);
                        ui.close();
                    }
                    if ui.button("Save G-code... (Ctrl+S)").clicked() {
                        self.file_modal_is_open = Some(false);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Export Lint Report JSON").clicked() {
                        let cmd = Command::GcodeLint {
                            gcode: self.gcode_buffer.clone(),
                            profile: if self.profile_cnc { "cnc" } else { "printer" }.to_string(),
                        };
                        self.run_command("Export Lint JSON", cmd);
                        self.active_tab = BenchTab::Safety;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit (Ctrl+Q)").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo G-code (Ctrl+Z)").clicked() {
                        self.undo_gcode();
                        ui.close();
                    }
                    if ui.button("Redo G-code (Ctrl+Y)").clicked() {
                        self.redo_gcode();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Copy G-code to Clipboard").clicked() {
                        ui.ctx().copy_text(self.gcode_buffer.clone());
                        self.status_msg = "Copied G-code to clipboard".to_string();
                        ui.close();
                    }
                    if ui.button("LLM & Bench Preferences...").clicked() {
                        self.chat.open = true;
                        self.chat.show_settings = true;
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In (Ctrl+=)").clicked() {
                        self.preview_zoom = (self.preview_zoom * 1.25).clamp(0.25, 8.0);
                        ui.close();
                    }
                    if ui.button("Zoom Out (Ctrl+-)").clicked() {
                        self.preview_zoom = (self.preview_zoom / 1.25).clamp(0.25, 8.0);
                        ui.close();
                    }
                    if ui.button("Fit Bed to Window (Ctrl+0)").clicked() {
                        self.preview_zoom = 1.0;
                        self.preview_pan = egui::Vec2::ZERO;
                        ui.close();
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.show_grid, "Show Bed Grid").clicked() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.show_rapids, "Show Rapid Moves (G0)").clicked() {
                        ui.close();
                    }
                    ui.separator();
                    let theme_label = if self.is_dark_theme {
                        "Switch to Light Theme"
                    } else {
                        "Switch to Dark Theme"
                    };
                    if ui.button(theme_label).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                        ui.close();
                    }
                });

                ui.menu_button("Insert", |ui| {
                    if ui.button("Sample 60x60 CNC Square").clicked() {
                        self.set_gcode("G90\nG21\nG0 Z25.0 F3000\nG1 X20 Y20 F1200\nG1 Z-1.0 F300\nG1 X80 Y20\nG1 X80 Y80\nG1 X20 Y80\nG1 X20 Y20\nG0 Z25\nM2".to_string());
                        self.active_tab = BenchTab::Preview;
                        ui.close();
                    }
                    if ui.button("Sample Printer Calibration Frame").clicked() {
                        self.profile_cnc = false;
                        self.set_gcode("G90\nG21\nM104 S205\nM140 S60\nG0 Z0.3 F3000\nG1 X20 Y20 F1500\nG1 X120 Y20 E4.0\nG1 X120 Y120 E8.0\nG1 X20 Y120 E12.0\nG1 X20 Y20 E16.0\nG0 Z20\nM104 S0\nM140 S0\nM2".to_string());
                        self.active_tab = BenchTab::Preview;
                        ui.close();
                    }
                    if ui.button("Safe Retract (G0 Z25)").clicked() {
                        let mut next = self.gcode_buffer.clone();
                        next.push_str("\nG0 Z25.0");
                        self.set_gcode(next);
                        ui.close();
                    }
                });

                ui.menu_button("Modify", |ui| {
                    if ui.button("Profile: 3018 CNC (GRBL 1.1h)").clicked() {
                        self.profile_cnc = true;
                        self.analyze();
                        ui.close();
                    }
                    if ui.button("Profile: Ender-3 Printer (Klipper)").clicked() {
                        self.profile_cnc = false;
                        self.analyze();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Prepend Work Zero (G92 X0 Y0 Z0)").clicked() {
                        let next = format!("G92 X0 Y0 Z0\n{}", self.gcode_buffer);
                        self.set_gcode(next);
                        self.status_msg = "Prepended G92 X0 Y0 Z0".to_string();
                        ui.close();
                    }
                    if ui.button("Basic Auth / Operator Sign-In...").clicked() {
                        self.show_auth_modal = true;
                        ui.close();
                    }
                });

                ui.menu_button("Commands", |ui| {
                    if ui.button("Run Safety Lint (F5)").clicked() {
                        self.analyze();
                        self.active_tab = BenchTab::Safety;
                        ui.close();
                    }
                    if ui.button("Probe Klipper Printer (Moonraker)").clicked() {
                        self.run_command("printer_live", Command::PrinterLive { url: String::new() });
                        self.active_tab = BenchTab::Machines;
                        ui.close();
                    }
                    if ui.button("Probe GRBL CNC (Serial)").clicked() {
                        self.run_command("cnc_probe", Command::CncProbe { port: String::new() });
                        self.active_tab = BenchTab::Machines;
                        ui.close();
                    }
                    if ui.button("Probe Bus Pirate (Serial)").clicked() {
                        self.run_command("bus_probe", Command::BusProbe { port: String::new() });
                        self.active_tab = BenchTab::Machines;
                        ui.close();
                    }
                    if ui.button("Capture Oscilloscope (SCPI)").clicked() {
                        self.run_command("scope_idn", Command::ScopeIdn { addr: String::new() });
                        self.active_tab = BenchTab::Instruments;
                        ui.close();
                    }
                    if ui.button("Capture Logic Analyzer").clicked() {
                        self.run_command("logic_capture", Command::LogicCapture { port: String::new(), hz: 1000, samples: 32 });
                        self.active_tab = BenchTab::Instruments;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Emergency Stop (Esc)").clicked() {
                        self.estop = true;
                        self.status_msg = "EMERGENCY STOP: ALL MOTORS CUT".to_string();
                        ui.close();
                    }
                    if ui.button("Reset E-Stop").clicked() {
                        self.estop = false;
                        self.status_msg = "ESTOP cleared - machines READY".to_string();
                        ui.close();
                    }
                });

                ui.menu_button("Window", |ui| {
                    if ui.button("Reset Panel Layout").clicked() {
                        self.show_telemetry_panel = true;
                        self.chat.open = false;
                        self.chat.maximized = false;
                        ui.close();
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.show_telemetry_panel, "Telemetry Sidebar").clicked() {
                        ui.close();
                    }
                    if ui.button("AI Bench Chat (Ctrl+J)").clicked() {
                        self.chat.open = !self.chat.open;
                        ui.close();
                    }
                    ui.separator();
                    for (tab, label) in [
                        (BenchTab::Workshop, "Workshop"),
                        (BenchTab::Preview, "2D Preview"),
                        (BenchTab::Safety, "Safety Lint"),
                        (BenchTab::Machines, "Machines"),
                        (BenchTab::Instruments, "Instruments"),
                        (BenchTab::Gcode, "G-code Editor"),
                        (BenchTab::ViennaBuying, "Vienna & Buying"),
                    ] {
                        if ui.button(label).clicked() {
                            self.active_tab = tab;
                            ui.close();
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Command Palette... (Ctrl+K)").clicked() {
                        self.show_command_palette = true;
                        ui.close();
                    }
                    if ui.button("Keyboard Shortcuts... (F1)").clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                    if ui.button("Project Files & Docs...").clicked() {
                        self.show_docs = true;
                        ui.close();
                    }
                    if ui.button("Licenses & Notices...").clicked() {
                        self.show_licenses = true;
                        ui.close();
                    }
                    ui.separator();
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

                    if ui.small_button(if self.chat.open { "Chat On" } else { "Chat (Ctrl+J)" }).clicked() {
                        self.chat.open = !self.chat.open;
                    }
                    if self.estop {
                        ui.colored_label(ROSE, "ESTOP ACTIVE");
                    }
                    let theme_icon = if self.is_dark_theme { "Light" } else { "Dark" };
                    if ui.small_button(theme_icon).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
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
                    (BenchTab::Workshop, "Workshop"),
                    (BenchTab::Preview, "2D Preview"),
                    (BenchTab::Safety, "Safety Lint"),
                    (BenchTab::Machines, "Machines"),
                    (BenchTab::Instruments, "Instruments"),
                    (BenchTab::Gcode, "G-code"),
                    (BenchTab::ViennaBuying, "Vienna & Buying"),
                ] {
                    if ui.selectable_label(self.active_tab == tab, label).clicked() {
                        self.active_tab = tab;
                    }
                }
                ui.separator();
                ui.label(egui::RichText::new(&self.status_msg).size(15.0));
            });
        });

        // 2. Left: profile + telemetry
        if self.show_telemetry_panel && !(self.chat.open && self.chat.maximized) {
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
        }

        // Right-side AI Chat Dock (when open and not maximized)
        if self.chat.open && !self.chat.maximized {
            let profile_name = self.profile().name;
            let estop = self.estop;
            let lint = self.lint_report.clone();
            egui::Panel::right("makercraft_chat_dock")
                .default_size(380.0)
                .resizable(true)
                .show(ui, |ui| {
                    self.chat.ui(ui, &profile_name, estop, lint.as_ref());
                });
        }

        // 3. Center tabs (or maximized Chat)
        egui::CentralPanel::default().show(ui, |ui| {
            if self.chat.open && self.chat.maximized {
                let profile_name = self.profile().name;
                let estop = self.estop;
                let lint = self.lint_report.clone();
                self.chat.ui(ui, &profile_name, estop, lint.as_ref());
                return;
            }
            match self.active_tab {
            BenchTab::Workshop => {
                egui::ScrollArea::vertical().id_salt("workshop_scroll").show(ui, |ui| {
                    big_heading(ui, "Maker's Workshop & ESD Workbench");
                    body_text(
                        ui,
                        "Click any machine, bench instrument, or the E-Stop mushroom button to inspect or probe.",
                    );
                    ui.add_space(6.0);
                    let avail_w = ui.available_width().max(680.0);
                    match draw_workshop_panorama(ui, avail_w, self.estop, self.selected_station) {
                        WorkshopAction::SelectHardware(kind) => {
                            self.selected_station = Some(kind);
                            self.status_msg = format!("Selected {}", kind.title());
                        }
                        WorkshopAction::ToggleEstop => {
                            self.estop = !self.estop;
                            self.status_msg = if self.estop {
                                "EMERGENCY STOP LATCHED".to_string()
                            } else {
                                "ESTOP cleared - machines READY".to_string()
                            };
                        }
                        WorkshopAction::None => {}
                    }
                    ui.add_space(10.0);
                    if let Some(kind) = self.selected_station {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                draw_hardware_diagram(
                                    ui,
                                    kind,
                                    egui::vec2(260.0, 145.0),
                                    self.estop,
                                    TEAL,
                                );
                                ui.vertical(|ui| {
                                    ui.strong(kind.title());
                                    ui.label(kind.subtitle());
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        match kind {
                                            HardwareKind::Printer3d => {
                                                if ui.button("Probe Klipper (Moonraker)").clicked() {
                                                    self.run_command(
                                                        "printer_live",
                                                        Command::PrinterLive { url: String::new() },
                                                    );
                                                }
                                            }
                                            HardwareKind::CncRouter => {
                                                if ui.button("Probe GRBL (Serial)").clicked() {
                                                    self.run_command(
                                                        "cnc_probe",
                                                        Command::CncProbe { port: String::new() },
                                                    );
                                                }
                                            }
                                            HardwareKind::Oscilloscope => {
                                                if ui.button("Live SCPI *IDN?").clicked() {
                                                    self.run_command(
                                                        "scope_idn",
                                                        Command::ScopeIdn { addr: String::new() },
                                                    );
                                                }
                                            }
                                            HardwareKind::LogicAnalyzer => {
                                                if ui.button("Sample 4-Pin").clicked() {
                                                    self.run_command(
                                                        "logic_capture",
                                                        Command::LogicCapture {
                                                            port: String::new(),
                                                            hz: 1000,
                                                            samples: 32,
                                                        },
                                                    );
                                                }
                                            }
                                            HardwareKind::BusPirate if ui.button("Probe Bus Pirate").clicked() => {
                                                self.run_command(
                                                    "bus_probe",
                                                    Command::BusProbe { port: String::new() },
                                                );
                                            }
                                            _ => {}
                                        }
                                        let src_id = match kind {
                                            HardwareKind::Oscilloscope => Some("oscilloscope"),
                                            HardwareKind::LogicAnalyzer => Some("logic-analyzer"),
                                            HardwareKind::BusPirate => Some("bus-pirate"),
                                            HardwareKind::Multimeter => Some("multimeter"),
                                            HardwareKind::FunctionGenerator => Some("function-generator"),
                                            HardwareKind::SpectrumAnalyzer => Some("spectrum-analyzer"),
                                            HardwareKind::JtagSwd => Some("jtag-swd"),
                                            HardwareKind::Printer3d | HardwareKind::CncRouter => None,
                                        };
                                        if let Some(id) = src_id {
                                            if ui.button("Capture Demo JSON").clicked() {
                                                let dir = std::env::temp_dir()
                                                    .join("makercraft-captures")
                                                    .to_string_lossy()
                                                    .to_string();
                                                self.run_command(
                                                    &format!("instrument_capture({id})"),
                                                    Command::InstrumentCapture {
                                                        source: id.to_string(),
                                                        dir,
                                                    },
                                                );
                                            }
                                        }
                                    });
                                });
                            });
                        });
                    }
                    if !self.last_probe_json.is_empty() {
                        ui.add_space(6.0);
                        ui.strong("Last Probe / Capture Output");
                        egui::ScrollArea::vertical()
                            .id_salt("workshop_probe_json")
                            .max_height(140.0)
                            .show(ui, |ui| {
                                ui.monospace(&self.last_probe_json);
                            });
                    }
                });
            }
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
                                ui.label(egui::RichText::new(format!("- {w}")).size(17.0));
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
                egui::ScrollArea::vertical().id_salt("machines_scroll").show(ui, |ui| {
                    big_heading(ui, "Machines & Safety Interlock");
                    body_text(ui, MakerEngine::reference_rig());
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if self.estop {
                            if ui.button("Reset ESTOP").clicked() {
                                self.estop = false;
                                self.status_msg = "ESTOP cleared - machines READY".to_string();
                            }
                            ui.colored_label(ROSE, "ESTOP ACTIVE - all motion cut");
                        } else if ui.button("Emergency Kill All Spindles").clicked() {
                            self.estop = true;
                            self.status_msg = "EMERGENCY STOP: ALL MOTORS CUT".to_string();
                        }
                        if ui.button("Probe Klipper (Moonraker)").clicked() {
                            self.run_command("printer_live", Command::PrinterLive { url: String::new() });
                        }
                        if ui.button("Probe GRBL (Serial)").clicked() {
                            self.run_command("cnc_probe", Command::CncProbe { port: String::new() });
                        }
                        if ui.button("Probe Bus Pirate").clicked() {
                            self.run_command("bus_probe", Command::BusProbe { port: String::new() });
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        for kind in [HardwareKind::Printer3d, HardwareKind::CncRouter, HardwareKind::BusPirate] {
                            ui.group(|ui| {
                                ui.set_width(290.0);
                                ui.strong(kind.title());
                                ui.small(kind.subtitle());
                                draw_hardware_diagram(ui, kind, egui::vec2(274.0, 150.0), self.estop, TEAL);
                            });
                        }
                    });
                    if !self.last_probe_json.is_empty() {
                        ui.add_space(8.0);
                        ui.strong("Last Probe / Command Output");
                        egui::ScrollArea::vertical()
                            .id_salt("machine_probe_json")
                            .max_height(160.0)
                            .show(ui, |ui| {
                                ui.monospace(&self.last_probe_json);
                            });
                    }
                });
            }

            BenchTab::Instruments => {
                egui::ScrollArea::vertical().id_salt("instruments_scroll").show(ui, |ui| {
                    big_heading(ui, "Electronics Bench & Test Instruments");
                    body_text(
                        ui,
                        "Pictorial bench studio for all 7 folded instruments. Capture writes a labelled DEMO JSON; live probes talk to SCPI/sigrok/Bus Pirate when attached.",
                    );
                    ui.add_space(8.0);
                    let pairs = [
                        ("oscilloscope", HardwareKind::Oscilloscope),
                        ("logic-analyzer", HardwareKind::LogicAnalyzer),
                        ("bus-pirate", HardwareKind::BusPirate),
                        ("multimeter", HardwareKind::Multimeter),
                        ("function-generator", HardwareKind::FunctionGenerator),
                        ("spectrum-analyzer", HardwareKind::SpectrumAnalyzer),
                        ("jtag-swd", HardwareKind::JtagSwd),
                    ];
                    ui.horizontal_wrapped(|ui| {
                        for (src_id, kind) in pairs {
                            let info = INSTRUMENT_SOURCES.iter().find(|s| s.id == src_id);
                            ui.group(|ui| {
                                ui.set_width(280.0);
                                ui.strong(kind.title());
                                ui.small(kind.subtitle());
                                draw_hardware_diagram(ui, kind, egui::vec2(264.0, 140.0), self.estop, GOLD);
                                if let Some(s) = info {
                                    ui.small(format!("{} ({})", s.role, s.former_repo));
                                }
                                ui.horizontal(|ui| {
                                    if ui.small_button("Capture Demo JSON").clicked() {
                                        let dir = std::env::temp_dir()
                                            .join("makercraft-captures")
                                            .to_string_lossy()
                                            .to_string();
                                        self.run_command(
                                            &format!("instrument_capture({src_id})"),
                                            Command::InstrumentCapture {
                                                source: src_id.to_string(),
                                                dir,
                                            },
                                        );
                                    }
                                    match src_id {
                                        "oscilloscope" if ui.small_button("Live SCPI *IDN?").clicked() => {
                                            self.run_command("scope_idn", Command::ScopeIdn { addr: String::new() });
                                        }
                                        "logic-analyzer" if ui.small_button("Sample 4-Pin").clicked() => {
                                            self.run_command(
                                                "logic_capture",
                                                Command::LogicCapture { port: String::new(), hz: 1000, samples: 32 },
                                            );
                                        }
                                        "bus-pirate" if ui.small_button("Probe Bus").clicked() => {
                                            self.run_command("bus_probe", Command::BusProbe { port: String::new() });
                                        }
                                        _ => {}
                                    }
                                });
                            });
                        }
                    });
                    if !self.last_probe_json.is_empty() {
                        ui.add_space(8.0);
                        ui.strong("Last Instrument / Capture Output");
                        egui::ScrollArea::vertical()
                            .id_salt("instrument_probe_json")
                            .max_height(180.0)
                            .show(ui, |ui| {
                                ui.monospace(&self.last_probe_json);
                            });
                    }
                });
            }

            BenchTab::ViennaBuying => {
                render_vienna_and_buying_tab(ui);
            }

            BenchTab::Gcode => {
                big_heading(ui, "G-code Editor");
                ui.horizontal(|ui| {
                    body_text(ui, &format!("Profile: {}", self.profile().name));
                    if ui.button("Analyze Toolpath Safety (F5)").clicked() {
                        self.analyze();
                    }
                    if ui.button("Open File...").clicked() {
                        self.file_modal_is_open = Some(true);
                    }
                    if ui.button("Save File...").clicked() {
                        self.file_modal_is_open = Some(false);
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.gcode_buffer);
                });
            }
            }
        });

        // Help Suite Windows
        render_shortcuts_window(ui.ctx(), &mut self.show_shortcuts);
        render_help_docs_window(
            ui.ctx(),
            &mut self.show_docs,
            &mut self.help_doc_tab,
            &mut self.selected_skill_idx,
        );
        render_licenses_window(ui.ctx(), &mut self.show_licenses);

        // Command Palette (Ctrl+K)
        if self.show_command_palette {
            let mut open = true;
            let mut close_palette = false;
            egui::Window::new("Command Palette (Ctrl+K)")
                .open(&mut open)
                .collapsible(false)
                .default_size([460.0, 320.0])
                .show(ui.ctx(), |ui| {
                    ui.text_edit_singleline(&mut self.palette_query)
                        .request_focus();
                    ui.separator();
                    let q = self.palette_query.to_lowercase();
                    let actions: &[(&str, &str)] = &[
                        ("workshop", "Switch to Workshop & ESD Workbench Panorama"),
                        ("lint", "Run G-code Safety Lint (F5)"),
                        ("preview", "Switch to 2D Preview"),
                        ("machines", "Switch to Machines & Pictorials"),
                        ("instruments", "Switch to Electronics Instruments"),
                        ("vienna", "Switch to Vienna Makerspaces & Buying Guide"),
                        ("chat", "Toggle AI Bench Assistant Chat (Ctrl+J)"),
                        ("probe_printer", "Probe Klipper Printer (Moonraker)"),
                        ("probe_cnc", "Probe GRBL CNC (Serial)"),
                        ("probe_bus", "Probe Bus Pirate (Serial)"),
                        ("probe_scope", "Capture Oscilloscope (SCPI *IDN?)"),
                        ("probe_logic", "Capture Logic Analyzer (4-Pin)"),
                        ("docs", "Open Project Files, Docs & Skills"),
                        ("estop", "Latch Emergency Stop (Esc)"),
                    ];
                    let mut first_match: Option<&str> = None;
                    for (id, label) in actions {
                        if q.is_empty() || label.to_lowercase().contains(&q) || id.contains(&q) {
                            if first_match.is_none() {
                                first_match = Some(*id);
                            }
                            if ui.button(*label).clicked() {
                                self.palette_query.clear();
                                close_palette = true;
                                match *id {
                                    "workshop" => self.active_tab = BenchTab::Workshop,
                                    "lint" => {
                                        self.analyze();
                                        self.active_tab = BenchTab::Safety;
                                    }
                                    "preview" => self.active_tab = BenchTab::Preview,
                                    "machines" => self.active_tab = BenchTab::Machines,
                                    "instruments" => self.active_tab = BenchTab::Instruments,
                                    "vienna" => self.active_tab = BenchTab::ViennaBuying,
                                    "chat" => self.chat.open = !self.chat.open,
                                    "probe_printer" => {
                                        self.run_command(
                                            "printer_live",
                                            Command::PrinterLive { url: String::new() },
                                        );
                                        self.active_tab = BenchTab::Machines;
                                    }
                                    "probe_cnc" => {
                                        self.run_command(
                                            "cnc_probe",
                                            Command::CncProbe {
                                                port: String::new(),
                                            },
                                        );
                                        self.active_tab = BenchTab::Machines;
                                    }
                                    "probe_bus" => {
                                        self.run_command(
                                            "bus_probe",
                                            Command::BusProbe {
                                                port: String::new(),
                                            },
                                        );
                                        self.active_tab = BenchTab::Machines;
                                    }
                                    "probe_scope" => {
                                        self.run_command(
                                            "scope_idn",
                                            Command::ScopeIdn {
                                                addr: String::new(),
                                            },
                                        );
                                        self.active_tab = BenchTab::Instruments;
                                    }
                                    "probe_logic" => {
                                        self.run_command(
                                            "logic_capture",
                                            Command::LogicCapture {
                                                port: String::new(),
                                                hz: 1000,
                                                samples: 32,
                                            },
                                        );
                                        self.active_tab = BenchTab::Instruments;
                                    }
                                    "docs" => self.show_docs = true,
                                    "estop" => {
                                        self.estop = true;
                                        self.status_msg = "EMERGENCY STOP LATCHED".to_string();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(id) = first_match {
                            if id == "lint" {
                                self.analyze();
                            }
                            close_palette = true;
                        }
                    }
                });
            if !open || close_palette {
                self.show_command_palette = false;
            }
        }

        // Open / Save G-code File Modal
        if let Some(is_open) = self.file_modal_is_open {
            let title = if is_open {
                "Open G-code File"
            } else {
                "Save G-code File"
            };
            let mut keep_open = true;
            egui::Window::new(title)
                .open(&mut keep_open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("File path:");
                    ui.text_edit_singleline(&mut self.file_path_input);
                    ui.horizontal(|ui| {
                        let btn = if is_open { "Load" } else { "Save" };
                        if ui.button(btn).clicked() {
                            if is_open {
                                match std::fs::read_to_string(&self.file_path_input) {
                                    Ok(contents) => {
                                        self.set_gcode(contents);
                                        self.active_tab = BenchTab::Preview;
                                        self.status_msg =
                                            format!("Loaded {}", self.file_path_input);
                                        self.file_modal_is_open = None;
                                    }
                                    Err(e) => {
                                        self.status_msg = format!("Open failed: {e}");
                                    }
                                }
                            } else {
                                match std::fs::write(&self.file_path_input, &self.gcode_buffer) {
                                    Ok(()) => {
                                        self.status_msg = format!("Saved {}", self.file_path_input);
                                        self.file_modal_is_open = None;
                                    }
                                    Err(e) => {
                                        self.status_msg = format!("Save failed: {e}");
                                    }
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.file_modal_is_open = None;
                        }
                    });
                });
            if !keep_open {
                self.file_modal_is_open = None;
            }
        }

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
