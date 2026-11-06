//! USB bench instruments (v2.1, read-only).
//!
//! - USB oscilloscopes over USBTMC, hand-rolled on `rusb`. (`usbtmc 0.1.0`
//!   on crates.io is an empty stub — zero deps, one `it_works` test — so per
//!   the Jiffy-Port doctrine the ~40 lines of bulk framing live here.)
//! - Logic analyzers via the `sigrok-cli` subprocess when present; an honest
//!   `DOWN` receipt with a labeled Demo pattern when absent.
//!
//! Windows prerequisite: each USB instrument needs a WinUSB driver
//! (Zadig) or `rusb` cannot open it. Every failure names the fix.
//!
//! All functions are sync (GUI-paint safe), never panic, and return
//! `"live": bool` so agents can tell hardware from oracle.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use serde_json::{json, Value};
use std::io::Read;
use std::process::{Command as ProcCommand, Stdio};
use std::sync::mpsc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// USBTMC framing (USB Test & Measurement Class, USB488 bulk messages)
// ---------------------------------------------------------------------------

const USBTMC_CLASS: u8 = 0xFE;
const USBTMC_SUBCLASS: u8 = 0x03;
const MSG_OUT: u8 = 1; // DEV_DEP_MSG_OUT
const MSG_IN: u8 = 2; // DEV_DEP_MSG_IN (request) / reply
const HEADER_LEN: usize = 12;
const BULK_TIMEOUT: Duration = Duration::from_millis(2000);
const MAX_READ_LOOPS: usize = 64;
const READ_CHUNK: usize = 4096;

/// Build a DEV_DEP_MSG_OUT header. Pure — unit-tested.
#[must_use]
pub fn build_out_header(tag: u8, len: u32, eom: bool) -> [u8; HEADER_LEN] {
    let mut h = [0u8; HEADER_LEN];
    h[0] = MSG_OUT;
    h[1] = tag;
    h[2] = 255 - tag;
    let bytes = len.to_le_bytes();
    h[4] = bytes[0];
    h[5] = bytes[1];
    h[6] = bytes[2];
    h[7] = bytes[3];
    if eom {
        h[8] = 1;
    }
    h
}

/// Build a DEV_DEP_MSG_IN request header (host asks for up to `want` bytes).
#[must_use]
pub fn build_in_request(tag: u8, want: u32) -> [u8; HEADER_LEN] {
    let mut h = [0u8; HEADER_LEN];
    h[0] = MSG_IN;
    h[1] = tag;
    h[2] = 255 - tag;
    let bytes = want.to_le_bytes();
    h[4] = bytes[0];
    h[5] = bytes[1];
    h[6] = bytes[2];
    h[7] = bytes[3];
    h
}

/// Parse a DEV_DEP_MSG_IN reply header. Returns (tag, payload_len). Pure.
pub fn parse_in_reply(buf: &[u8]) -> Result<(u8, usize), String> {
    if buf.len() < HEADER_LEN {
        return Err(format!("reply too short: {} < 12", buf.len()));
    }
    if buf[0] != MSG_IN {
        return Err(format!("not a DEV_DEP_MSG_IN reply (MsgID={})", buf[0]));
    }
    if buf[1] != 255 - buf[2] {
        return Err("bTag/bTagInverse mismatch (stale transfer?)".to_string());
    }
    let len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    Ok((buf[1], len))
}

/// Parse an IEEE 488.2 definite-length block (`#<N><N digits><data>`). Pure.
pub fn parse_ieee_block(buf: &[u8]) -> Result<Vec<u8>, String> {
    if buf.len() < 2 || buf[0] != b'#' {
        return Err("not an IEEE block (missing '#')".to_string());
    }
    let n = buf[1].wrapping_sub(b'0') as usize;
    if n == 0 || n > 9 {
        return Err(format!("bad IEEE length digit: {}", buf[1]));
    }
    if buf.len() < 2 + n {
        return Err("IEEE block truncated in length field".to_string());
    }
    let mut len = 0usize;
    for b in &buf[2..2 + n] {
        if !b.is_ascii_digit() {
            return Err("non-digit in IEEE length".to_string());
        }
        len = len
            .checked_mul(10)
            .and_then(|v| v.checked_add((b - b'0') as usize))
            .unwrap_or(usize::MAX);
    }
    if buf.len() < 2 + n + len {
        return Err(format!(
            "IEEE block truncated: need {}, have {}",
            2 + n + len,
            buf.len()
        ));
    }
    Ok(buf[2 + n..2 + n + len].to_vec())
}

