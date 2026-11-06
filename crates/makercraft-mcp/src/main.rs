//! MakerCraft MCP Server (L2)
//!
//! CLI-first binary: humans run subcommands, MCP clients run `mcp` stdio.
//! Both construct a [`makercraft_core::Command`] and pass it to
//! [`makercraft_core::dispatch`], so CLI and MCP can never disagree.
//!
//! Stdio invariant: in `mcp` mode `stdout` is 100% pure line-delimited
//! JSON-RPC (`MCP 2024-11-05`). Every diagnostic goes to `stderr`.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use clap::{Parser, Subcommand};
use makercraft_core::{dispatch, Command};
use serde_json::{json, Value};
use std::io::{BufRead, Write as _};

#[derive(Parser, Debug)]
#[command(
    name = "makercraft-mcp",
    about = "MakerCraft Digital Fabrication Bench (CLI + MCP stdio)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bench status + capability catalog (JSON)
    Status,
    /// Klipper printer telemetry (JSON)
    PrinterStatus,
    /// GRBL CNC telemetry (JSON)
    CncStatus,
    /// Bus Pirate pin levels (JSON)
    BusPins,
    /// Lint a G-code file (JSON report)
    Lint {
        path: String,
        #[arg(long, default_value = "cnc")]
        profile: String,
    },
    /// Check a file; exit 2 when unsafe (CI gate)
    Check {
        path: String,
        #[arg(long, default_value = "cnc")]
        profile: String,
    },
    /// Estimate time + travel for a G-code file (JSON)
    Estimate {
        path: String,
        #[arg(long, default_value = "cnc")]
        profile: String,
    },
    /// Preview motion segments for a G-code file (JSON)
    Preview { path: String },
    /// Run the MCP 2024-11-05 stdio server (stdout = pure JSON-RPC)
    Mcp,
}

fn read_or_sample(path: &str) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|_| "G21\nG90\nG0 Z25\nG1 X10 Y10 F800\nM2".to_string())
}

fn run(cmd: Command) -> Value {
    dispatch(cmd)
}

fn show(value: &Value) {
    if let Ok(text) = serde_json::to_string_pretty(value) {
        println!("{text}");
    } else {
        println!("{value}");
    }
}

/// Pure catalog so tests exercise it without spawning processes.
#[must_use]
pub fn tools_list() -> Value {
    json!({
        "tools": [
            {"name": "status", "description": "Bench status + capability catalog + reference rig", "inputSchema": {"type": "object", "properties": {}}},
            {"name": "printer_status", "description": "Klipper printer telemetry (mock until hardware-in-the-loop)", "inputSchema": {"type": "object", "properties": {}}},
            {"name": "cnc_status", "description": "GRBL CNC telemetry (mock until hardware-in-the-loop)", "inputSchema": {"type": "object", "properties": {}}},
            {"name": "bus_pins", "description": "Bus Pirate pin logic levels + pullups", "inputSchema": {"type": "object", "properties": {}}},
            {"name": "gcode_lint", "description": "Lint G-code: envelope + plunge + temp + feed gates. Args: gcode, profile (cnc|printer)", "inputSchema": {"type": "object", "properties": {"gcode": {"type": "string"}, "profile": {"type": "string"}}, "required": ["gcode"]}},
            {"name": "gcode_preview", "description": "Motion segments for 2D preview. Args: gcode", "inputSchema": {"type": "object", "properties": {"gcode": {"type": "string"}}, "required": ["gcode"]}},
            {"name": "gcode_check", "description": "Lint G-code for CI (same as lint, machine-readable passes_safety). Args: gcode, profile", "inputSchema": {"type": "object", "properties": {"gcode": {"type": "string"}, "profile": {"type": "string"}}, "required": ["gcode"]}},
            {"name": "gcode_estimate", "description": "Time + travel estimate. Args: gcode, profile", "inputSchema": {"type": "object", "properties": {"gcode": {"type": "string"}, "profile": {"type": "string"}}, "required": ["gcode"]}}
        ]
    })
}

