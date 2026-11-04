use makercraft_core::MakerEngine;
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
}
