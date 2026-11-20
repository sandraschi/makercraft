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
    Pitfalls,
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
        Self::Pitfalls,
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
            Self::Pitfalls => "Pitfalls & Safety",
            Self::OtherCrafts => "Other Crafts",
            Self::Skills => "Skills (8)",
            Self::FullManual => "HELP.md",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideSubTab {
    BuyingMatrix,
    ViennaSpaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixCategory {
    Printers3d,
    Oscilloscopes,
    LogicAnalyzers,
    CncRouters,
}

impl MatrixCategory {
    const ALL: &'static [MatrixCategory] = &[
        Self::Printers3d,
        Self::Oscilloscopes,
        Self::LogicAnalyzers,
        Self::CncRouters,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Printers3d => "3D Printers",
            Self::Oscilloscopes => "Oscilloscopes (Display-less & Bench)",
            Self::LogicAnalyzers => "Logic Analyzers",
            Self::CncRouters => "Desktop CNCs",
        }
    }
}

pub struct BuyingGuideState {
    pub sub_tab: GuideSubTab,
    pub category: MatrixCategory,
    pub prefer_open_firmware: bool,
    pub prefer_enclosed: bool,
    pub prefer_displayless_usb: bool,
    pub prefer_budget_under_400: bool,
}

impl Default for BuyingGuideState {
    fn default() -> Self {
        Self {
            sub_tab: GuideSubTab::BuyingMatrix,
            category: MatrixCategory::Printers3d,
            prefer_open_firmware: true,
            prefer_enclosed: false,
            prefer_displayless_usb: false,
            prefer_budget_under_400: false,
        }
    }
}

struct MatrixRow {
    model: &'static str,
    price_eur: u32,
    form_factor: &'static str,
    interface_fw: &'static str,
    open_fw: bool,
    enclosed: bool,
    displayless: bool,
    tradeoffs: &'static str,
}

impl MatrixRow {
    fn score(&self, st: &BuyingGuideState) -> u8 {
        let mut s: i32 = 70;
        if st.prefer_open_firmware {
            s += if self.open_fw { 18 } else { -15 };
        }
        if st.prefer_enclosed {
            s += if self.enclosed { 15 } else { -12 };
        }
        if st.prefer_displayless_usb {
            s += if self.displayless { 18 } else { -10 };
        }
        if st.prefer_budget_under_400 {
            s += if self.price_eur <= 400 { 15 } else { -15 };
        }
        s.clamp(10, 100) as u8
    }
}

pub fn render_vienna_and_buying_tab(ui: &mut egui::Ui, state: &mut BuyingGuideState) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                state.sub_tab == GuideSubTab::BuyingMatrix,
                "Device Acquisition Matrices",
            )
            .clicked()
        {
            state.sub_tab = GuideSubTab::BuyingMatrix;
        }
        if ui
            .selectable_label(
                state.sub_tab == GuideSubTab::ViennaSpaces,
                "Vienna Makerspaces (How-To)",
            )
            .clicked()
        {
            state.sub_tab = GuideSubTab::ViennaSpaces;
        }
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("vienna_buying_scroll")
        .show(ui, |ui| match state.sub_tab {
            GuideSubTab::BuyingMatrix => render_buying_matrices(ui, state),
            GuideSubTab::ViennaSpaces => render_vienna_spaces(ui),
        });
}

