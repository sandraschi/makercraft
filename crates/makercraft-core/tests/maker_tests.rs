use makercraft_core::{MachineProfile, MakerEngine};
use makercraft_hal::MachineType;

#[test]
fn test_makercraft_hardware_status() {
    let printer = MakerEngine::get_printer_status();
    assert_eq!(printer.machine_type, MachineType::Klipper3DPrinter);
    assert_eq!(printer.state, "READY");

    let cnc = MakerEngine::get_cnc_status();
    assert_eq!(cnc.machine_type, MachineType::GrblCncRouter);
    assert!(cnc.extruder_or_spindle_temp >= 10000.0);

    let pins = MakerEngine::get_bus_pins();
    assert_eq!(pins.len(), 4);
    assert!(!MakerEngine::reference_rig().is_empty());
}

#[test]
fn test_makercraft_gcode_safety_lint() {
    let sample_gcode = "
        G90 ; absolute
        G21 ; metric
        G0 Z25.0 F3000 ; rapid safe height
        G1 X10.0 Y10.0 F1200
        G1 Z-1.5 F300 ; cutting plunge
        G1 X50.0 Y10.0
        M2 ; end
    ";
    let report = MakerEngine::lint_gcode(sample_gcode).expect("lint pass");
    assert!(report.passes_safety);
    assert_eq!(report.min_z_mm, -1.5);
    assert!(!report.warnings.is_empty());
    assert!(report.travel_distance_mm > 0.0);
    assert!(report.estimated_minutes >= 0.0);
}

#[test]
fn test_rapid_plunge_is_hard_error() {
    let bad = "G21\nG90\nG0 Z-5.0\nM2\n";
    let report = MakerEngine::lint_gcode(bad).expect("lint");
    assert!(!report.passes_safety);
    assert!(report.errors.iter().any(|e| e.contains("rapid")));
}

#[test]
fn test_envelope_violation_is_hard_error() {
    let bad = "G21\nG90\nG0 Z10\nG1 X500 Y10 F800\nM2\n";
    let report = MakerEngine::lint_gcode(bad).expect("lint");
    assert!(!report.passes_safety);
    assert!(report.errors.iter().any(|e| e.contains("outside")));
}

#[test]
fn test_empty_program_fails() {
    let report = MakerEngine::lint_gcode("  \n ; nothing\n").expect("lint");
    assert!(!report.passes_safety);
}

#[test]
fn test_printer_rejects_below_bed() {
    let g = "G21\nG90\nG1 X10 Y10 Z-1.0 F600\n";
    let report =
        MakerEngine::lint_gcode_with_profile(g, &MachineProfile::default_printer()).expect("lint");
    assert!(!report.passes_safety);
    assert!(report.errors.iter().any(|e| e.contains("below bed")));
}

#[test]
fn test_modal_coords_and_relative_mode() {
    // Bare coords repeat G1; G91 offsets from current pos.
    let g = "G21\nG90\nG1 X10 Y10 F1000\nX20 Y20\nG91\nG1 X5\n";
    let path = MakerEngine::preview_segments(g).expect("parse");
    assert_eq!(path.segments.len(), 3);
    assert!((path.segments[2].to_x_mm - 25.0).abs() < 1e-6);
}

#[test]
fn test_feedrate_and_units_warnings() {
    let g = "G90\nG1 X10 Y10 F99999\n";
    let report = MakerEngine::lint_gcode(g).expect("lint");
    assert!(report
        .warnings
        .iter()
        .any(|w| w.contains("assuming millimeters")));
    assert!(report.warnings.iter().any(|w| w.contains("feed")));
}

#[test]
fn test_overtemp_and_spindle_warnings() {
    let printer = MakerEngine::lint_gcode_with_profile(
        "G21\nG90\nM104 S350\n",
        &MachineProfile::default_printer(),
    )
    .expect("lint");
    assert!(!printer.passes_safety);

    let cnc = MakerEngine::lint_gcode("G21\nG90\nM3\nG1 X10 F500\nM5\n").expect("lint");
    assert!(cnc.passes_safety);
    assert!(cnc.warnings.iter().any(|w| w.contains("without S speed")));
}
