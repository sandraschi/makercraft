//! Bench Readiness Dashboard, Interactive Handholding Guides, and SQLite-backed
//! Maker Project Kanban board.

use makercraft_core::{ToolpathLintReport, INSTRUMENT_SOURCES};
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KanbanCol {
    IdeaCad = 0,
    GcodeLint = 1,
    Fabricating = 2,
    BenchDebug = 3,
    Done = 4,
}

impl KanbanCol {
    pub const ALL: &'static [KanbanCol] = &[
        Self::IdeaCad,
        Self::GcodeLint,
        Self::Fabricating,
        Self::BenchDebug,
        Self::Done,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::IdeaCad => "Idea / CAD",
            Self::GcodeLint => "G-code Lint",
            Self::Fabricating => "Fabricating",
            Self::BenchDebug => "Bench Debug",
            Self::Done => "Done",
        }
    }

    #[must_use]
    pub fn from_i64(v: i64) -> Self {
        match v {
            0 => Self::IdeaCad,
            1 => Self::GcodeLint,
            2 => Self::Fabricating,
            3 => Self::BenchDebug,
            _ => Self::Done,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KanbanCard {
    pub id: i64,
    pub title: String,
    pub station: String,
    pub notes: String,
    pub col: KanbanCol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardAction {
    None,
    LoadSafeCnc,
    LoadBadPlungeDemo,
    LoadPrinterCalibration,
    SelectCncProfile,
    SelectPrinterProfile,
    RunLint,
    OpenPreview,
    OpenWorkshop,
    OpenInstruments,
    OpenChat,
    OpenVienna,
}

pub struct DashboardState {
    conn: Option<Connection>,
    pub cards: Vec<KanbanCard>,
    pub new_title: String,
    pub new_station: String,
    pub new_notes: String,
    pub active_guide: usize,
    pub storage_note: String,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardState {
    #[must_use]
    pub fn new() -> Self {
        let (conn, storage_note) = open_kanban_db();
        let mut state = Self {
            conn,
            cards: Vec::new(),
            new_title: String::new(),
            new_station: "3018 CNC".to_string(),
            new_notes: String::new(),
            active_guide: 0,
            storage_note,
        };
        state.load_or_seed();
        state
    }

    fn load_or_seed(&mut self) {
        let Some(conn) = &self.conn else {
            self.cards = default_seed_cards();
            return;
        };
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM kanban_cards", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            for c in default_seed_cards() {
                let _ = conn.execute(
                    "INSERT INTO kanban_cards (title, station, notes, col) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![c.title, c.station, c.notes, c.col as i64],
                );
            }
        }
        self.reload();
    }

    fn reload(&mut self) {
        let Some(conn) = &self.conn else {
            return;
        };
        let Ok(mut stmt) =
            conn.prepare("SELECT id, title, station, notes, col FROM kanban_cards ORDER BY id ASC")
        else {
            return;
        };
        let Ok(rows) = stmt.query_map([], |r| {
            Ok(KanbanCard {
                id: r.get(0)?,
                title: r.get(1)?,
                station: r.get(2)?,
                notes: r.get(3)?,
                col: KanbanCol::from_i64(r.get(4)?),
            })
        }) else {
            return;
        };
        self.cards = rows.filter_map(Result::ok).collect();
    }

    pub fn add_card(&mut self, title: String, station: String, notes: String) {
        let t = title.trim().to_string();
        if t.is_empty() {
            return;
        }
        let s = if station.trim().is_empty() {
            "Bench".to_string()
        } else {
            station.trim().to_string()
        };
        let n = notes.trim().to_string();
        if let Some(conn) = &self.conn {
            let _ = conn.execute(
                "INSERT INTO kanban_cards (title, station, notes, col) VALUES (?1, ?2, ?3, 0)",
                rusqlite::params![t, s, n],
            );
            self.reload();
        } else {
            let next_id = self.cards.iter().map(|c| c.id).max().unwrap_or(0) + 1;
            self.cards.push(KanbanCard {
                id: next_id,
                title: t,
                station: s,
                notes: n,
                col: KanbanCol::IdeaCad,
            });
        }
    }

    pub fn move_card(&mut self, id: i64, delta: i64) {
        if let Some(card) = self.cards.iter_mut().find(|c| c.id == id) {
            let next = ((card.col as i64) + delta).clamp(0, 4);
            card.col = KanbanCol::from_i64(next);
            if let Some(conn) = &self.conn {
                let _ = conn.execute(
                    "UPDATE kanban_cards SET col = ?1 WHERE id = ?2",
                    rusqlite::params![next, id],
                );
            }
        }
    }

    pub fn delete_card(&mut self, id: i64) {
        self.cards.retain(|c| c.id != id);
        if let Some(conn) = &self.conn {
            let _ = conn.execute(
                "DELETE FROM kanban_cards WHERE id = ?1",
                rusqlite::params![id],
            );
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        profile_name: &str,
        estop: bool,
        lint: Option<&ToolpathLintReport>,
    ) -> DashboardAction {
        let mut action = DashboardAction::None;

        egui::ScrollArea::vertical()
            .id_salt("dashboard_scroll")
            .show(ui, |ui| {
                ui.heading("MakerCraft Bench Dashboard & Project Kanban");
                ui.small(&self.storage_note);
                ui.add_space(6.0);

                // 1. Readiness KPI strip
                ui.horizontal_wrapped(|ui| {
                    ui.group(|ui| {
                        ui.strong("Active Profile");
                        ui.label(profile_name);
                    });
                    ui.group(|ui| {
                        ui.strong("Safety Interlock");
                        if estop {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 120, 120),
                                "ESTOP LATCHED",
                            );
                        } else {
                            ui.colored_label(
                                egui::Color32::from_rgb(120, 255, 170),
                                "READY",
                            );
                        }
                    });
                    ui.group(|ui| {
                        ui.strong("G-code Lint");
                        match lint {
                            Some(r) if r.passes_safety => {
                                ui.colored_label(
                                    egui::Color32::from_rgb(120, 255, 170),
                                    format!(
                                        "PASS ({} segs, ~{:.1} min)",
                                        r.segment_count, r.estimated_minutes
                                    ),
                                );
                            }
                            Some(r) => {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 120, 120),
                                    format!(
                                        "FAIL ({} errs, {} warns)",
                                        r.errors.len(),
                                        r.warnings.len()
                                    ),
                                );
                            }
                            None => {
                                ui.label("Not run yet (F5)");
                            }
                        }
                    });
                    ui.group(|ui| {
                        ui.strong("Bench Instruments");
                        ui.label(format!("{} sources ready", INSTRUMENT_SOURCES.len()));
                    });
                    ui.group(|ui| {
                        ui.strong("Active Builds");
                        let open_builds = self
                            .cards
                            .iter()
                            .filter(|c| c.col != KanbanCol::Done)
                            .count();
                        ui.label(format!("{open_builds} in progress / {} total", self.cards.len()));
                    });
                });

                ui.add_space(10.0);

                // 2. Interactive Handholding Guides
                ui.group(|ui| {
                    ui.strong("Interactive Handholding Guides (Step-by-Step)");
                    ui.horizontal(|ui| {
                        for (idx, lbl) in [
                            (0, "1. First 3018 CNC Cut"),
                            (1, "2. First Klipper 3D Print"),
                            (2, "3. Electronics Bench Debug"),
                        ] {
                            if ui.selectable_label(self.active_guide == idx, lbl).clicked() {
                                self.active_guide = idx;
                            }
                        }
                    });
                    ui.separator();
                    match self.active_guide {
                        0 => {
                            ui.label("Step 1: Select 3018 CNC profile -> Step 2: Load safe square or test the rapid-plunge blocker -> Step 3: Lint (F5) -> Step 4: Inspect 2D Preview.");
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Select 3018 CNC").clicked() {
                                    action = DashboardAction::SelectCncProfile;
                                }
                                if ui.button("Load Safe CNC Square").clicked() {
                                    action = DashboardAction::LoadSafeCnc;
                                }
                                if ui.button("Test Rapid-Plunge Blocker (Unsafe)").clicked() {
                                    action = DashboardAction::LoadBadPlungeDemo;
                                }
                                if ui.button("Run Safety Lint (F5)").clicked() {
                                    action = DashboardAction::RunLint;
                                }
                                if ui.button("Open 2D Preview").clicked() {
                                    action = DashboardAction::OpenPreview;
                                }
                            });
                        }
                        1 => {
                            ui.label("Step 1: Select Ender-3 Klipper profile -> Step 2: Load calibration frame -> Step 3: Verify nozzle/bed temps & Z >= 0 -> Step 4: Check Vienna Makerspaces if printing on shared hardware.");
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Select Ender-3").clicked() {
                                    action = DashboardAction::SelectPrinterProfile;
                                }
                                if ui.button("Load Klipper Calibration Frame").clicked() {
                                    action = DashboardAction::LoadPrinterCalibration;
                                }
                                if ui.button("Run Safety Lint (F5)").clicked() {
                                    action = DashboardAction::RunLint;
                                }
                                if ui.button("Open Vienna & Buying Guide").clicked() {
                                    action = DashboardAction::OpenVienna;
                                }
                            });
                        }
                        _ => {
                            ui.label("Step 1: Open the Workshop Panorama to inspect wiring & E-Stop -> Step 2: Capture Oscilloscope / Logic / Bus Pirate telemetry -> Step 3: Ask the skill-grounded AI Assistant (Ctrl+J) about pinouts or signals.");
                            ui.horizontal_wrapped(|ui| {
                                if ui.button("Open Workshop Panorama").clicked() {
                                    action = DashboardAction::OpenWorkshop;
                                }
                                if ui.button("Open Instruments Tab").clicked() {
                                    action = DashboardAction::OpenInstruments;
                                }
                                if ui.button("Open AI Bench Chat (Ctrl+J)").clicked() {
                                    action = DashboardAction::OpenChat;
                                }
                            });
                        }
                    }
                });

                ui.add_space(10.0);

                // 3. New Kanban Card bar
                ui.group(|ui| {
                    ui.strong("Add Maker Project Card");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Title:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_title)
                                .desired_width(210.0)
                                .hint_text("e.g. RP2040 Macro Pad Case"),
                        );
                        ui.label("Station:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_station)
                                .desired_width(120.0),
                        );
                        ui.label("Notes:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_notes)
                                .desired_width(220.0)
                                .hint_text("toolpath, filament, or bus notes"),
                        );
                        if ui.button("Add Card").clicked() {
                            let t = self.new_title.clone();
                            let s = self.new_station.clone();
                            let n = self.new_notes.clone();
                            self.add_card(t, s, n);
                            self.new_title.clear();
                            self.new_notes.clear();
                        }
                    });
                });

                ui.add_space(8.0);

                // 4. 5-Column Kanban Board
                let mut move_req: Option<(i64, i64)> = None;
                let mut del_req: Option<i64> = None;
                ui.columns(KanbanCol::ALL.len(), |cols| {
                    for (idx, col_kind) in KanbanCol::ALL.iter().enumerate() {
                        let col_ui = &mut cols[idx];
                        col_ui.group(|ui| {
                            let n_cards = self.cards.iter().filter(|c| c.col == *col_kind).count();
                            ui.strong(format!("{} ({n_cards})", col_kind.label()));
                            ui.separator();
                            for card in self.cards.iter().filter(|c| c.col == *col_kind) {
                                ui.group(|ui| {
                                    ui.strong(&card.title);
                                    ui.small(format!("Station: {}", card.station));
                                    if !card.notes.is_empty() {
                                        ui.small(&card.notes);
                                    }
                                    ui.horizontal(|ui| {
                                        if idx > 0 && ui.small_button("<-").clicked() {
                                            move_req = Some((card.id, -1));
                                        }
                                        if idx + 1 < KanbanCol::ALL.len()
                                            && ui.small_button("->").clicked()
                                        {
                                            move_req = Some((card.id, 1));
                                        }
                                        if ui.small_button("Del").clicked() {
                                            del_req = Some(card.id);
                                        }
                                    });
                                });
                            }
                        });
                    }
                });
                if let Some((id, delta)) = move_req {
                    self.move_card(id, delta);
                }
                if let Some(id) = del_req {
                    self.delete_card(id);
                }
            });

        action
    }
}