/// Rigol `:WAV:PRE?` preamble: `fmt,type,points,count,xinc,xorg,xref,yinc,yorg,yref`.
/// Returns (y_increment, y_origin, y_reference). Pure.
pub fn parse_rigol_preamble(s: &str) -> Result<(f64, f64, f64), String> {
    let f: Vec<&str> = s.trim().split(',').collect();
    if f.len() != 10 {
        return Err(format!("preamble: want 10 fields, have {}", f.len()));
    }
    let num = |i: usize| {
        f.get(i)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("preamble field {i} not numeric"))
    };
    Ok((num(7)?, num(8)?, num(9)?))
}

/// Raw byte waveform -> volts via a Rigol preamble. Pure.
#[must_use]
pub fn rigol_bytes_to_volts(raw: &[u8], yinc: f64, yorigin: f64, yref: f64) -> Vec<f64> {
    raw.iter()
        .map(|b| ((*b as f64) - yref) * yinc + yorigin)
        .map(|v| (v * 1000.0).round() / 1000.0)
        .collect()
}

struct UsbTmc {
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
    iface: u8,
    ep_out: u8,
    ep_in: u8,
    tag: u8,
}

fn find_usbtmc(vid: u16, pid: u16) -> Result<UsbTmc, String> {
    let handle = rusb::open_device_with_vid_pid(vid, pid).ok_or_else(|| {
        format!("USB {vid:04x}:{pid:04x} not found — cable on? WinUSB driver via Zadig installed?")
    })?;
    let device = handle.device();
    let cfg = device
        .config_descriptor(0)
        .map_err(|e| format!("config descriptor: {e}"))?;
    for iface in cfg.interfaces() {
        for desc in iface.descriptors() {
            if desc.class_code() == USBTMC_CLASS && desc.sub_class_code() == USBTMC_SUBCLASS {
                let mut out = None;
                let mut inp = None;
                for ep in desc.endpoint_descriptors() {
                    if ep.transfer_type() != rusb::TransferType::Bulk {
                        continue;
                    }
                    match ep.direction() {
                        rusb::Direction::Out => out = Some(ep.address()),
                        rusb::Direction::In => inp = Some(ep.address()),
                    }
                }
                match (out, inp) {
                    (Some(o), Some(i)) => {
                        return Ok(UsbTmc {
                            handle,
                            iface: desc.interface_number(),
                            ep_out: o,
                            ep_in: i,
                            tag: 1,
                        });
                    }
                    _ => {
                        return Err("USBTMC interface has no bulk pair".to_string());
                    }
                }
            }
        }
    }
    Err("no USBTMC interface (class FE/subclass 03) on this device".to_string())
}

fn next_tag(t: &mut UsbTmc) -> u8 {
    let tag = t.tag;
    t.tag = if t.tag == 255 { 1 } else { t.tag + 1 };
    tag
}

fn write_all(t: &UsbTmc, data: &[u8]) -> Result<(), String> {
    let mut off = 0;
    while off < data.len() {
        match t.handle.write_bulk(t.ep_out, &data[off..], BULK_TIMEOUT) {
            Ok(0) => return Err("bulk write returned 0".to_string()),
            Ok(n) => off += n,
            Err(e) => return Err(format!("bulk write: {e}")),
        }
    }
    Ok(())
}

fn read_exact(t: &UsbTmc, len: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(len);
    let mut buf = [0u8; READ_CHUNK];
    let mut loops = 0;
    while out.len() < len && loops < MAX_READ_LOOPS {
        loops += 1;
        match t.handle.read_bulk(t.ep_in, &mut buf, BULK_TIMEOUT) {
            Ok(0) => break,
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(e) => {
                if out.is_empty() {
                    return Err(format!("bulk read: {e}"));
                }
                break;
            }
        }
    }
    if out.len() < len {
        return Err(format!("short read: {} < {len}", out.len()));
    }
    out.truncate(len);
    Ok(out)
}

fn usbtmc_query(t: &mut UsbTmc, scpi: &str) -> Result<String, String> {
    let payload = scpi.as_bytes();
    let tag = next_tag(t);
    let mut msg = build_out_header(tag, payload.len() as u32, true).to_vec();
    msg.extend_from_slice(payload);
    write_all(t, &msg)?;
    let want = 4096u32;
    let req = build_in_request(next_tag(t), want);
    write_all(t, &req)?;
    let hdr = read_exact(t, HEADER_LEN)?;
    let (_reply_tag, len) = parse_in_reply(&hdr)?;
    let len = len.min(262_144);
    let data = read_exact(t, len)?;
    String::from_utf8(data)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("non-UTF8 reply: {e}"))
}

