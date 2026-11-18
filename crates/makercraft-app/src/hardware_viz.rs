//! Pictorial vector representations of MakerCraft's fabrication machines
//! and bench instruments, rendered directly via `egui::Painter` so they are
//! crisp at any DPI and work headless in `--smoke` mode.

use egui::{Color32, Pos2, Rect, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareKind {
    Printer3d,
    CncRouter,
    Oscilloscope,
    LogicAnalyzer,
    BusPirate,
    Multimeter,
    FunctionGenerator,
    SpectrumAnalyzer,
    JtagSwd,
}

impl HardwareKind {
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Printer3d => "Ender-3 Klipper 3D Printer",
            Self::CncRouter => "3018 GRBL 1.1h CNC Router",
            Self::Oscilloscope => "Oscilloscope (SCPI / USBTMC)",
            Self::LogicAnalyzer => "Logic Analyzer (sigrok / SUMP)",
            Self::BusPirate => "Bus Pirate v4 (@3.3V)",
            Self::Multimeter => "Digital Multimeter (DMM)",
            Self::FunctionGenerator => "Function Generator (50 Ohm)",
            Self::SpectrumAnalyzer => "Spectrum Analyzer (RF)",
            Self::JtagSwd => "JTAG / SWD Debug Probe",
        }
    }

    #[must_use]
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Printer3d => "220x220x250 mm | Moonraker HTTP",
            Self::CncRouter => "300x180x45 mm | Serial GRBL",
            Self::Oscilloscope => "4-Ch | 100 MHz | Edge Trigger",
            Self::LogicAnalyzer => "8-Ch Digital | SPI / I2C / UART",
            Self::BusPirate => "10-Pin IDC | VPU 3.3V | I2C/SPI",
            Self::Multimeter => "CAT III | V / A / Ohm / Cont",
            Self::FunctionGenerator => "Sine / Square / Pulse | BNC",
            Self::SpectrumAnalyzer => "100 kHz - 3 GHz | dBm FFT",
            Self::JtagSwd => "ARM Cortex 10-Pin | VTref / SWD",
        }
    }
}

pub fn draw_hardware_diagram(
    ui: &mut egui::Ui,
    kind: HardwareKind,
    size: Vec2,
    estop: bool,
    accent: Color32,
) {
    let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter_at(rect);
    let bg = Color32::from_rgb(12, 18, 30);
    let frame = Color32::from_rgb(78, 108, 150);
    let metal = Color32::from_rgb(165, 182, 205);
    let teal = Color32::from_rgb(110, 240, 205);
    let gold = Color32::from_rgb(255, 205, 105);
    let rose = Color32::from_rgb(255, 115, 115);

    p.rect_filled(rect, 6.0, bg);
    p.rect_stroke(
        rect,
        6.0,
        Stroke::new(1.5, if estop { rose } else { frame }),
        egui::StrokeKind::Inside,
    );

    let r = rect.shrink(10.0);
    match kind {
        HardwareKind::Printer3d => draw_printer(&p, r, metal, teal, gold, rose, estop),
        HardwareKind::CncRouter => draw_cnc(&p, r, metal, accent, gold, rose, estop),
        HardwareKind::Oscilloscope => draw_scope(&p, r, metal, teal, gold),
        HardwareKind::LogicAnalyzer => draw_logic(&p, r, metal, teal, gold, rose),
        HardwareKind::BusPirate => draw_bus_pirate(&p, r, metal, teal, gold, rose),
        HardwareKind::Multimeter => draw_dmm(&p, r, metal, gold, rose, teal),
        HardwareKind::FunctionGenerator => draw_fgen(&p, r, metal, teal, gold),
        HardwareKind::SpectrumAnalyzer => draw_spectrum(&p, r, metal, gold, teal),
        HardwareKind::JtagSwd => draw_jtag(&p, r, metal, teal, gold, rose),
    }
}