fn render_buying_matrices(ui: &mut egui::Ui, st: &mut BuyingGuideState) {
    ui.heading("Device Acquisition Decision Matrices");
    ui.label("Toggle your priorities below to re-score models live (0-100 Fit Score).");
    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        for cat in MatrixCategory::ALL {
            if ui
                .selectable_label(st.category == *cat, cat.label())
                .clicked()
            {
                st.category = *cat;
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.strong("Priorities:");
        ui.checkbox(
            &mut st.prefer_open_firmware,
            "Open Firmware / sigrok / SCPI",
        );
        ui.checkbox(&mut st.prefer_enclosed, "Enclosed (ABS/ASA/Dust)");
        ui.checkbox(
            &mut st.prefer_displayless_usb,
            "Display-less / USB / Portable",
        );
        ui.checkbox(&mut st.prefer_budget_under_400, "Budget <= 400 EUR");
    });
    ui.separator();

    let rows: &[MatrixRow] = match st.category {
        MatrixCategory::Printers3d => &[
            MatrixRow {
                model: "Prusa MK4S",
                price_eur: 799,
                form_factor: "250x210x220 Open",
                interface_fw: "PrusaLink / Open Marlin",
                open_fw: true,
                enclosed: false,
                displayless: false,
                tradeoffs: "Best EU repairability & open hardware; add enclosure for ABS/ASA.",
            },
            MatrixRow {
                model: "Prusa CORE One",
                price_eur: 1199,
                form_factor: "250x220x270 CoreXY",
                interface_fw: "PrusaLink / Open",
                open_fw: true,
                enclosed: true,
                displayless: false,
                tradeoffs: "Enclosed CoreXY with open ecosystem; higher upfront cost.",
            },
            MatrixRow {
                model: "Bambu Lab A1 / A1 Mini",
                price_eur: 299,
                form_factor: "256x256x256 / 180^3",
                interface_fw: "Proprietary LAN/Cloud",
                open_fw: false,
                enclosed: false,
                displayless: false,
                tradeoffs: "Unbeatable turn-key PLA/PETG speed; closed firmware.",
            },
            MatrixRow {
                model: "Bambu Lab P1S / X1C",
                price_eur: 599,
                form_factor: "256x256x256 Enclosed",
                interface_fw: "Proprietary MQTT/LAN",
                open_fw: false,
                enclosed: true,
                displayless: false,
                tradeoffs: "Reliable ABS/ASA/PA-CF out of the box; proprietary parts/firmware.",
            },
            MatrixRow {
                model: "Sovol SV08 (Voron 2.4)",
                price_eur: 549,
                form_factor: "350x350x345 CoreXY",
                interface_fw: "Mainline Klipper + Moonraker",
                open_fw: true,
                enclosed: false,
                displayless: false,
                tradeoffs: "Huge Klipper build volume; needs enclosure panels + minor tuning.",
            },
            MatrixRow {
                model: "Voron 0.2 / 2.4 Kit",
                price_eur: 850,
                form_factor: "120^3 / 350^3 Enclosed",
                interface_fw: "Mainline Klipper + Moonraker",
                open_fw: true,
                enclosed: true,
                displayless: false,
                tradeoffs: "Gold-standard open CoreXY; requires 15-30h build & calibration.",
            },
        ],
        MatrixCategory::Oscilloscopes => &[
            MatrixRow {
                model: "Digilent Analog Discovery 3",
                price_eur: 379,
                form_factor: "USB-C Display-less",
                interface_fw: "WaveForms SDK / 2-Ch 30MHz + 16-Ch Logic",
                open_fw: true,
                enclosed: true,
                displayless: true,
                tradeoffs: "Best all-in-one USB lab (scope + logic + AWG + PSU); 30 MHz BW.",
            },
            MatrixRow {
                model: "PicoScope 2204A / 2206B",
                price_eur: 165,
                form_factor: "USB Display-less",
                interface_fw: "PicoSDK / Deep Memory FFT",
                open_fw: true,
                enclosed: true,
                displayless: true,
                tradeoffs: "Rock-solid PC software & API; 2 channels only on entry tier.",
            },
            MatrixRow {
                model: "LOTO OSC802 / OSCH02",
                price_eur: 110,
                form_factor: "USB Display-less",
                interface_fw: "USB PC App / 2-Ch",
                open_fw: false,
                enclosed: true,
                displayless: true,
                tradeoffs: "Very cheap & compact; basic software, limited trigger depth.",
            },
            MatrixRow {
                model: "Rigol DHO804 (12-bit)",
                price_eur: 369,
                form_factor: "4-Ch Benchtop / HDMI / USB-C PD",
                interface_fw: "SCPI over LAN & USBTMC",
                open_fw: true,
                enclosed: true,
                displayless: false,
                tradeoffs: "Best value 4-ch 12-bit scope; runs headless over SCPI/HDMI/Web.",
            },
            MatrixRow {
                model: "Siglent SDS804X HD",
                price_eur: 439,
                form_factor: "4-Ch 12-bit Benchtop",
                interface_fw: "SCPI over LAN & USBTMC",
                open_fw: true,
                enclosed: true,
                displayless: false,
                tradeoffs: "Ultra-low noise floor & crisp FFT; slightly above 400 EUR.",
            },
        ],
        MatrixCategory::LogicAnalyzers => &[
            MatrixRow {
                model: "FX2LP 24MHz 8-Ch Clone",
                price_eur: 15,
                form_factor: "USB Display-less",
                interface_fw: "sigrok / PulseView (fx2lafw)",
                open_fw: true,
                enclosed: true,
                displayless: true,
                tradeoffs: "Best 15 EUR tool for UART/I2C/slow SPI; no 1.8V support.",
            },
            MatrixRow {
                model: "DreamSourceLab DSLogic Plus",
                price_eur: 129,
                form_factor: "USB-C Display-less",
                interface_fw: "sigrok / DSView (100-400MHz)",
                open_fw: true,
                enclosed: true,
                displayless: true,
                tradeoffs: "Shielded fly-leads & adjustable thresholds (1.2-5V).",
            },
            MatrixRow {
                model: "Saleae Logic Pro 8",
                price_eur: 699,
                form_factor: "USB-C Display-less",
                interface_fw: "Saleae Logic 2 (Analog + Digital)",
                open_fw: false,
                enclosed: true,
                displayless: true,
                tradeoffs: "Best UI & mixed-signal capture; expensive, proprietary app.",
            },
            MatrixRow {
                model: "Bus Pirate 5 (RP2040)",
                price_eur: 45,
                form_factor: "USB-C Pocket Bridge",
                interface_fw: "Open Firmware + Built-in Pullups",
                open_fw: true,
                enclosed: true,
                displayless: true,
                tradeoffs: "Interactive I2C/SPI/UART + programmable PSU; not a deep LA.",
            },
        ],
        MatrixCategory::CncRouters => &[
            MatrixRow {
                model: "Genmitsu 3018-PROVer V2",
                price_eur: 260,
                form_factor: "300x180x45 Open",
                interface_fw: "GRBL 1.1h + Limit Switches",
                open_fw: true,
                enclosed: false,
                displayless: false,
                tradeoffs: "Great starter for PCBs, acrylic & softwood; too flexy for aluminum.",
            },
            MatrixRow {
                model: "Makera Carvera Air / Carvera",
                price_eur: 1399,
                form_factor: "Enclosed Desktop CNC",
                interface_fw: "GRBL-compatible + Auto-Probe",
                open_fw: false,
                enclosed: true,
                displayless: false,
                tradeoffs: "Quiet, enclosed, dust-contained; premium price.",
            },
            MatrixRow {
                model: "RatRig Stronghold / PrintNC",
                price_eur: 1450,
                form_factor: "Rigid Steel / Extrusion",
                interface_fw: "GRBLHAL / LinuxCNC",
                open_fw: true,
                enclosed: false,
                displayless: false,
                tradeoffs: "True aluminum/brass milling rigidity; needs shop space + dust shoe.",
            },
        ],
    };

    let mut ranked: Vec<(&MatrixRow, u8)> = rows.iter().map(|r| (r, r.score(st))).collect();
    ranked.sort_by_key(|a| std::cmp::Reverse(a.1));

    egui::Grid::new("buying_matrix_grid")
        .striped(true)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.strong("Fit");
            ui.strong("Model");
            ui.strong("EUR");
            ui.strong("Form / Envelope");
            ui.strong("Firmware / API");
            ui.strong("Key Tradeoffs");
            ui.end_row();

            for (row, score) in ranked {
                let col = if score >= 85 {
                    egui::Color32::from_rgb(120, 255, 170)
                } else if score >= 65 {
                    egui::Color32::from_rgb(255, 205, 105)
                } else {
                    egui::Color32::from_rgb(255, 135, 135)
                };
                ui.colored_label(col, format!("{score}/100"));
                ui.strong(row.model);
                ui.label(format!("~{}", row.price_eur));
                ui.label(row.form_factor);
                ui.label(row.interface_fw);
                ui.label(row.tradeoffs);
                ui.end_row();
            }
        });
}

