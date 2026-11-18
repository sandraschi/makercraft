//! Local-first AI Chat Dock (`Ctrl+J`) grounded in MakerCraft's bundled skills
//! and live bench status. Zero fake fallback text: errors surface directly.

use makercraft_core::{dispatch, skills::skills_list, Command, ToolpathLintReport};
use serde_json::json;
use std::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Ollama,
    LmStudio,
    OpenAiCompat,
}

impl LlmProvider {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Ollama => "Ollama (Local)",
            Self::LmStudio => "LM Studio (Local)",
            Self::OpenAiCompat => "OpenAI-Compatible",
        }
    }

    #[must_use]
    pub fn default_url(self) -> &'static str {
        match self {
            Self::Ollama => "http://127.0.0.1:11434",
            Self::LmStudio => "http://127.0.0.1:1234",
            Self::OpenAiCompat => "https://api.openai.com",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub text: String,
    pub is_error: bool,
}

pub struct ChatDockState {
    pub open: bool,
    pub maximized: bool,
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub discovered_models: Vec<String>,
    pub input: String,
    pub messages: Vec<ChatMessage>,
    pub show_prompt_preview: bool,
    pub show_settings: bool,
    pub busy: bool,
    rx: Option<mpsc::Receiver<Result<String, String>>>,
    scan_rx: Option<mpsc::Receiver<Result<Vec<String>, String>>>,
    pub last_user_prompt: Option<String>,
}

impl Default for ChatDockState {
    fn default() -> Self {
        Self {
            open: false,
            maximized: false,
            provider: LlmProvider::Ollama,
            base_url: LlmProvider::Ollama.default_url().to_string(),
            model: String::new(),
            api_key: String::new(),
            discovered_models: Vec::new(),
            input: String::new(),
            messages: vec![ChatMessage {
                role: "system".to_string(),
                text: "MakerCraft AI Assistant ready. Grounded in 8 bundled hardware, Vienna makerspace, and buying-guide skills. Configure or scan a local Ollama/LM Studio model in Settings below to start.".to_string(),
                is_error: false,
            }],
            show_prompt_preview: false,
            show_settings: false,
            busy: false,
            rx: None,
            scan_rx: None,
            last_user_prompt: None,
        }
    }
}