fn draw_printer(
    p: &egui::Painter,
    r: Rect,
    metal: Color32,
    teal: Color32,
    gold: Color32,
    rose: Color32,
    estop: bool,
) {
    let base_y = r.bottom() - 16.0;
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(r.left() + 12.0, base_y),
            Pos2::new(r.right() - 12.0, r.bottom()),
        ),
        2.0,
        Color32::from_rgb(40, 50, 68),
    );
    // Gantry uprights + top beam
    let lx = r.left() + 22.0;
    let rx = r.right() - 22.0;
    let ty = r.top() + 10.0;
    p.line_segment(
        [Pos2::new(lx, base_y), Pos2::new(lx, ty)],
        Stroke::new(4.0, metal),
    );
    p.line_segment(
        [Pos2::new(rx, base_y), Pos2::new(rx, ty)],
        Stroke::new(4.0, metal),
    );
    p.line_segment(
        [Pos2::new(lx - 4.0, ty), Pos2::new(rx + 4.0, ty)],
        Stroke::new(4.0, metal),
    );
    // Heated bed
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(lx + 12.0, base_y - 10.0),
            Pos2::new(rx - 12.0, base_y - 3.0),
        ),
        1.0,
        if estop { rose } else { gold },
    );
    // X-gantry + hotend
    let xy = r.center().y - 6.0;
    p.line_segment(
        [Pos2::new(lx, xy), Pos2::new(rx, xy)],
        Stroke::new(3.0, metal),
    );
    let hx = r.center().x + 10.0;
    p.rect_filled(
        Rect::from_center_size(Pos2::new(hx, xy + 8.0), Vec2::new(22.0, 22.0)),
        2.0,
        Color32::from_rgb(55, 75, 110),
    );
    // Nozzle triangle
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(hx - 6.0, xy + 19.0),
            Pos2::new(hx + 6.0, xy + 19.0),
            Pos2::new(hx, xy + 29.0),
        ],
        if estop { rose } else { teal },
        Stroke::NONE,
    ));
    p.text(
        Pos2::new(r.left() + 4.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "220x220x250 | NOZZLE 210C / BED 60C",
        egui::FontId::monospace(10.0),
        teal,
    );
}

fn draw_cnc(
    p: &egui::Painter,
    r: Rect,
    metal: Color32,
    accent: Color32,
    gold: Color32,
    rose: Color32,
    estop: bool,
) {
    let bed_y = r.bottom() - 18.0;
    // T-slot aluminum bed
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(r.left() + 10.0, bed_y),
            Pos2::new(r.right() - 10.0, bed_y + 10.0),
        ),
        2.0,
        Color32::from_rgb(95, 110, 130),
    );
    for i in 1..6 {
        let sx = r.left() + 10.0 + (r.width() - 20.0) * (i as f32 / 6.0);
        p.line_segment(
            [Pos2::new(sx, bed_y + 1.0), Pos2::new(sx, bed_y + 9.0)],
            Stroke::new(1.5, Color32::from_rgb(25, 32, 44)),
        );
    }
    // Bridge gantry
    let gx1 = r.left() + 18.0;
    let gx2 = r.right() - 18.0;
    let gy = r.top() + 26.0;
    p.line_segment(
        [Pos2::new(gx1, bed_y), Pos2::new(gx1, gy)],
        Stroke::new(4.0, metal),
    );
    p.line_segment(
        [Pos2::new(gx2, bed_y), Pos2::new(gx2, gy)],
        Stroke::new(4.0, metal),
    );
    p.rect_filled(
        Rect::from_min_max(Pos2::new(gx1 - 2.0, gy), Pos2::new(gx2 + 2.0, gy + 12.0)),
        2.0,
        Color32::from_rgb(65, 80, 105),
    );
    // Spindle motor + ER11 collet + endmill
    let sx = r.center().x;
    p.rect_filled(
        Rect::from_center_size(Pos2::new(sx, gy + 24.0), Vec2::new(24.0, 34.0)),
        3.0,
        if estop { rose } else { accent },
    );
    p.rect_filled(
        Rect::from_center_size(Pos2::new(sx, gy + 44.0), Vec2::new(12.0, 8.0)),
        1.0,
        gold,
    );
    p.line_segment(
        [Pos2::new(sx, gy + 48.0), Pos2::new(sx, bed_y - 4.0)],
        Stroke::new(2.5, Color32::WHITE),
    );
    p.text(
        Pos2::new(r.left() + 4.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "300x180x45 | GRBL 1.1h | ER11",
        egui::FontId::monospace(10.0),
        gold,
    );
}

