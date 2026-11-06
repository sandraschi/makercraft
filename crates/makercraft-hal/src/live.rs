//! MakerCraft live hardware bridges (v2, read-only).
//!
//! - Moonraker HTTP (Klipper) with a 3 s timeout, `Demo` fallback.
//! - GRBL `?` status over serial (115200 8N1), `Demo` fallback. Never sends motion.
//! - Bus Pirate identity over serial (text mode `i`), `Demo` fallback. Read-only.
//! - Generic SCPI instrument (`*IDN?`) over TCP for oscilloscopes / PSUs / DMMs.
//! - 4-pin logic sampler: deterministic `Demo` pattern until SUMP lands (v3).
//!
//! Every function is sync (GUI-paint safe), never panics, and returns
//! `serde_json::Value` with `"live": bool` so callers can tell oracle
//! from hardware. No hardware is required for `cargo test`.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde_json::{json, Value};
use std::io::{BufRead, Read, Write};
use std::time::Duration;

use super::{BusPiratePinState, HardwareAdapter};

const MOONRAKER_DEFAULT: &str = "http://127.0.0.1:7125";
const HTTP_TIMEOUT_SECS: u64 = 3;
const SERIAL_TIMEOUT_MS: u64 = 1500;

/// Explicit arg > env var > built-in default (empty default = Demo).
fn resolve_opt(explicit: Option<String>, env_key: &str) -> String {
    if let Some(v) = explicit {
        if !v.trim().is_empty() {
            return v;
        }
    }
    std::env::var(env_key).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Moonraker (Klipper HTTP)
// ---------------------------------------------------------------------------

/// Live Klipper telemetry via Moonraker. `url` like `http://printer:7125`.
#[must_use]
pub fn printer_live(url_opt: Option<String>) -> Value {
    let url = {
        let v = resolve_opt(url_opt, "MOONRAKER_URL");
        if v.is_empty() {
            MOONRAKER_DEFAULT.to_string()
        } else {
            v
        }
    };
    let base = url.trim_end_matches('/');
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(e) => return demo_printer(&url, &format!("http client: {e}")),
    };
    let server: Value = client
        .get(format!("{base}/server/info"))
        .send()
        .ok()
        .and_then(|r| r.json::<Value>().ok())
        .unwrap_or(Value::Null);
    let printer: Value = client
        .get(format!("{base}/printer/info"))
        .send()
        .ok()
        .and_then(|r| r.json::<Value>().ok())
        .unwrap_or(Value::Null);
    if server.is_null() && printer.is_null() {
        return demo_printer(
            &url,
            "moonraker unreachable (is Klipper + Moonraker up? set MOONRAKER_URL)",
        );
    }
    let mock = HardwareAdapter::mock_klipper_printer();
    json!({
        "live": true,
        "url": url,
        "machine_type": "Klipper3DPrinter",
        "is_connected": true,
        "toolhead_fallback_mm": [mock.toolhead_x_mm, mock.toolhead_y_mm, mock.toolhead_z_mm],
        "server": server,
        "printer": printer,
        "note": "toolhead_fallback_mm is the Demo oracle; query /printer/objects for live XYZ (v2.1)",
    })
}

fn demo_printer(url: &str, error: &str) -> Value {
    let mock = HardwareAdapter::mock_klipper_printer();
    json!({
        "live": false,
        "url": url,
        "machine_type": "Klipper3DPrinter",
        "is_connected": false,
        "state": mock.state,
        "toolhead_mm": [mock.toolhead_x_mm, mock.toolhead_y_mm, mock.toolhead_z_mm],
        "extruder_c": mock.extruder_or_spindle_temp,
        "target_c": mock.target_temp,
        "error": error,
        "hint": "set MOONRAKER_URL=http://<printer>:7125 or power on the Ender-3 rig",
    })
}

// ---------------------------------------------------------------------------
// GRBL (serial `?`, read-only)
// ---------------------------------------------------------------------------