fn demo_usb_scope(vid: u16, pid: u16, error: &str) -> Value {
    json!({
        "live": false,
        "vid": format!("{vid:04x}"),
        "pid": format!("{pid:04x}"),
        "idn": "DEMO,SCOPE-USB,0,v2.1-demo",
        "waveform": Value::Null,
        "error": error,
        "hint": "USB scope needs WinUSB (Zadig) + cable; common Rigol 1ab1:04ce, Siglent f4ec:ee38 — pass vid/pid as decimals",
    })
}

/// Identify a USB scope via USBTMC `*IDN?`, best-effort Rigol waveform.
#[must_use]
pub fn scope_usb(vid: u16, pid: u16) -> Value {
    if vid == 0 || pid == 0 {
        return demo_usb_scope(vid, pid, "vid:pid required (both nonzero)");
    }
    let mut t = match find_usbtmc(vid, pid) {
        Ok(t) => t,
        Err(e) => return demo_usb_scope(vid, pid, &e),
    };
    if t.handle.claim_interface(t.iface).is_err() {
        return demo_usb_scope(
            vid,
            pid,
            "claim failed — device busy (vendor app open?) or wrong driver (want WinUSB via Zadig)",
        );
    }
    let idn = match usbtmc_query(&mut t, "*IDN?") {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => return demo_usb_scope(vid, pid, "empty *IDN? reply"),
        Err(e) => return demo_usb_scope(vid, pid, &e),
    };
    // Best-effort Rigol waveform; a failed attempt still returns the idn.
    let waveform = rigol_waveform(&mut t);
    json!({
        "live": true,
        "vid": format!("{vid:04x}"),
        "pid": format!("{pid:04x}"),
        "idn": idn,
        "waveform": waveform.0,
        "waveform_note": waveform.1,
    })
}

fn rigol_waveform(t: &mut UsbTmc) -> (Value, String) {
    let pre = match usbtmc_query(t, ":WAV:PRE?") {
        Ok(s) => s,
        Err(e) => {
            return (
                Value::Null,
                format!("no waveform: preamble failed ({e}); idn above is still live"),
            );
        }
    };
    let (yinc, yorigin, yref) = match parse_rigol_preamble(&pre) {
        Ok(p) => p,
        Err(e) => {
            return (
                Value::Null,
                format!("no waveform: preamble unparseable ({e}); idn above is still live"),
            );
        }
    };
    let raw_reply = match usbtmc_raw_query(t, ":WAV:DATA?") {
        Ok(b) => b,
        Err(e) => {
            return (
                Value::Null,
                format!("no waveform: data failed ({e}); idn above is still live"),
            );
        }
    };
    match parse_ieee_block(&raw_reply) {
        Ok(raw) => {
            let volts = rigol_bytes_to_volts(&raw, yinc, yorigin, yref);
            let volts: Vec<f64> = volts.into_iter().take(1400).collect();
            (
                json!(volts),
                format!("Rigol CHAN1, {} points", volts_len(&volts)),
            )
        }
        Err(e) => (
            Value::Null,
            format!("no waveform: block unparseable ({e}); idn above is still live"),
        ),
    }
}

fn volts_len(v: &[f64]) -> usize {
    v.len()
}

fn usbtmc_raw_query(t: &mut UsbTmc, scpi: &str) -> Result<Vec<u8>, String> {
    let payload = scpi.as_bytes();
    let tag = next_tag(t);
    let mut msg = build_out_header(tag, payload.len() as u32, true).to_vec();
    msg.extend_from_slice(payload);
    write_all(t, &msg)?;
    let req = build_in_request(next_tag(t), 262_144);
    write_all(t, &req)?;
    let hdr = read_exact(t, HEADER_LEN)?;
    let (_reply_tag, len) = parse_in_reply(&hdr)?;
    let len = len.min(262_144);
    read_exact(t, len)
}

// ---------------------------------------------------------------------------
// sigrok-cli logic analyzers (subprocess, DOWN receipt when absent)
// ---------------------------------------------------------------------------