fn draw_scope(p: &egui::Painter, r: Rect, metal: Color32, teal: Color32, gold: Color32) {
    let scr = Rect::from_min_max(
        Pos2::new(r.left() + 6.0, r.top() + 18.0),
        Pos2::new(r.right() - 62.0, r.bottom() - 22.0),
    );
    p.rect_filled(scr, 3.0, Color32::from_rgb(6, 14, 24));
    p.rect_stroke(scr, 3.0, Stroke::new(1.5, metal), egui::StrokeKind::Inside);
    // Graticule
    for i in 1..4 {
        let y = scr.top() + scr.height() * (i as f32 / 4.0);
        p.line_segment(
            [Pos2::new(scr.left(), y), Pos2::new(scr.right(), y)],
            Stroke::new(0.7, Color32::from_rgb(30, 52, 74)),
        );
    }
    for i in 1..6 {
        let x = scr.left() + scr.width() * (i as f32 / 6.0);
        p.line_segment(
            [Pos2::new(x, scr.top()), Pos2::new(x, scr.bottom())],
            Stroke::new(0.7, Color32::from_rgb(30, 52, 74)),
        );
    }
    // CH1 sine wave
    let mut pts = Vec::with_capacity(48);
    for i in 0..48 {
        let t = i as f32 / 47.0;
        let x = scr.left() + t * scr.width();
        let y = scr.center().y - (t * std::f32::consts::TAU * 2.5).sin() * (scr.height() * 0.34);
        pts.push(Pos2::new(x, y));
    }
    p.add(egui::Shape::line(pts, Stroke::new(2.0, gold)));
    // Knobs + BNC jacks on right
    let kx = r.right() - 30.0;
    for (idx, col) in [(0, gold), (1, teal), (2, metal)] {
        let ky = r.top() + 30.0 + idx as f32 * 28.0;
        p.circle_filled(Pos2::new(kx, ky), 9.0, Color32::from_rgb(45, 58, 80));
        p.circle_stroke(Pos2::new(kx, ky), 9.0, Stroke::new(1.5, col));
    }
    p.text(
        Pos2::new(r.left() + 6.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "CH1 1.00V/div  500us/div  TRIG ^",
        egui::FontId::monospace(10.0),
        teal,
    );
    // BNC connectors at bottom right
    for i in 0..2 {
        let bx = scr.left() + 22.0 + i as f32 * 40.0;
        let by = r.bottom() - 10.0;
        p.circle_stroke(Pos2::new(bx, by), 6.0, Stroke::new(2.0, metal));
        p.circle_filled(Pos2::new(bx, by), 2.0, gold);
    }
}

fn draw_logic(
    p: &egui::Painter,
    r: Rect,
    metal: Color32,
    teal: Color32,
    gold: Color32,
    rose: Color32,
) {
    let colors = [gold, teal, rose, Color32::from_rgb(130, 195, 255)];
    let top = r.top() + 20.0;
    let row_h = (r.height() - 26.0) / 4.0;
    for (ch, col) in colors.iter().enumerate() {
        let y_hi = top + ch as f32 * row_h + 4.0;
        let y_lo = y_hi + row_h - 10.0;
        p.text(
            Pos2::new(r.left() + 4.0, (y_hi + y_lo) * 0.5),
            egui::Align2::LEFT_CENTER,
            format!("D{ch}"),
            egui::FontId::monospace(10.0),
            *col,
        );
        let x0 = r.left() + 26.0;
        let w = (r.width() - 34.0) / 8.0;
        let mut pts: Vec<Pos2> = Vec::new();
        for step in 0..8 {
            let high = ((step + ch) % (ch + 1).max(1)) == 0 || (step % 2 == 0 && ch == 0);
            let y = if high { y_hi } else { y_lo };
            let xs = x0 + step as f32 * w;
            let xe = xs + w;
            if let Some(prev) = pts.last().copied() {
                if (prev.y - y).abs() > 0.5 {
                    pts.push(Pos2::new(xs, y));
                }
            } else {
                pts.push(Pos2::new(xs, y));
            }
            pts.push(Pos2::new(xe, y));
        }
        p.add(egui::Shape::line(pts, Stroke::new(1.8, *col)));
    }
    p.text(
        Pos2::new(r.left() + 4.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "SIGROK / SUMP | 24 MHz | SPI/I2C/UART",
        egui::FontId::monospace(10.0),
        metal,
    );
}

fn draw_bus_pirate(
    p: &egui::Painter,
    r: Rect,
    metal: Color32,
    teal: Color32,
    gold: Color32,
    rose: Color32,
) {
    let pcb = r.shrink2(Vec2::new(10.0, 16.0));
    p.rect_filled(pcb, 4.0, Color32::from_rgb(16, 62, 46));
    p.rect_stroke(pcb, 4.0, Stroke::new(1.5, teal), egui::StrokeKind::Inside);
    // 10-pin IDC header on right
    let hx = pcb.right() - 26.0;
    let hy = pcb.top() + 10.0;
    let pin_h = (pcb.height() - 20.0) / 5.0;
    for row in 0..5 {
        let py = hy + row as f32 * pin_h + pin_h * 0.5;
        p.circle_filled(Pos2::new(hx, py), 3.0, gold);
        p.circle_filled(Pos2::new(hx + 12.0, py), 3.0, gold);
    }
    // USB connector on left
    p.rect_filled(
        Rect::from_center_size(Pos2::new(pcb.left(), pcb.center().y), Vec2::new(12.0, 22.0)),
        2.0,
        metal,
    );
    // Status LEDs
    p.circle_filled(Pos2::new(pcb.left() + 26.0, pcb.top() + 16.0), 3.5, teal);
    p.circle_filled(Pos2::new(pcb.left() + 38.0, pcb.top() + 16.0), 3.5, gold);
    p.circle_filled(Pos2::new(pcb.left() + 50.0, pcb.top() + 16.0), 3.5, rose);
    p.text(
        Pos2::new(pcb.left() + 20.0, pcb.center().y - 6.0),
        egui::Align2::LEFT_CENTER,
        "BUS PIRATE v4\nVPU 3.3V | 10-PIN IDC",
        egui::FontId::monospace(10.0),
        Color32::WHITE,
    );
}

fn draw_dmm(
    p: &egui::Painter,
    r: Rect,
    metal: Color32,
    gold: Color32,
    rose: Color32,
    teal: Color32,
) {
    let body = Rect::from_center_size(r.center(), Vec2::new(r.width() * 0.56, r.height() - 8.0));
    p.rect_filled(body, 6.0, Color32::from_rgb(185, 130, 22));
    p.rect_stroke(body, 6.0, Stroke::new(1.5, gold), egui::StrokeKind::Inside);
    let lcd = Rect::from_min_max(
        Pos2::new(body.left() + 10.0, body.top() + 10.0),
        Pos2::new(body.right() - 10.0, body.top() + 42.0),
    );
    p.rect_filled(lcd, 2.0, Color32::from_rgb(165, 195, 165));
    p.text(
        lcd.center(),
        egui::Align2::CENTER_CENTER,
        "3.302 V",
        egui::FontId::monospace(14.0),
        Color32::BLACK,
    );
    let dial_c = Pos2::new(body.center().x, body.center().y + 12.0);
    p.circle_filled(dial_c, 20.0, Color32::from_rgb(35, 42, 54));
    p.line_segment(
        [dial_c, Pos2::new(dial_c.x - 12.0, dial_c.y - 12.0)],
        Stroke::new(3.0, teal),
    );
    // 3 banana jacks
    for (idx, col) in [(0, rose), (1, Color32::BLACK), (2, metal)] {
        let jx = body.left() + 24.0 + idx as f32 * (body.width() - 48.0) * 0.5;
        p.circle_filled(Pos2::new(jx, body.bottom() - 12.0), 5.0, col);
    }
}

fn draw_fgen(p: &egui::Painter, r: Rect, metal: Color32, teal: Color32, gold: Color32) {
    let lcd = Rect::from_min_max(
        Pos2::new(r.left() + 8.0, r.top() + 20.0),
        Pos2::new(r.right() - 56.0, r.bottom() - 20.0),
    );
    p.rect_filled(lcd, 3.0, Color32::from_rgb(10, 28, 42));
    p.rect_stroke(lcd, 3.0, Stroke::new(1.2, teal), egui::StrokeKind::Inside);
    p.text(
        lcd.center(),
        egui::Align2::CENTER_CENTER,
        "SINE  1.000 kHz\n3.30 Vpp  HiZ",
        egui::FontId::monospace(12.0),
        gold,
    );
    p.circle_stroke(
        Pos2::new(r.right() - 26.0, r.center().y),
        10.0,
        Stroke::new(2.0, metal),
    );
    p.text(
        Pos2::new(r.left() + 8.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "FUNCTION GEN | NO OUTPUT DRIVEN",
        egui::FontId::monospace(10.0),
        metal,
    );
}

fn draw_spectrum(p: &egui::Painter, r: Rect, metal: Color32, gold: Color32, teal: Color32) {
    let scr = Rect::from_min_max(
        Pos2::new(r.left() + 8.0, r.top() + 20.0),
        Pos2::new(r.right() - 8.0, r.bottom() - 12.0),
    );
    p.rect_filled(scr, 3.0, Color32::from_rgb(8, 14, 24));
    p.rect_stroke(scr, 3.0, Stroke::new(1.2, metal), egui::StrokeKind::Inside);
    let mut pts = Vec::with_capacity(40);
    for i in 0..40 {
        let t = i as f32 / 39.0;
        let x = scr.left() + t * scr.width();
        let peak1 = (-((t - 0.35) * 14.0).powi(2)).exp() * 0.72;
        let peak2 = (-((t - 0.70) * 18.0).powi(2)).exp() * 0.35;
        let noise = ((i * 7) % 5) as f32 * 0.015;
        let y = scr.bottom() - 10.0 - (peak1 + peak2 + noise) * (scr.height() - 20.0);
        pts.push(Pos2::new(x, y));
    }
    p.add(egui::Shape::line(pts, Stroke::new(2.0, gold)));
    p.text(
        Pos2::new(r.left() + 8.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "SPECTRUM | -20 dBm PEAK @ 2.40 GHz",
        egui::FontId::monospace(10.0),
        teal,
    );
}

fn draw_jtag(
    p: &egui::Painter,
    r: Rect,
    metal: Color32,
    teal: Color32,
    gold: Color32,
    rose: Color32,
) {
    let dongle = Rect::from_center_size(r.center(), Vec2::new(r.width() * 0.68, r.height() * 0.56));
    p.rect_filled(dongle, 4.0, Color32::from_rgb(35, 48, 72));
    p.rect_stroke(
        dongle,
        4.0,
        Stroke::new(1.5, metal),
        egui::StrokeKind::Inside,
    );
    p.text(
        Pos2::new(dongle.left() + 10.0, dongle.top() + 10.0),
        egui::Align2::LEFT_TOP,
        "SWD / JTAG\nVTref  SWDIO\nSWCLK  nRST",
        egui::FontId::monospace(10.0),
        teal,
    );
    // 10-pin Cortex header
    for col_i in 0..5 {
        let px = dongle.right() - 50.0 + col_i as f32 * 9.0;
        p.circle_filled(Pos2::new(px, dongle.bottom() - 22.0), 2.5, gold);
        p.circle_filled(Pos2::new(px, dongle.bottom() - 13.0), 2.5, rose);
    }
    p.text(
        Pos2::new(r.left() + 8.0, r.top() + 2.0),
        egui::Align2::LEFT_TOP,
        "INSPECT ONLY | NO FLASH / NO HALT",
        egui::FontId::monospace(10.0),
        rose,
    );
}

pub enum WorkshopAction {
    None,
    SelectHardware(HardwareKind),
    ToggleEstop,
}

/// Interactive full-scene pictorial depiction of a maker's workshop:
/// Left Bay = Digital Fabrication Bench (Ender-3 Printer + 3018 CNC + HEPA Vac + E-Stop)
/// Right Bay = ESD Electronics Workbench (Shelf with Scope/FGen/Spectrum + ESD Mat with Logic/BusPirate/DMM/JTAG/Soldering/DUT)
pub fn draw_workshop_panorama(
    ui: &mut egui::Ui,
    width: f32,
    estop: bool,
    selected: Option<HardwareKind>,
) -> WorkshopAction {
    let w = width.max(680.0);
    let h = 420.0;
    let (canvas, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let p = ui.painter_at(canvas);

    let wall_bg = Color32::from_rgb(13, 18, 28);
    let pegboard_bg = Color32::from_rgb(24, 32, 48);
    let bench_wood = Color32::from_rgb(112, 78, 46);
    let bench_edge = Color32::from_rgb(82, 54, 30);
    let steel_leg = Color32::from_rgb(48, 58, 74);
    let esd_mat = Color32::from_rgb(16, 68, 96);
    let metal = Color32::from_rgb(170, 186, 208);
    let teal = Color32::from_rgb(110, 240, 205);
    let gold = Color32::from_rgb(255, 205, 105);
    let rose = Color32::from_rgb(255, 115, 115);

    // 1. Workshop wall & floor
    p.rect_filled(canvas, 8.0, wall_bg);
    p.rect_stroke(
        canvas,
        8.0,
        Stroke::new(1.5, Color32::from_rgb(60, 82, 115)),
        egui::StrokeKind::Inside,
    );
    let floor_y = canvas.bottom() - 28.0;
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(canvas.left() + 2.0, floor_y),
            Pos2::new(canvas.right() - 2.0, canvas.bottom() - 2.0),
        ),
        4.0,
        Color32::from_rgb(18, 24, 34),
    );

    // 2. Overhead LED shop light + bay headers
    let mid_x = canvas.left() + w * 0.47;
    p.line_segment(
        [
            Pos2::new(mid_x, canvas.top() + 14.0),
            Pos2::new(mid_x, floor_y),
        ],
        Stroke::new(1.5, Color32::from_rgb(42, 56, 78)),
    );
    p.text(
        Pos2::new(canvas.left() + 18.0, canvas.top() + 12.0),
        egui::Align2::LEFT_TOP,
        "BAY 1: DIGITAL FABRICATION BENCH (KLIPPER + GRBL + DUST EXTRACTION)",
        egui::FontId::monospace(11.0),
        gold,
    );
    p.text(
        Pos2::new(mid_x + 16.0, canvas.top() + 12.0),
        egui::Align2::LEFT_TOP,
        "BAY 2: ESD ELECTRONICS WORKBENCH (SCPI + SIGROK + BUS PIRATE + SOLDERING)",
        egui::FontId::monospace(11.0),
        teal,
    );

    // 3. Pegboard on Right Bay wall with hand tools (calipers, cutters, hex keys, safety glasses)
    let peg = Rect::from_min_max(
        Pos2::new(mid_x + 18.0, canvas.top() + 34.0),
        Pos2::new(canvas.right() - 18.0, canvas.top() + 104.0),
    );
    p.rect_filled(peg, 4.0, pegboard_bg);
    p.rect_stroke(
        peg,
        4.0,
        Stroke::new(1.0, steel_leg),
        egui::StrokeKind::Inside,
    );
    for gx in (0..18).map(|i| peg.left() + 16.0 + i as f32 * ((peg.width() - 32.0) / 17.0)) {
        for gy in [peg.top() + 14.0, peg.top() + 34.0, peg.top() + 54.0] {
            p.circle_filled(Pos2::new(gx, gy), 1.3, Color32::from_rgb(70, 88, 114));
        }
    }
    // Safety glasses icon
    let g_y = peg.top() + 34.0;
    p.rect_stroke(
        Rect::from_center_size(Pos2::new(peg.left() + 44.0, g_y), Vec2::new(18.0, 11.0)),
        3.0,
        Stroke::new(1.8, teal),
        egui::StrokeKind::Inside,
    );
    p.rect_stroke(
        Rect::from_center_size(Pos2::new(peg.left() + 68.0, g_y), Vec2::new(18.0, 11.0)),
        3.0,
        Stroke::new(1.8, teal),
        egui::StrokeKind::Inside,
    );
    p.line_segment(
        [
            Pos2::new(peg.left() + 53.0, g_y - 2.0),
            Pos2::new(peg.left() + 59.0, g_y - 2.0),
        ],
        Stroke::new(1.8, teal),
    );
    // Digital calipers icon
    let cal_x = peg.left() + 115.0;
    p.line_segment(
        [
            Pos2::new(cal_x, peg.top() + 14.0),
            Pos2::new(cal_x, peg.bottom() - 10.0),
        ],
        Stroke::new(2.5, metal),
    );
    p.line_segment(
        [
            Pos2::new(cal_x, peg.top() + 14.0),
            Pos2::new(cal_x + 14.0, peg.top() + 18.0),
        ],
        Stroke::new(2.0, metal),
    );
    p.line_segment(
        [
            Pos2::new(cal_x, peg.top() + 28.0),
            Pos2::new(cal_x + 14.0, peg.top() + 24.0),
        ],
        Stroke::new(2.0, metal),
    );
    // Hex key rack
    for i in 0..5 {
        let hx = peg.left() + 165.0 + i as f32 * 9.0;
        let len = 34.0 - i as f32 * 4.0;
        p.line_segment(
            [
                Pos2::new(hx, peg.top() + 16.0),
                Pos2::new(hx, peg.top() + 16.0 + len),
            ],
            Stroke::new(2.0, gold),
        );
        p.line_segment(
            [
                Pos2::new(hx, peg.top() + 16.0),
                Pos2::new(hx + 6.0, peg.top() + 16.0),
            ],
            Stroke::new(2.0, gold),
        );
    }
    p.text(
        Pos2::new(peg.right() - 12.0, peg.bottom() - 8.0),
        egui::Align2::RIGHT_BOTTOM,
        "PEGBOARD: PPE | CALIPERS | HEX | FLUSH CUTTERS",
        egui::FontId::monospace(9.5),
        metal,
    );

    // 4. Two physical workbenches (tabletop + steel legs)
    let bench_y = canvas.top() + 290.0;
    let bench1 = Rect::from_min_max(
        Pos2::new(canvas.left() + 16.0, bench_y),
        Pos2::new(mid_x - 14.0, bench_y + 18.0),
    );
    let bench2 = Rect::from_min_max(
        Pos2::new(mid_x + 16.0, bench_y),
        Pos2::new(canvas.right() - 16.0, bench_y + 18.0),
    );
    for b in [bench1, bench2] {
        p.rect_filled(b, 3.0, bench_wood);
        p.rect_filled(
            Rect::from_min_max(
                Pos2::new(b.left(), b.bottom() - 5.0),
                Pos2::new(b.right(), b.bottom()),
            ),
            1.0,
            bench_edge,
        );
        p.rect_filled(
            Rect::from_min_max(
                Pos2::new(b.left() + 10.0, b.bottom()),
                Pos2::new(b.left() + 20.0, floor_y),
            ),
            0.0,
            steel_leg,
        );
        p.rect_filled(
            Rect::from_min_max(
                Pos2::new(b.right() - 20.0, b.bottom()),
                Pos2::new(b.right() - 10.0, floor_y),
            ),
            0.0,
            steel_leg,
        );
    }

    // HEPA Shop Vac under CNC bench + dust hose
    let vac = Rect::from_min_max(
        Pos2::new(bench1.right() - 95.0, bench1.bottom() + 24.0),
        Pos2::new(bench1.right() - 34.0, floor_y),
    );
    p.rect_filled(vac, 5.0, Color32::from_rgb(52, 62, 82));
    p.rect_stroke(vac, 5.0, Stroke::new(1.5, metal), egui::StrokeKind::Inside);
    p.text(
        vac.center(),
        egui::Align2::CENTER_CENTER,
        "HEPA\nVAC",
        egui::FontId::monospace(10.0),
        gold,
    );

    // 5. Left Bay stations: Ender-3 Printer + Filament Spool, 3018 CNC in Enclosure + Hose, E-Stop button
    let printer_rect = Rect::from_min_max(
        Pos2::new(bench1.left() + 10.0, bench_y - 190.0),
        Pos2::new(bench1.left() + (bench1.width() * 0.46), bench_y - 2.0),
    );
    // Filament spool above printer
    let spool_c = Pos2::new(printer_rect.center().x, printer_rect.top() - 22.0);
    p.circle_stroke(spool_c, 16.0, Stroke::new(4.0, teal));
    p.circle_filled(spool_c, 5.0, steel_leg);
    p.line_segment(
        [
            spool_c,
            Pos2::new(printer_rect.center().x + 8.0, printer_rect.top() + 28.0),
        ],
        Stroke::new(1.5, teal),
    );
    draw_station_frame(
        &p,
        printer_rect,
        "ENDER-3 KLIPPER",
        selected == Some(HardwareKind::Printer3d),
        teal,
    );
    draw_printer(&p, printer_rect.shrink(8.0), metal, teal, gold, rose, estop);

    let cnc_rect = Rect::from_min_max(
        Pos2::new(printer_rect.right() + 12.0, bench_y - 190.0),
        Pos2::new(bench1.right() - 10.0, bench_y - 2.0),
    );
    // Dust hose from CNC to HEPA Vac
    p.line_segment(
        [
            Pos2::new(cnc_rect.center().x + 20.0, cnc_rect.bottom() - 24.0),
            Pos2::new(vac.center().x, vac.top()),
        ],
        Stroke::new(4.0, Color32::from_rgb(80, 95, 118)),
    );
    draw_station_frame(
        &p,
        cnc_rect,
        "3018 GRBL CNC",
        selected == Some(HardwareKind::CncRouter),
        gold,
    );
    draw_cnc(&p, cnc_rect.shrink(8.0), metal, teal, gold, rose, estop);

    // Red Mushroom E-Stop Button on front rail of Fabrication Bench
    let estop_rect = Rect::from_center_size(
        Pos2::new(bench1.center().x, bench1.bottom() + 28.0),
        Vec2::new(96.0, 38.0),
    );
    p.rect_filled(estop_rect, 5.0, Color32::from_rgb(190, 150, 25));
    p.circle_filled(
        Pos2::new(estop_rect.left() + 22.0, estop_rect.center().y),
        12.0,
        if estop {
            rose
        } else {
            Color32::from_rgb(195, 36, 36)
        },
    );
    p.text(
        Pos2::new(estop_rect.left() + 40.0, estop_rect.center().y),
        egui::Align2::LEFT_CENTER,
        if estop {
            "E-STOP\nLATCHED"
        } else {
            "E-STOP\nREADY"
        },
        egui::FontId::monospace(10.0),
        Color32::BLACK,
    );

    // 6. Right Bay: Instrument Shelf (Scope, FGen, Spectrum) + ESD Mat (Logic, Bus Pirate, DMM, JTAG, Soldering + DUT)
    let shelf_y = canvas.top() + 195.0;
    p.rect_filled(
        Rect::from_min_max(
            Pos2::new(bench2.left(), shelf_y),
            Pos2::new(bench2.right(), shelf_y + 8.0),
        ),
        2.0,
        steel_leg,
    );

    let shelf_w = (bench2.width() - 24.0) / 3.0;
    let scope_rect = Rect::from_min_max(
        Pos2::new(bench2.left(), shelf_y - 84.0),
        Pos2::new(bench2.left() + shelf_w, shelf_y - 2.0),
    );
    let fgen_rect = Rect::from_min_max(
        Pos2::new(scope_rect.right() + 12.0, shelf_y - 84.0),
        Pos2::new(scope_rect.right() + 12.0 + shelf_w, shelf_y - 2.0),
    );
    let spec_rect = Rect::from_min_max(
        Pos2::new(fgen_rect.right() + 12.0, shelf_y - 84.0),
        Pos2::new(bench2.right(), shelf_y - 2.0),
    );

    draw_station_frame(
        &p,
        scope_rect,
        "OSCILLOSCOPE",
        selected == Some(HardwareKind::Oscilloscope),
        gold,
    );
    draw_scope(&p, scope_rect.shrink(6.0), metal, teal, gold);

    draw_station_frame(
        &p,
        fgen_rect,
        "FUNC GEN",
        selected == Some(HardwareKind::FunctionGenerator),
        teal,
    );
    draw_fgen(&p, fgen_rect.shrink(6.0), metal, teal, gold);

    draw_station_frame(
        &p,
        spec_rect,
        "SPECTRUM",
        selected == Some(HardwareKind::SpectrumAnalyzer),
        gold,
    );
    draw_spectrum(&p, spec_rect.shrink(6.0), metal, gold, teal);

    // ESD Mat on Desk Surface
    let mat_rect = Rect::from_min_max(
        Pos2::new(bench2.left() + 8.0, bench_y - 12.0),
        Pos2::new(bench2.right() - 8.0, bench_y + 4.0),
    );
    p.rect_filled(mat_rect, 2.0, esd_mat);
    p.text(
        Pos2::new(mat_rect.right() - 8.0, mat_rect.center().y),
        egui::Align2::RIGHT_CENTER,
        "ESD MAT GND",
        egui::FontId::monospace(9.0),
        teal,
    );

    // 4 desk instruments on the ESD bench
    let desk_w = (bench2.width() - 36.0) / 4.0;
    let desk_top = shelf_y + 14.0;
    let desk_bot = bench_y - 14.0;
    let logic_rect = Rect::from_min_max(
        Pos2::new(bench2.left(), desk_top),
        Pos2::new(bench2.left() + desk_w, desk_bot),
    );
    let bus_rect = Rect::from_min_max(
        Pos2::new(logic_rect.right() + 12.0, desk_top),
        Pos2::new(logic_rect.right() + 12.0 + desk_w, desk_bot),
    );
    let dmm_rect = Rect::from_min_max(
        Pos2::new(bus_rect.right() + 12.0, desk_top),
        Pos2::new(bus_rect.right() + 12.0 + desk_w, desk_bot),
    );
    let jtag_rect = Rect::from_min_max(
        Pos2::new(dmm_rect.right() + 12.0, desk_top),
        Pos2::new(bench2.right(), desk_bot),
    );

    draw_station_frame(
        &p,
        logic_rect,
        "LOGIC ANALYZER",
        selected == Some(HardwareKind::LogicAnalyzer),
        teal,
    );
    draw_logic(&p, logic_rect.shrink(6.0), metal, teal, gold, rose);

    draw_station_frame(
        &p,
        bus_rect,
        "BUS PIRATE v4",
        selected == Some(HardwareKind::BusPirate),
        teal,
    );
    draw_bus_pirate(&p, bus_rect.shrink(6.0), metal, teal, gold, rose);

    draw_station_frame(
        &p,
        dmm_rect,
        "MULTIMETER",
        selected == Some(HardwareKind::Multimeter),
        gold,
    );
    draw_dmm(&p, dmm_rect.shrink(6.0), metal, gold, rose, teal);

    draw_station_frame(
        &p,
        jtag_rect,
        "JTAG / SWD",
        selected == Some(HardwareKind::JtagSwd),
        rose,
    );
    draw_jtag(&p, jtag_rect.shrink(6.0), metal, teal, gold, rose);

    // Interactive hover highlight + click dispatch
    let hotspots: &[(Rect, HardwareKind)] = &[
        (printer_rect, HardwareKind::Printer3d),
        (cnc_rect, HardwareKind::CncRouter),
        (scope_rect, HardwareKind::Oscilloscope),
        (fgen_rect, HardwareKind::FunctionGenerator),
        (spec_rect, HardwareKind::SpectrumAnalyzer),
        (logic_rect, HardwareKind::LogicAnalyzer),
        (bus_rect, HardwareKind::BusPirate),
        (dmm_rect, HardwareKind::Multimeter),
        (jtag_rect, HardwareKind::JtagSwd),
    ];

    if let Some(hover_pos) = resp.hover_pos() {
        for (rect, kind) in hotspots {
            if rect.contains(hover_pos) {
                p.rect_stroke(
                    *rect,
                    6.0,
                    Stroke::new(2.5, Color32::WHITE),
                    egui::StrokeKind::Outside,
                );
                p.text(
                    Pos2::new(canvas.center().x, canvas.bottom() - 14.0),
                    egui::Align2::CENTER_CENTER,
                    format!(
                        "Click to inspect/probe: {} ({})",
                        kind.title(),
                        kind.subtitle()
                    ),
                    egui::FontId::monospace(11.0),
                    Color32::WHITE,
                );
            }
        }
        if estop_rect.contains(hover_pos) {
            p.rect_stroke(
                estop_rect,
                5.0,
                Stroke::new(2.5, rose),
                egui::StrokeKind::Outside,
            );
        }
    }

    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if estop_rect.contains(pos) {
                return WorkshopAction::ToggleEstop;
            }
            for (rect, kind) in hotspots {
                if rect.contains(pos) {
                    return WorkshopAction::SelectHardware(*kind);
                }
            }
        }
    }

    WorkshopAction::None
}

fn draw_station_frame(
    p: &egui::Painter,
    rect: Rect,
    label: &str,
    is_selected: bool,
    accent: Color32,
) {
    p.rect_filled(rect, 5.0, Color32::from_rgb(12, 18, 30));
    p.rect_stroke(
        rect,
        5.0,
        Stroke::new(
            if is_selected { 2.5 } else { 1.2 },
            if is_selected {
                accent
            } else {
                Color32::from_rgb(70, 96, 134)
            },
        ),
        egui::StrokeKind::Inside,
    );
    p.text(
        Pos2::new(rect.center().x, rect.bottom() - 2.0),
        egui::Align2::CENTER_BOTTOM,
        label,
        egui::FontId::monospace(9.5),
        if is_selected {
            accent
        } else {
            Color32::from_rgb(185, 202, 224)
        },
    );
}