impl ChatDockState {
    #[must_use]
    pub fn assemble_preprompt(
        &self,
        profile_name: &str,
        estop: bool,
        lint: Option<&ToolpathLintReport>,
    ) -> String {
        let mut out = String::new();
        out.push_str("# MakerCraft Assistant Rules\n");
        out.push_str("- Answer from the bundled skills and live bench state below.\n");
        out.push_str("- Say clearly when you do not know.\n");
        out.push_str("- Never invent MakerCraft tool names, pinouts, or measurements.\n");
        out.push_str("- Remind the user about safety gates (no rapid plunges, scope ground-clip safety, E-stop) when relevant.\n\n");

        out.push_str("# Bundled Skills\n");
        for s in skills_list() {
            out.push_str("## Skill: ");
            out.push_str(s.name);
            out.push('\n');
            out.push_str(s.skill_md);
            out.push_str("\n\n");
        }

        out.push_str("# Live Bench Status\n");
        let status = dispatch(Command::Status);
        let lint_summary = match lint {
            Some(r) => format!(
                "passes_safety={}, segments={}, travel_mm={:.1}, est_min={:.1}, errors={}, warnings={}",
                r.passes_safety,
                r.segment_count,
                r.travel_distance_mm,
                r.estimated_minutes,
                r.errors.len(),
                r.warnings.len()
            ),
            None => "none".to_string(),
        };
        let state_json = json!({
            "active_profile": profile_name,
            "estop_latched": estop,
            "lint_summary": lint_summary,
            "hardware_status": status,
        });
        out.push_str(&state_json.to_string());
        out
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(res) = rx.try_recv() {
                self.busy = false;
                self.rx = None;
                match res {
                    Ok(reply) => self.messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        text: reply,
                        is_error: false,
                    }),
                    Err(err) => self.messages.push(ChatMessage {
                        role: "error".to_string(),
                        text: err,
                        is_error: true,
                    }),
                }
            }
        }
        if let Some(rx) = &self.scan_rx {
            if let Ok(res) = rx.try_recv() {
                self.scan_rx = None;
                match res {
                    Ok(models) => {
                        if self.model.trim().is_empty() {
                            if let Some(first) = models.first() {
                                self.model = first.clone();
                            }
                        }
                        self.discovered_models = models;
                    }
                    Err(err) => {
                        self.messages.push(ChatMessage {
                            role: "error".to_string(),
                            text: format!("Model scan failed: {err}"),
                            is_error: true,
                        });
                    }
                }
            }
        }
    }

    pub fn scan_local_models(&mut self) {
        let base = self.base_url.trim_end_matches('/').to_string();
        let provider = self.provider;
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);
        std::thread::spawn(move || {
            let res = fetch_models(provider, &base);
            let _ = tx.send(res);
        });
    }

    pub fn send_message(
        &mut self,
        user_text: String,
        profile_name: &str,
        estop: bool,
        lint: Option<&ToolpathLintReport>,
    ) {
        let trimmed = user_text.trim().to_string();
        if trimmed.is_empty() || self.busy {
            return;
        }
        if self.model.trim().is_empty() {
            self.show_settings = true;
            self.messages.push(ChatMessage {
                role: "error".to_string(),
                text: "No model selected. Open Settings below and pick or scan a local model (e.g. Ollama).".to_string(),
                is_error: true,
            });
            return;
        }
        self.last_user_prompt = Some(trimmed.clone());
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            text: trimmed,
            is_error: false,
        });
        self.input.clear();
        self.busy = true;

        let preprompt = self.assemble_preprompt(profile_name, estop, lint);
        let history: Vec<(String, String)> = self
            .messages
            .iter()
            .filter(|m| !m.is_error && (m.role == "user" || m.role == "assistant"))
            .map(|m| (m.role.clone(), m.text.clone()))
            .collect();
        let provider = self.provider;
        let base_url = self.base_url.trim_end_matches('/').to_string();
        let model = self.model.trim().to_string();
        let api_key = self.api_key.trim().to_string();

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        std::thread::spawn(move || {
            let res = call_llm(provider, &base_url, &model, &api_key, &preprompt, &history);
            let _ = tx.send(res);
        });
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        profile_name: &str,
        estop: bool,
        lint: Option<&ToolpathLintReport>,
    ) {
        ui.horizontal(|ui| {
            ui.strong("AI Bench Assistant (Ctrl+J)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("X").clicked() {
                    self.open = false;
                    self.maximized = false;
                }
                let max_lbl = if self.maximized {
                    "Restore"
                } else {
                    "Maximize"
                };
                if ui.small_button(max_lbl).clicked() {
                    self.maximized = !self.maximized;
                }
                if ui.small_button("Settings").clicked() {
                    self.show_settings = !self.show_settings;
                }
                if ui.small_button("View Prompt").clicked() {
                    self.show_prompt_preview = !self.show_prompt_preview;
                }
            });
        });

        let preprompt = self.assemble_preprompt(profile_name, estop, lint);
        let est_tokens = preprompt.len() / 4;
        ui.small(format!(
            "Provider: {} | Model: {} | Preprompt: ~{} tokens (16384 ctx)",
            self.provider.label(),
            if self.model.is_empty() {
                "<unset>"
            } else {
                &self.model
            },
            est_tokens
        ));
        ui.separator();

        if self.show_settings {
            ui.group(|ui| {
                ui.strong("LLM Provider Settings (Local-First)");
                ui.horizontal(|ui| {
                    for p in [
                        LlmProvider::Ollama,
                        LlmProvider::LmStudio,
                        LlmProvider::OpenAiCompat,
                    ] {
                        if ui.selectable_label(self.provider == p, p.label()).clicked() {
                            self.provider = p;
                            self.base_url = p.default_url().to_string();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut self.base_url);
                });
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.text_edit_singleline(&mut self.model);
                    if ui.button("Scan Local").clicked() {
                        self.scan_local_models();
                    }
                });
                if !self.discovered_models.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for m in &self.discovered_models {
                            if ui.small_button(m).clicked() {
                                self.model = m.clone();
                            }
                        }
                    });
                }
                if self.provider == LlmProvider::OpenAiCompat {
                    ui.horizontal(|ui| {
                        ui.label("API Key:");
                        ui.add(egui::TextEdit::singleline(&mut self.api_key).password(true));
                    });
                }
            });
            ui.separator();
        }

        if self.show_prompt_preview {
            ui.group(|ui| {
                ui.strong("Assembled Preprompt (Skills + Live Bench State)");
                egui::ScrollArea::vertical()
                    .id_salt("preprompt_preview_scroll")
                    .max_height(180.0)
                    .show(ui, |ui| {
                        ui.monospace(&preprompt);
                    });
            });
            ui.separator();
        }

        let avail_h = (ui.available_height() - 68.0).max(120.0);
        let mut retry_prompt: Option<String> = None;
        egui::ScrollArea::vertical()
            .id_salt("makercraft_chat_scroll")
            .max_height(avail_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &self.messages {
                    ui.group(|ui| {
                        if msg.is_error {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 120, 120),
                                format!("ERROR: {}", msg.text),
                            );
                            ui.horizontal(|ui| {
                                if ui.small_button("Open Settings").clicked() {
                                    self.show_settings = true;
                                }
                                if let Some(last) = self.last_user_prompt.clone() {
                                    if ui.small_button("Retry").clicked() {
                                        retry_prompt = Some(last);
                                    }
                                }
                            });
                        } else {
                            let tag = match msg.role.as_str() {
                                "user" => "You",
                                "assistant" => "MakerCraft AI",
                                _ => "System",
                            };
                            ui.strong(tag);
                            ui.label(&msg.text);
                        }
                    });
                }
                if self.busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Thinking...");
                    });
                }
            });
        if let Some(last) = retry_prompt {
            self.send_message(last, profile_name, estop, lint);
        }

        ui.separator();
        ui.horizontal(|ui| {
            let resp = ui.add_sized(
                egui::vec2((ui.available_width() - 64.0).max(120.0), 44.0),
                egui::TextEdit::multiline(&mut self.input)
                    .hint_text("Ask about G-code, Klipper, GRBL, scopes, Vienna makerspaces, or buying gear..."),
            );
            let send_pressed = ui.button("Send").clicked()
                || (resp.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl));
            if send_pressed {
                let text = self.input.clone();
                self.send_message(text, profile_name, estop, lint);
            }
        });
    }
}