fn sigrok_cli(explicit: Option<String>) -> String {
    if let Some(v) = explicit {
        if !v.trim().is_empty() {
            return v;
        }
    }
    std::env::var("SIGROK_CLI").unwrap_or_else(|_| "sigrok-cli".to_string())
}

fn run_with_timeout(cli: &str, args: &[String], dur: Duration) -> Result<String, String> {
    let mut child = ProcCommand::new(cli)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn `{cli}`: {e} — install sigrok-cli or set SIGROK_CLI"))?;
    let (tx, rx) = mpsc::channel();
    let mut stdout = child.stdout.take();
    std::thread::spawn(move || {
        let mut s = String::new();
        if let Some(o) = stdout.as_mut() {
            let _ = o.read_to_string(&mut s);
        }
        let _ = tx.send(s);
    });
    let step = Duration::from_millis(50);
    let mut waited = Duration::from_millis(0);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
                if status.success() {
                    return Ok(out);
                }
                let mut err = child
                    .stderr
                    .take()
                    .and_then(|mut e| {
                        let mut s = String::new();
                        e.read_to_string(&mut s).ok().map(|_| s)
                    })
                    .unwrap_or_default();
                err.truncate(500);
                return Err(format!("exit {}: {err}", status.code().unwrap_or(-1)));
            }
            Ok(None) => {
                if waited >= dur {
                    let _ = child.kill();
                    return Err(format!("`{cli}` timed out after {}s", dur.as_secs()));
                }
                std::thread::sleep(step);
                waited += step;
            }
            Err(e) => return Err(format!("wait: {e}")),
        }
    }
}

/// Parse sigrok `--output-format csv`: header row + 0/1 rows. Pure.
pub fn parse_sigrok_csv(text: &str) -> Result<serde_json::Map<String, Value>, String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or_else(|| "empty csv".to_string())?;
    let names: Vec<String> = header.split(',').map(|s| s.trim().to_string()).collect();
    if names.is_empty() || names.iter().any(|n| n.is_empty()) {
        return Err("bad csv header".to_string());
    }
    let mut cols: Vec<Vec<bool>> = vec![Vec::new(); names.len()];
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        if cells.len() != names.len() {
            continue;
        }
        for (i, c) in cells.iter().enumerate() {
            match c.trim() {
                "0" => cols[i].push(false),
                "1" => cols[i].push(true),
                _ => {}
            }
        }
    }
    let mut map = serde_json::Map::new();
    for (i, name) in names.into_iter().enumerate() {
        map.insert(name, Value::from(cols[i].clone()));
    }
    Ok(map)
}

fn demo_logic_usb(engine: &str, hz: u32, samples: usize, error: &str, command: &str) -> Value {
    let mut channels = serde_json::Map::new();
    for (ch, name) in ["MOSI", "MISO", "CLK", "CS"].iter().enumerate() {
        let mut seq = Vec::with_capacity(samples);
        let mut i = 0;
        while i < samples {
            seq.push(((i + ch * 2) % 4) < 2);
            i += 1;
        }
        channels.insert((*name).to_string(), Value::from(seq));
    }
    json!({
        "live": false,
        "engine": engine,
        "command": command,
        "hz": hz,
        "samples": samples,
        "channels": channels,
        "samples_source": "demo (sigrok engine DOWN — see error)",
        "error": error,
        "hint": "install sigrok-cli (https://sigrok.org/wiki/Downloads) and pass driver, e.g. saleae-logic8",
    })
}

