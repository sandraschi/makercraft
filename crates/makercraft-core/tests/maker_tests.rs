use makercraft_core::{
    capture_instrument, dispatch, pick_profile, Command, MachineProfile, MakerEngine,
    INSTRUMENT_SOURCES,
};
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

#[test]
fn test_dispatch_status_lists_all_commands() {
    let v = dispatch(Command::Status);
    assert_eq!(v.get("app").and_then(|x| x.as_str()), Some("makercraft"));
    let cmds = v
        .get("commands")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    assert_eq!(cmds.len(), 17);
}

#[test]
fn test_dispatch_lint_safe_and_unsafe() {
    let safe = dispatch(Command::GcodeLint {
        gcode: "G21\nG90\nG0 Z25\nG1 X10 Y10 F800\nM2".to_string(),
        profile: "cnc".to_string(),
    });
    assert_eq!(
        safe.get("passes_safety").and_then(|v| v.as_bool()),
        Some(true)
    );
    let bad = dispatch(Command::GcodeCheck {
        gcode: "G21\nG90\nG0 Z-5\nM2\n".to_string(),
        profile: "cnc".to_string(),
    });
    assert_eq!(
        bad.get("passes_safety").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn test_pick_profile_aliases() {
    assert_eq!(pick_profile("printer").name, "Ender-3 class (Klipper)");
    assert_eq!(pick_profile("ENDER-3").name, "Ender-3 class (Klipper)");
    assert_eq!(
        pick_profile("nope").name,
        "3018 class (GRBL 1.1h)".to_string()
    );
}

#[test]
fn test_dispatch_live_demos_without_hardware() {
    let printer = dispatch(Command::PrinterLive {
        url: "http://127.0.0.1:9".to_string(),
    });
    assert_eq!(printer.get("live").and_then(|v| v.as_bool()), Some(false));
    let cnc = dispatch(Command::CncProbe {
        port: "DEMO_DOES_NOT_EXIST_000".to_string(),
    });
    assert_eq!(cnc.get("live").and_then(|v| v.as_bool()), Some(false));
    let scope = dispatch(Command::ScopeIdn {
        addr: "127.0.0.1:9".to_string(),
    });
    assert_eq!(scope.get("live").and_then(|v| v.as_bool()), Some(false));
    let logic = dispatch(Command::LogicCapture {
        port: String::new(),
        hz: 1000,
        samples: 8,
    });
    assert_eq!(
        logic
            .get("channels")
            .and_then(|c| c.as_object())
            .map(|o| o.len()),
        Some(4)
    );
}

#[test]
fn test_dispatch_usb_demos_without_hardware() {
    let scope = dispatch(Command::ScopeUsb {
        vid: 0xFFFF,
        pid: 0xFFFF,
    });
    assert_eq!(scope.get("live").and_then(|v| v.as_bool()), Some(false));
    let logic = dispatch(Command::LogicUsb {
        driver: "saleae-logic8".to_string(),
        hz: 1000,
        samples: 8,
        cli: "DEMO_NO_SUCH_BIN_000".to_string(),
    });
    assert_eq!(logic.get("live").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(logic.get("engine").and_then(|v| v.as_str()), Some("down"));
}

#[test]
fn instrument_capture_writes_a_demo_file_and_refuses_an_unknown_source() {
    assert_eq!(INSTRUMENT_SOURCES.len(), 7);
    let dir = std::env::temp_dir().join(format!("makercraft-cap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("dir");
    for source in INSTRUMENT_SOURCES {
        let written = capture_instrument(source.id, &dir.display().to_string());
        assert_eq!(written["demo"], true, "{written}");
        assert_eq!(written["live"], false, "{written}");
        assert_eq!(written["former_repo"], source.former_repo);
        let path = written["path"].as_str().expect("path");
        let text = std::fs::read_to_string(path).expect("read");
        assert!(text.contains("\"demo\": true"), "{text}");
        assert!(text.contains(source.id), "{text}");
        if source.id == "function-generator" {
            assert!(text.contains("\"output_driven\": false"), "{text}");
        }
        if source.id == "jtag-swd" {
            assert!(text.contains("\"flashed\": false"), "{text}");
        }
    }
    let bad = capture_instrument("benchcraft", &dir.display().to_string());
    assert!(bad.get("error").is_some(), "{bad}");
}