fn fetch_models(provider: LlmProvider, base_url: &str) -> Result<Vec<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    match provider {
        LlmProvider::Ollama => {
            let url = format!("{base_url}/api/tags");
            let resp = client
                .get(&url)
                .send()
                .map_err(|e| format!("GET {url}: {e}"))?;
            let val: serde_json::Value = resp.json().map_err(|e| format!("parse tags: {e}"))?;
            let names = val
                .get("models")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(names)
        }
        LlmProvider::LmStudio | LlmProvider::OpenAiCompat => {
            let url = format!("{base_url}/v1/models");
            let resp = client
                .get(&url)
                .send()
                .map_err(|e| format!("GET {url}: {e}"))?;
            let val: serde_json::Value = resp.json().map_err(|e| format!("parse models: {e}"))?;
            let names = val
                .get("data")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.get("id").and_then(|n| n.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(names)
        }
    }
}

fn call_llm(
    provider: LlmProvider,
    base_url: &str,
    model: &str,
    api_key: &str,
    preprompt: &str,
    history: &[(String, String)],
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let mut msgs = vec![json!({"role": "system", "content": preprompt})];
    for (role, content) in history {
        msgs.push(json!({"role": role, "content": content}));
    }

    match provider {
        LlmProvider::Ollama => {
            let url = format!("{base_url}/api/chat");
            let body = json!({
                "model": model,
                "messages": msgs,
                "stream": false,
                "think": false,
                "options": { "num_ctx": 16384 }
            });
            let resp = client
                .post(&url)
                .json(&body)
                .send()
                .map_err(|e| format!("Ollama unreachable at {url}: {e}"))?;
            if !resp.status().is_success() {
                return Err(format!("Ollama HTTP {}", resp.status()));
            }
            let val: serde_json::Value = resp.json().map_err(|e| format!("invalid JSON: {e}"))?;
            let content = val
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if content.is_empty() {
                Err("Ollama returned an empty reply.".to_string())
            } else {
                Ok(content)
            }
        }
        LlmProvider::LmStudio | LlmProvider::OpenAiCompat => {
            let url = format!("{base_url}/v1/chat/completions");
            let body = json!({
                "model": model,
                "messages": msgs,
                "stream": false,
            });
            let mut req = client.post(&url).json(&body);
            if !api_key.is_empty() {
                req = req.bearer_auth(api_key);
            }
            let resp = req.send().map_err(|e| format!("POST {url}: {e}"))?;
            if !resp.status().is_success() {
                return Err(format!("HTTP {}", resp.status()));
            }
            let val: serde_json::Value = resp.json().map_err(|e| format!("invalid JSON: {e}"))?;
            let content = val
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|a| a.first())
                .and_then(|x| x.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if content.is_empty() {
                Err("Endpoint returned an empty reply.".to_string())
            } else {
                Ok(content)
            }
        }
    }
}