/// Probe a GRBL controller with `?` (status report). Never sends motion.
#[must_use]
pub fn cnc_probe(port_opt: Option<String>) -> Value {
    let port = resolve_opt(port_opt, "GRBL_PORT");
    if port.is_empty() {
        return demo_cnc(
            "",
            "no port given (set GRBL_PORT, e.g. COM4 or /dev/ttyUSB0)",
        );
    }
    let opened = serialport::new(port.clone(), 115_200)
        .timeout(Duration::from_millis(SERIAL_TIMEOUT_MS))
        .open();
    let mut stream = match opened {
        Ok(s) => s,
        Err(e) => return demo_cnc(&port, &format!("serial open failed: {e}")),
    };
    // DTR open resets most 3018 controllers: drain the boot banner first
    // (grbl-mcp oracle: ~2 s), then ask. Best-effort — the `?` below is
    // the real gate.
    let mut drain = [0u8; 512];
    let _ = stream.read(&mut drain);
    if stream.write_all(b"?").is_err() {
        return demo_cnc(&port, "serial write failed");
    }
    let mut buf = vec![0u8; 256];
    let raw = match stream.read(&mut buf) {
        Ok(0) => String::new(),
        Ok(n) => String::from_utf8_lossy(&buf[..n]).to_string(),
        Err(e) => return demo_cnc(&port, &format!("serial read failed/timeout: {e}")),
    };
    if !raw.contains('<') {
        return demo_cnc(
            &port,
            "no status report in reply (controller still booting?)",
        );
    }
    let state = parse_grbl_state(&raw);
    let (mpos, wpos) = parse_grbl_positions(&raw);
    // Best-effort firmware identity; a failed `$I` never fails the probe.
    let mut info_buf = [0u8; 256];
    let firmware = if stream.write_all(b"$I\n").is_ok() {
        match stream.read(&mut info_buf) {
            Ok(0) => String::new(),
            Ok(n) => String::from_utf8_lossy(&info_buf[..n])
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };
    let mock = HardwareAdapter::mock_grbl_cnc();
    json!({
        "live": true,
        "port": port,
        "machine_type": "GrblCncRouter",
        "is_connected": true,
        "state": state,
        "machine_pos": mpos.map(|(x, y, z)| vec![x, y, z]),
        "work_pos": wpos.map(|(x, y, z)| vec![x, y, z]),
        "firmware": firmware,
        "raw": raw,
        "spindle_rpm_fallback": mock.extruder_or_spindle_temp,
    })
}

fn parse_grbl_state(raw: &str) -> String {
    let s = raw.trim();
    let inner = s.strip_prefix('<').unwrap_or(s);
    let end = inner.find(['|', '>']).unwrap_or(inner.len());
    normalize_grbl_state(inner[..end].trim())
}

/// Normalize GRBL state + sub-states (`Hold:0`, `Door:3`) the way the
/// firmware means them. Pure — cross-checked against the `grbl-mcp` oracle.
fn normalize_grbl_state(s: &str) -> String {
    let head = s.split(':').next().unwrap_or("");
    match head {
        "Idle" | "Run" | "Hold" | "Jog" | "Alarm" | "Door" | "Check" | "Home" | "Sleep" => {
            head.to_string()
        }
        _ if head.is_empty() => "UNKNOWN".to_string(),
        _ => s.to_string(),
    }
}

/// A parsed XYZ triple, if the report carried one.
type GrblPos = Option<(f64, f64, f64)>;

/// Extract `MPos:` and `WPos:` triples from a `<...>` status report. Pure.
fn parse_grbl_positions(raw: &str) -> (GrblPos, GrblPos) {
    let inner = raw.trim().strip_prefix('<').unwrap_or(raw.trim());
    let mut mpos = None;
    let mut wpos = None;
    for field in inner.split(['|', '>']) {
        if let Some(p) = field.strip_prefix("MPos:") {
            mpos = parse_triple(p);
        } else if let Some(p) = field.strip_prefix("WPos:") {
            wpos = parse_triple(p);
        }
    }
    (mpos, wpos)
}

fn parse_triple(s: &str) -> Option<(f64, f64, f64)> {
    let mut it = s.split(',');
    let x: f64 = it.next()?.trim().parse().ok()?;
    let y: f64 = it.next()?.trim().parse().ok()?;
    let z: f64 = it.next()?.trim().parse().ok()?;
    Some((x, y, z))
}

fn demo_cnc(port: &str, error: &str) -> Value {
    let mock = HardwareAdapter::mock_grbl_cnc();
    let ports: Vec<String> = serialport::available_ports()
        .map(|list| list.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default();
    json!({
        "live": false,
        "port": port,
        "machine_type": "GrblCncRouter",
        "is_connected": false,
        "state": mock.state,
        "toolhead_mm": [mock.toolhead_x_mm, mock.toolhead_y_mm, mock.toolhead_z_mm],
        "available_ports": ports,
        "error": error,
        "hint": "set GRBL_PORT to the 3018 USB serial port; read-only `?` probe, no motion ever",
    })
}

// ---------------------------------------------------------------------------
// Bus Pirate (serial text-mode identity, read-only)
// ---------------------------------------------------------------------------

/// Identify a Bus Pirate over serial (text-mode `i`). Pin levels stay mock
/// until binary-mode ADC lands; this call never drives pins.
#[must_use]
pub fn bus_probe(port_opt: Option<String>) -> Value {
    let port = resolve_opt(port_opt, "BUS_PIRATE_PORT");
    if port.is_empty() {
        return demo_bus("", "no port given (set BUS_PIRATE_PORT)");
    }
    let opened = serialport::new(port.clone(), 115_200)
        .timeout(Duration::from_millis(SERIAL_TIMEOUT_MS))
        .open();
    let mut stream = match opened {
        Ok(s) => s,
        Err(e) => return demo_bus(&port, &format!("serial open failed: {e}")),
    };
    if stream.write_all(b"i\n").is_err() {
        return demo_bus(&port, "serial write failed");
    }
    let mut buf = vec![0u8; 512];
    let banner = match stream.read(&mut buf) {
        Ok(0) => String::new(),
        Ok(n) => String::from_utf8_lossy(&buf[..n]).to_string(),
        Err(e) => return demo_bus(&port, &format!("serial read failed/timeout: {e}")),
    };
    json!({
        "live": true,
        "port": port,
        "banner": banner,
        "pins": HardwareAdapter::mock_bus_pirate(),
        "note": "pin levels are still the Demo oracle; binary-mode ADC lands in v3",
    })
}

fn demo_bus(port: &str, error: &str) -> Value {
    let pins: Vec<BusPiratePinState> = HardwareAdapter::mock_bus_pirate();
    let ports: Vec<String> = serialport::available_ports()
        .map(|list| list.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default();
    json!({
        "live": false,
        "port": port,
        "pins": pins,
        "available_ports": ports,
        "error": error,
        "hint": "set BUS_PIRATE_PORT to the Bus Pirate v4 USB serial port @3.3V",
    })
}

// ---------------------------------------------------------------------------
// SCPI instrument (oscilloscope / PSU / DMM over TCP, `*IDN?`)
// ---------------------------------------------------------------------------

/// Identify any SCPI instrument over TCP (`*IDN?`). Covers oscilloscopes,
/// bench PSUs, and DMMs with LAN (VXI-11 raw). Demo when unreachable.
#[must_use]
pub fn scope_idn(addr_opt: Option<String>) -> Value {
    let addr = resolve_opt(addr_opt, "SCOPE_ADDR");
    if addr.is_empty() {
        return demo_scope("", "no addr given (set SCOPE_ADDR, e.g. 192.168.1.50:5025)");
    }
    let socket: std::net::SocketAddr = match addr.parse() {
        Ok(a) => a,
        Err(e) => return demo_scope(&addr, &format!("bad addr (want ip:port): {e}")),
    };
    let stream = std::net::TcpStream::connect_timeout(&socket, Duration::from_secs(2));
    let stream = match stream {
        Ok(s) => s,
        Err(e) => return demo_scope(&addr, &format!("tcp connect failed: {e}")),
    };
    let mut stream = stream;
    if stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .is_err()
    {
        return demo_scope(&addr, "set_read_timeout failed");
    }
    if stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .is_err()
    {
        return demo_scope(&addr, "set_write_timeout failed");
    }
    if stream.write_all(b"*IDN?\n").is_err() {
        return demo_scope(&addr, "scpi write failed");
    }
    let mut line = String::new();
    let mut reader = std::io::BufReader::new(&stream);
    let idn = match reader.read_line(&mut line) {
        Ok(0) => String::new(),
        Ok(_) => line.trim().to_string(),
        Err(e) => return demo_scope(&addr, &format!("scpi read failed: {e}")),
    };
    if idn.is_empty() {
        return demo_scope(&addr, "empty *IDN? reply");
    }
    json!({
        "live": true,
        "addr": addr,
        "idn": idn,
        "waveform_hint": "use logic.capture for the Demo 4-pin pattern; SCPI waveform fetch lands in v3",
    })
}

fn demo_scope(addr: &str, error: &str) -> Value {
    json!({
        "live": false,
        "addr": addr,
        "idn": "DEMO,SCOPE,0,v2-demo",
        "waveform": demo_sine_8(),
        "error": error,
        "hint": "set SCOPE_ADDR=<scope-lan-ip>:5025; Demo trace is an 8-sample sine",
    })
}

fn demo_sine_8() -> Vec<f64> {
    let mut out = Vec::with_capacity(8);
    let mut i = 0;
    while i < 8 {
        let v = ((i as f64) * std::f64::consts::PI / 4.0).sin();
        out.push((v * 100.0).round() / 100.0);
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Logic sampler (4-pin Demo pattern; SUMP firmware lands in v3)
// ---------------------------------------------------------------------------

/// Sample the 4 Bus Pirate pins as digital channels. Today the pattern is
/// deterministic Demo (phase-shifted squares); when a serial port opens
/// successfully the envelope reports `"live": true` with
/// `"samples_source": "demo"` so agents never mistake it for a capture.
#[must_use]
pub fn logic_capture(port_opt: Option<String>, hz: u32, samples: usize) -> Value {
    let port = resolve_opt(port_opt, "BUS_PIRATE_PORT");
    let hz = hz.clamp(1, 100_000);
    let samples = samples.clamp(1, 256);
    let port_ok = if port.is_empty() {
        false
    } else {
        serialport::new(port.clone(), 115_200)
            .timeout(Duration::from_millis(400))
            .open()
            .is_ok()
    };
    let pins = ["MOSI", "MISO", "CLK", "CS"];
    let mut channels = serde_json::Map::new();
    for (ch, name) in pins.iter().enumerate() {
        let mut seq = Vec::with_capacity(samples);
        let mut i = 0;
        while i < samples {
            let high = ((i + ch * 2) % 4) < 2;
            seq.push(high);
            i += 1;
        }
        channels.insert((*name).to_string(), Value::from(seq));
    }
    json!({
        "live": port_ok,
        "port": port,
        "hz": hz,
        "samples": samples,
        "channels": channels,
        "samples_source": "demo (SUMP capture lands in v3)",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moonraker_unreachable_is_demo_not_panic() {
        let v = printer_live(Some("http://127.0.0.1:9".to_string()));
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
    }

    #[test]
    fn grbl_bad_port_is_demo_not_panic() {
        let v = cnc_probe(Some("DEMO_DOES_NOT_EXIST_000".to_string()));
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
        assert_eq!(
            v.get("machine_type").and_then(|x| x.as_str()),
            Some("GrblCncRouter")
        );
    }

    #[test]
    fn bus_bad_port_is_demo_not_panic() {
        let v = bus_probe(Some("DEMO_DOES_NOT_EXIST_000".to_string()));
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
    }

    #[test]
    fn scope_closed_port_is_demo_not_panic() {
        let v = scope_idn(Some("127.0.0.1:9".to_string()));
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
        assert!(v.get("waveform").and_then(|w| w.as_array()).is_some());
    }

    #[test]
    fn scope_bad_addr_is_demo_not_panic() {
        let v = scope_idn(Some("not-an-addr".to_string()));
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
    }

    #[test]
    fn grbl_state_parses_idle() {
        assert_eq!(parse_grbl_state("<Idle|MPos:0,0,5.0|FS:0,0>"), "Idle");
        assert_eq!(parse_grbl_state(""), "UNKNOWN");
        assert_eq!(parse_grbl_state("garbage"), "garbage");
        // Sub-states normalize the way the firmware means them.
        assert_eq!(parse_grbl_state("<Hold:1|MPos:0,0,0>"), "Hold");
        assert_eq!(parse_grbl_state("<Door:3|MPos:0,0,0>"), "Door");
    }

    #[test]
    fn grbl_positions_parse() {
        let (m, w) = parse_grbl_positions("<Run|MPos:10.5,-2.0,5.0|WPos:1.0,2.0,3.0>");
        assert_eq!(m, Some((10.5, -2.0, 5.0)));
        assert_eq!(w, Some((1.0, 2.0, 3.0)));
        let (m2, w2) = parse_grbl_positions("<Idle|MPos:0,0,0>");
        assert!(m2.is_some());
        assert_eq!(w2, None);
    }

    #[test]
    fn logic_capture_shape_and_clamps() {
        let v = logic_capture(Some("".to_string()), 0, 9999);
        assert_eq!(v.get("hz").and_then(|x| x.as_u64()), Some(1));
        assert_eq!(v.get("samples").and_then(|x| x.as_u64()), Some(256));
        let ch = v
            .get("channels")
            .and_then(|c| c.as_object())
            .cloned()
            .unwrap_or_default();
        assert_eq!(ch.len(), 4);
    }
}