fn arg_str(args: &Value, key: &str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn content_text(result: &Value) -> String {
    result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string()
}

/// Pure call so tests exercise it without pipes. Accepts `gcode.lint`
/// and `gcode_lint` spellings (strict hosts reject `.` in tool names).
#[must_use]
pub fn call_tool(name: &str, args: &Value) -> Value {
    let profile = {
        let p = arg_str(args, "profile");
        if p.is_empty() {
            "cnc".to_string()
        } else {
            p
        }
    };
    let value = match name {
        "status" => run(Command::Status),
        "printer_status" | "printer.status" => run(Command::PrinterStatus),
        "cnc_status" | "cnc.status" => run(Command::CncStatus),
        "bus_pins" | "bus.pins" => run(Command::BusPins),
        "gcode_lint" | "gcode.lint" => run(Command::GcodeLint {
            gcode: arg_str(args, "gcode"),
            profile,
        }),
        "gcode_preview" | "gcode.preview" => run(Command::GcodePreview {
            gcode: arg_str(args, "gcode"),
        }),
        "gcode_check" | "gcode.check" => run(Command::GcodeCheck {
            gcode: arg_str(args, "gcode"),
            profile,
        }),
        "gcode_estimate" | "gcode.estimate" => run(Command::GcodeEstimate {
            gcode: arg_str(args, "gcode"),
            profile,
        }),
        other => json!({"error": format!("unknown tool: {other}")}),
    };
    let text = serde_json::to_string_pretty(&value)
        .unwrap_or_else(|e| format!("{{\"error\":\"encode: {e}\"}}"));
    json!({"content": [{"type": "text", "text": text}]})
}

/// Minimal MCP 2024-11-05 server over stdio. Same `dispatch` as the CLI.
fn run_mcp_stdio() {
    eprintln!("makercraft-mcp: MCP server listening on stdio");
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(e) => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {"code": -32700, "message": format!("parse error: {e}")}
                });
                if let Ok(text) = serde_json::to_string(&resp) {
                    let _ = writeln!(out, "{text}");
                    let _ = out.flush();
                }
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let is_notification = req.get("id").is_none();

        let result: Option<Value> = match method {
            "initialize" => Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "makercraft-mcp", "version": env!("CARGO_PKG_VERSION")},
            })),
            "tools/list" => Some(tools_list()),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(json!({}));
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let tool_args = params.get("arguments").cloned().unwrap_or(json!({}));
                Some(call_tool(tool_name, &tool_args))
            }
            "ping" => Some(json!({})),
            _ => {
                if is_notification {
                    None
                } else {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {"code": -32601, "message": format!("unknown method: {method}")}
                    });
                    if let Ok(text) = serde_json::to_string(&resp) {
                        let _ = writeln!(out, "{text}");
                        let _ = out.flush();
                    }
                    continue;
                }
            }
        };

        if is_notification {
            continue;
        }
        let body = result.unwrap_or(json!({}));
        let resp = json!({"jsonrpc": "2.0", "id": id, "result": body});
        if let Ok(text) = serde_json::to_string(&resp) {
            let _ = writeln!(out, "{text}");
            let _ = out.flush();
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => show(&run(Command::Status)),
        Commands::PrinterStatus => show(&run(Command::PrinterStatus)),
        Commands::CncStatus => show(&run(Command::CncStatus)),
        Commands::BusPins => show(&run(Command::BusPins)),
        Commands::Lint { path, profile } => {
            show(&run(Command::GcodeLint {
                gcode: read_or_sample(&path),
                profile,
            }));
        }
        Commands::Check { path, profile } => {
            let value = run(Command::GcodeCheck {
                gcode: read_or_sample(&path),
                profile,
            });
            show(&value);
            let unsafe_job = value.get("content").is_none()
                && value.get("passes_safety").and_then(|v| v.as_bool()) == Some(false);
            let text = if value.get("content").is_none() {
                serde_json::to_string(&value).unwrap_or_default()
            } else {
                content_text(&value)
            };
            let fails = unsafe_job
                || text.contains("\"passes_safety\":false")
                || text.contains("\"passes_safety\": false");
            if fails {
                std::process::exit(2);
            }
        }
        Commands::Estimate { path, profile } => {
            show(&run(Command::GcodeEstimate {
                gcode: read_or_sample(&path),
                profile,
            }));
        }
        Commands::Preview { path } => {
            show(&run(Command::GcodePreview {
                gcode: read_or_sample(&path),
            }));
        }
        Commands::Mcp => run_mcp_stdio(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_eight_tools() {
        let catalog = tools_list();
        let tools = catalog
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        assert_eq!(tools.len(), 8);
    }

    #[test]
    fn unknown_tool_becomes_error_content_not_panic() {
        let result = call_tool("nope.missing", &json!({}));
        assert!(content_text(&result).contains("unknown tool"));
    }

    #[test]
    fn lint_round_trip_through_call_tool() {
        let result = call_tool(
            "gcode_lint",
            &json!({"gcode": "G21\nG90\nG0 Z25\nG1 X10 Y10 F800\nM2"}),
        );
        let text = content_text(&result);
        assert!(text.contains("passes_safety"));
    }

    #[test]
    fn dotted_and_underscore_names_agree() {
        let a = content_text(&call_tool("gcode.lint", &json!({"gcode": "G21\nG90\nM2"})));
        let b = content_text(&call_tool("gcode_lint", &json!({"gcode": "G21\nG90\nM2"})));
        assert_eq!(a, b);
    }

    #[test]
    fn unsafe_gcode_reports_failure_without_panic() {
        let result = call_tool("gcode_check", &json!({"gcode": "G21\nG90\nG0 Z-5\nM2"}));
        let text = content_text(&result);
        assert!(text.contains("false"));
    }

    #[test]
    fn cli_and_mcp_share_dispatch() {
        let direct = dispatch(Command::Status);
        let via_tool = call_tool("status", &json!({}));
        assert!(content_text(&via_tool).contains("makercraft"));
        assert_eq!(
            direct.get("app").and_then(|v| v.as_str()),
            Some("makercraft")
        );
    }
}
