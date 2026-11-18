//! Vienna Makerspaces directory, Machine Buying Guide, and the
//! mandatory 5-item Craft Help Suite (Command Palette, Shortcuts,
//! Project Files & Docs + Skills, Licenses & Notices, About).

use makercraft_core::{skills::skills_list, HELP_MD};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpDocTab {
    GetStarted,
    Functions,
    Usage,
    Tips,
    OtherCrafts,
    Skills,
    FullManual,
}

impl HelpDocTab {
    pub const ALL: &'static [HelpDocTab] = &[
        Self::GetStarted,
        Self::Functions,
        Self::Usage,
        Self::Tips,
        Self::OtherCrafts,
        Self::Skills,
        Self::FullManual,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::GetStarted => "Get Started",
            Self::Functions => "Functions",
            Self::Usage => "Usage (GUI/CLI/MCP)",
            Self::Tips => "Tips & Tricks",
            Self::OtherCrafts => "Other Crafts",
            Self::Skills => "Skills (8)",
            Self::FullManual => "HELP.md",
        }
    }
}

pub fn render_vienna_and_buying_tab(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_salt("vienna_buying_scroll")
        .show(ui, |ui| {
            ui.heading("Vienna Makerspaces — Complete Reference & How-To");
            ui.label("Verified local workshops in Vienna for laser cutting, CNC milling, 3D printing, and electronics benches.");
            ui.add_space(6.0);

            for (name, addr, focus, how_to, best_for) in [
                (
                    "Metalab",
                    "Rathausstrasse 6, 1010 Wien (1st District)",
                    "Community hackerspace (founded 2006): electronics lab, soldering/SMD rework, 3D printers, CNC/laser, software & hardware hacking.",
                    "Drop in during open evenings or weekly meetups and ask for a tour. Monthly membership grants 24/7 keycard access after intro.",
                    "Embedded debugging (Bus Pirate, logic analyzer, scope), Linux/Klipper builds, late-night prototyping.",
                ),
                (
                    "Happylab",
                    "Schoengasse 15-17, 1020 Wien (2nd District / Stuwerviertel)",
                    "900+ m2 structured Fab Lab: industrial laser cutters, FDM/SLA 3D printers, CNC routers, wood & metal workshops, vinyl/textile.",
                    "Book an online Einschulung (safety briefing) per machine class (Laser, CNC, Wood, 3D Print), then reserve machine slots via the member portal.",
                    "Laser-cutting acrylic/birch plywood, repeatable CNC jobs, beginners wanting formal machine training.",
                ),
                (
                    "Maker Austria",
                    "Schoenbrunner Strasse 125, 1050 Wien (5th District)",
                    "800 m2 non-profit open workshop: wood, metal, ceramics/pottery, textiles, 3D printing, laser cutting, and electronics.",
                    "Register for a workshop orientation and machine-specific safety briefings; day passes and memberships available.",
                    "Mixed-material builds (wood/metal enclosures + electronics), ceramics, collaborative craft.",
                ),
            ] {
                ui.group(|ui| {
                    ui.strong(format!("{name}  —  {addr}"));
                    ui.label(format!("Focus: {focus}"));
                    ui.label(format!("How to start: {how_to}"));
                    ui.label(format!("Best for: {best_for}"));
                });
                ui.add_space(4.0);
            }

            ui.group(|ui| {
                ui.strong("Before You Go: Makerspace Prep Checklist");
                ui.label("1. Lint G-code at home with MakerCraft (`gcode_lint` / `F5`) so envelope and plunge errors are caught before arriving.");
                ui.label("2. For laser cutters: export clean DXF/SVG (mm units, hairline strokes, text converted to paths). Never cut PVC (toxic chlorine gas).");
                ui.label("3. Bring your own consumables (filament, clean plywood/acrylic, endmills, USB stick) and respect all safety briefings & dust extraction.");
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(6.0);

            ui.heading("Maker Machine & Bench Instrument Buying Guide");
            ui.label("Practical hardware recommendations and common traps to avoid:");
            ui.add_space(6.0);

            for (cat, picks, avoid) in [
                (
                    "3D Printers (FDM)",
                    "Turn-key: Bambu Lab A1 / A1 Mini or P1S / X1C; Open/EU: Prusa MK4S / CORE One; Klipper tinkerer: Voron 0.2/2.4 or Sovol SV06/SV08.",
                    "Avoid unenclosed bed-slingers with proprietary hotend PCBs and locked firmware, or ultra-cheap acrylic frames.",
                ),
                (
                    "Desktop CNC Routers",
                    "Light PCB/wood/acrylic: Genmitsu 3018-PROVer V2 (GRBL 1.1h + limit switches + Z-probe). Aluminum/rigid: RatRig Stronghold, Shapeoko 4/5, PrintNC.",
                    "Avoid unguarded spindles without E-stop or expecting a stock 3018 to hog aluminum cleanly. Always get ER11/ER20 collets + dust shoe.",
                ),
                (
                    "Oscilloscopes",
                    "Best value 4-ch: Rigol DHO804 / DHO914 or Siglent SDS804X HD (12-bit, >= 70-100 MHz, SCPI over LAN/USBTMC).",
                    "Avoid 1-channel pocket toy scopes for protocol or power debugging. Remember earth-referenced BNC ground-clip safety.",
                ),
                (
                    "Logic Analyzers",
                    "Budget: 24 MHz 8-ch FX2LP (~15 EUR) + sigrok/PulseView. High-speed: DSLogic Plus / U3Pro16 or Saleae Logic Pro 8/16.",
                    "Avoid sampling slower than 4x-10x your bus clock, and never probe 1.8V logic with a 5V-only clone without level shifting.",
                ),
                (
                    "Multimeters, Soldering & Debug Probes",
                    "DMM: Brymen BM235/BM869s, Fluke 117/87V, UNI-T UT61E+ (check CAT III + HRC fuses). Soldering: Pinecil V2 or JBC C245 station. Debug: Bus Pirate 5, RPi Debugprobe / ST-Link V3.",
                    "Avoid un-fused cheap meters near mains voltage and ungrounded soldering irons on sensitive CMOS boards.",
                ),
            ] {
                ui.group(|ui| {
                    ui.strong(cat);
                    ui.label(format!("Recommended: {picks}"));
                    ui.colored_label(egui::Color32::from_rgb(255, 185, 130), format!("Watch out: {avoid}"));
                });
                ui.add_space(4.0);
            }
        });
}

pub fn render_help_docs_window(
    ctx: &egui::Context,
    open: &mut bool,
    active_tab: &mut HelpDocTab,
    selected_skill_idx: &mut usize,
) {
    if !*open {
        return;
    }
    egui::Window::new("Project Files & Docs — MakerCraft Help Center")
        .open(open)
        .default_size([760.0, 520.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for tab in HelpDocTab::ALL {
                    if ui.selectable_label(*active_tab == *tab, tab.label()).clicked() {
                        *active_tab = *tab;
                    }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("help_docs_scroll")
                .show(ui, |ui| match *active_tab {
                    HelpDocTab::GetStarted => {
                        ui.strong("1. Get Started (5-Step Tour)");
                        ui.label("1) Launch MakerCraft: starts on 2D Preview with a reference 3018 CNC square.");
                        ui.label("2) Pick a profile: 3018 CNC (GRBL 1.1h) or Ender-3 (Klipper/Moonraker).");
                        ui.label("3) Load G-code via File -> Open G-code... (Ctrl+O) or edit in the G-code tab.");
                        ui.label("4) Press F5 (Run Safety Lint) to verify envelopes, plunge safety, and feedrates.");
                        ui.label("5) Inspect hardware diagrams in Machines & Instruments, or press Ctrl+J for AI Chat.");
                    }
                    HelpDocTab::Functions => {
                        ui.strong("2. Core Functions & Architecture");
                        ui.label("- makercraft-core: Modal G-code parser (G0/G1/G90/G91/G20/G21/G2/G3), safety linter, job estimator, 8 bundled skills.");
                        ui.label("- makercraft-hal: Moonraker HTTP, GRBL serial, Bus Pirate, SCPI/USBTMC scope, sigrok logic analyzer.");
                        ui.label("- makercraft-mcp: Unified CLI + MCP 2024-11-05 stdio server (18 tools + skill:// resources).");
                        ui.label("- makercraft-app: Pure-Rust egui workstation with pictorial hardware studio and local-first AI chat.");
                    }
                    HelpDocTab::Usage => {
                        ui.strong("3. Usage (GUI, CLI & MCP)");
                        ui.monospace("makercraft-cli status\nmakercraft-cli lint sample.gcode --profile printer\nmakercraft-cli check sample.gcode\nmakercraft-mcp instrument-sources\nmakercraft-mcp instrument-capture oscilloscope --dir %TEMP%\\captures\nmakercraft-mcp skills list\nmakercraft-mcp mcp");
                    }
                    HelpDocTab::Tips => {
                        ui.strong("4. Tips & Tricks");
                        ui.label("- Use Ctrl+K (Command Palette) to jump to any tab, probe, or sample G-code.");
                        ui.label("- On 2D Preview, scroll to zoom, drag to pan, Ctrl+0 to fit the bed.");
                        ui.label("- Never attach an earth-referenced scope ground clip to a non-ground node.");
                        ui.label("- Run `gcode_check` in CI: it exits with code 2 if passes_safety is false.");
                    }
                    HelpDocTab::OtherCrafts => {
                        ui.strong("5. Other Storytold Craft Apps");
                        ui.label("- CADCraft: 3D parametric CAD & DXF/STEP/STL export -> mill or slice in MakerCraft.");
                        ui.label("- RobotCraft: URDF kinematics & Yahboom/VLA robotics (separate from bench/fab).");
                        ui.label("- DevCraft: Fleet orchestration, git, ports, worktrees.");
                        ui.label("- RevCraft: Firmware binary reverse engineering.");
                    }
                    HelpDocTab::Skills => {
                        let skills = skills_list();
                        ui.horizontal_wrapped(|ui| {
                            for (idx, s) in skills.iter().enumerate() {
                                if ui.selectable_label(*selected_skill_idx == idx, s.name).clicked() {
                                    *selected_skill_idx = idx;
                                }
                            }
                        });
                        ui.separator();
                        if let Some(s) = skills.get(*selected_skill_idx) {
                            ui.strong(s.name);
                            ui.small(s.description);
                            ui.add_space(4.0);
                            ui.monospace(s.skill_md);
                        }
                    }
                    HelpDocTab::FullManual => {
                        ui.monospace(HELP_MD);
                    }
                });
        });
}

pub fn render_shortcuts_window(ctx: &egui::Context, open: &mut bool) {
    if !*open {
        return;
    }
    egui::Window::new("Keyboard Shortcuts")
        .open(open)
        .default_size([460.0, 360.0])
        .show(ctx, |ui| {
            for (key, desc) in [
                ("Ctrl+K / Ctrl+Shift+P", "Open Command Palette"),
                ("Ctrl+J", "Toggle AI Bench Assistant Chat Dock"),
                ("F1 / Ctrl+/", "Open Keyboard Shortcuts"),
                ("F5", "Run G-code Safety Lint"),
                ("Ctrl+N", "New Blank G-code Program"),
                ("Ctrl+O", "Open G-code File..."),
                ("Ctrl+S", "Save G-code File..."),
                ("Ctrl+Z / Ctrl+Y", "Undo / Redo G-code Edit"),
                ("Ctrl+= / Ctrl+-", "Zoom In / Zoom Out 2D Preview"),
                ("Ctrl+0 / Ctrl+1", "Fit Bed to Window / Reset 100% Zoom"),
                ("Esc", "Latch Emergency Stop (E-Stop)"),
            ] {
                ui.horizontal(|ui| {
                    ui.monospace(format!("{key:<24}"));
                    ui.label(desc);
                });
            }
        });
}

pub fn render_licenses_window(ctx: &egui::Context, open: &mut bool) {
    if !*open {
        return;
    }
    egui::Window::new("Licenses & Notices")
        .open(open)
        .default_size([520.0, 340.0])
        .show(ctx, |ui| {
            ui.strong("MakerCraft — Dual Licensed: MIT OR Apache-2.0");
            ui.label("Copyright (c) Antigravity Craft Collective");
            ui.add_space(6.0);
            ui.strong("Safe Rust Compliance");
            ui.label("All workspace crates enforce #![deny(clippy::unwrap_used, expect_used, panic, unimplemented, todo)].");
            ui.add_space(6.0);
            ui.strong("Key Open-Source Crates");
            ui.label("- eframe / egui 0.37 (MIT / Apache-2.0)");
            ui.label("- serde / serde_json (MIT / Apache-2.0)");
            ui.label("- clap 4.5 (MIT / Apache-2.0)");
            ui.label("- reqwest 0.12 (MIT / Apache-2.0)");
            ui.label("- serialport / rusb / rusqlite / chrono / thiserror");
        });
}