fn open_kanban_db() -> (Option<Connection>, String) {
    let db_path: Option<PathBuf> = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(|root| PathBuf::from(root).join("makercraft").join("kanban.db"));

    if let Some(path) = db_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(conn) = Connection::open(&path) {
            let ok = conn.execute(
                "CREATE TABLE IF NOT EXISTS kanban_cards (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    station TEXT NOT NULL,
                    notes TEXT NOT NULL,
                    col INTEGER NOT NULL
                )",
                [],
            );
            if ok.is_ok() {
                return (Some(conn), format!("Kanban DB: {}", path.display()));
            }
        }
    }
    (None, "Kanban DB: in-memory session".to_string())
}

fn default_seed_cards() -> Vec<KanbanCard> {
    vec![
        KanbanCard {
            id: 1,
            title: "ESP32-S3 Enclosure Lid".to_string(),
            station: "3018 CNC".to_string(),
            notes: "3mm birch ply, 1.5mm DOC, verify G0 Z25".to_string(),
            col: KanbanCol::GcodeLint,
        },
        KanbanCard {
            id: 2,
            title: "Klipper Cable Chain".to_string(),
            station: "Ender-3".to_string(),
            notes: "PETG 235C/75C, 0.2mm layer".to_string(),
            col: KanbanCol::Fabricating,
        },
        KanbanCard {
            id: 3,
            title: "BME280 I2C Address Scan".to_string(),
            station: "Bus Pirate v4".to_string(),
            notes: "3.3V VPU enabled, expect 0x76/0x77".to_string(),
            col: KanbanCol::BenchDebug,
        },
        KanbanCard {
            id: 4,
            title: "Stepper STEP/DIR Timing Check".to_string(),
            station: "Oscilloscope".to_string(),
            notes: "CH1 STEP, CH2 DIR, rising edge trigger".to_string(),
            col: KanbanCol::IdeaCad,
        },
    ]
}