fn render_vienna_spaces(ui: &mut egui::Ui) {
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
                        ui.label("- Drag-and-drop any .gcode or .stl file directly onto MakerCraft to load and preview it.");
                        ui.label("- On 3D Orbit Preview, drag to rotate, scroll to zoom, and use Z-Layer Filter for depth passes.");
                        ui.label("- Run `gcode_check` in CI: it exits with code 2 if passes_safety is false.");
                    }
                    HelpDocTab::Pitfalls => {
                        ui.strong("5. Common Pitfalls & Critical Safety");
                        ui.colored_label(egui::Color32::from_rgb(255, 135, 135), "- Oscilloscope Ground Clip: Most benchtop scopes have earth-referenced BNC shields. Clipping GND to a non-ground node creates a dead short!");
                        ui.label("- CNC Rapid Plunge: Never use G0 downward into stock (G0 Z-5). Always rapid to safe Z, then plunge with G1 F150..F300.");
                        ui.label("- Logic Analyzer Thresholds: Cheap 24 MHz FX2LP clones are NOT 1.8V safe without a level shifter.");
                        ui.label("- PTFE Hotend Limit: Printing >260 C on PTFE-lined hotends degrades tubing and releases toxic fumes.");
                        ui.label("- Laser Cutters (Vienna Makerspaces): NEVER cut PVC or vinyl — it produces corrosive chlorine gas.");
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