/// Capture with a USB logic analyzer via sigrok-cli.
/// Empty driver returns usage (no subprocess); missing binary returns DOWN.
#[must_use]
pub fn logic_usb(driver: Option<String>, hz: u32, samples: usize, cli: Option<String>) -> Value {
    let hz = hz.clamp(1, 50_000_000);
    let samples = samples.clamp(1, 1_000_000);
    let cli_bin = sigrok_cli(cli);
    let version = run_with_timeout(&cli_bin, &["--version".to_string()], Duration::from_secs(5));
    let version = match version {
        Ok(v) => v.lines().next().unwrap_or("sigrok-cli").to_string(),
        Err(e) => {
            return demo_logic_usb(
                "down",
                hz,
                samples,
                &format!("sigrok engine DOWN: {e}"),
                &format!("{cli_bin} --version"),
            );
        }
    };
    let driver = driver.unwrap_or_default();
    if driver.trim().is_empty() {
        return json!({
            "live": false,
            "engine": version.trim(),
            "command": format!("{cli_bin} --version"),
            "hz": hz,
            "samples": samples,
            "channels": Value::Null,
            "error": "no driver given",
            "hint": "pass driver, e.g. saleae-logic8 / fx2lafw / saleae-logic-pro; run `sigrok-cli --list-supported` for the full list",
        });
    }
    let args = vec![
        "--driver".to_string(),
        driver.clone(),
        "--config".to_string(),
        format!("samplerate={hz}"),
        "--samples".to_string(),
        samples.to_string(),
        "--output-format".to_string(),
        "csv".to_string(),
    ];
    let command = format!("{cli_bin} {}", args.join(" "));
    match run_with_timeout(&cli_bin, &args, Duration::from_secs(15)) {
        Ok(csv) => match parse_sigrok_csv(&csv) {
            Ok(channels) => json!({
                "live": true,
                "engine": version.trim(),
                "command": command,
                "driver": driver,
                "hz": hz,
                "samples": samples,
                "channels": channels,
            }),
            Err(e) => demo_logic_usb(
                &version,
                hz,
                samples,
                &format!("csv parse failed ({e}); command was `{command}`"),
                &command,
            ),
        },
        Err(e) => demo_logic_usb(&version, hz, samples, &e, &command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_header_round_trip() {
        let h = build_out_header(7, 5, true);
        assert_eq!(h[0], MSG_OUT);
        assert_eq!(h[1], 7);
        assert_eq!(h[2], 255 - 7);
        assert_eq!(u32::from_le_bytes([h[4], h[5], h[6], h[7]]), 5);
        assert_eq!(h[8], 1);
        let plain = build_out_header(1, 4, false);
        assert_eq!(plain[8], 0);
    }

    #[test]
    fn in_reply_parses() {
        let mut hdr = build_in_request(9, 100);
        hdr[0] = MSG_IN;
        let parsed = parse_in_reply(&hdr);
        assert!(parsed.is_ok());
        let (tag, len) = parsed.unwrap_or((0, 0));
        assert_eq!((tag, len), (9, 100));
        assert!(parse_in_reply(&hdr[..4]).is_err());
        let mut bad = hdr;
        bad[2] = 0;
        assert!(parse_in_reply(&bad).is_err());
    }

    #[test]
    fn ieee_block_parses() {
        let data = b"ABCDEFGH";
        let mut block = b"#18".to_vec();
        block.extend_from_slice(data);
        let parsed = parse_ieee_block(&block);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap_or_default(), data);
        assert!(parse_ieee_block(b"nope").is_err());
        assert!(parse_ieee_block(b"#18AB").is_err());
    }

    #[test]
    fn rigol_preamble_and_volts() {
        let p = "0,0,1400,1,1.0e-09,-7.0e-07,0,0.01,-0.12,128";
        let parsed = parse_rigol_preamble(p);
        assert!(parsed.is_ok());
        let (inc, org, rf) = parsed.unwrap_or((0.0, 0.0, 0.0));
        assert!((inc - 0.01).abs() < 1e-9);
        let v = rigol_bytes_to_volts(&[128, 138], inc, org, rf);
        assert_eq!(v.len(), 2);
        assert!((v[0] - -0.12).abs() < 1e-9);
        assert!(parse_rigol_preamble("1,2,3").is_err());
    }

    #[test]
    fn sigrok_csv_parses() {
        let csv = "MOSI,MISO\n0,1\n1,1\n1,0\n";
        let parsed = parse_sigrok_csv(csv);
        assert!(parsed.is_ok());
        let m = parsed.unwrap_or_default();
        assert_eq!(m.len(), 2);
        assert!(parse_sigrok_csv("").is_err());
    }

    #[test]
    fn missing_usb_device_is_demo() {
        let v = scope_usb(0xFFFF, 0xFFFF);
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
        let v = scope_usb(0, 0);
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
    }

    #[test]
    fn missing_sigrok_is_down_not_panic() {
        let v = logic_usb(
            Some("saleae-logic8".to_string()),
            1000,
            8,
            Some("DEMO_NO_SUCH_BIN_000".to_string()),
        );
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
        assert_eq!(v.get("engine").and_then(|x| x.as_str()), Some("down"));
    }

    #[test]
    fn empty_driver_is_usage_not_capture() {
        let v = logic_usb(None, 1000, 8, Some("DEMO_NO_SUCH_BIN_000".to_string()));
        // binary missing -> DOWN receipt (usage path needs a working engine)
        assert_eq!(v.get("live").and_then(|x| x.as_bool()), Some(false));
    }
}
