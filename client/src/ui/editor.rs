use crate::network::ClientMessage;
use crate::ui::generation_status::GenerationStatusPanel;
use crate::ui::prompt_panel::{MuseMode, PromptPanel};
use crate::ui::style::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use egui::{
    Align, CentralPanel, Color32, Context, FontId, Frame, Layout, Margin, RichText,
    ScrollArea, SidePanel, Stroke,
};

#[derive(Clone)]
pub enum LogLevel {
    Ok,
    Info,
    Warn,
    Err,
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Default)]
pub struct PerformanceMetrics {
    pub fps: f32,
}

#[derive(PartialEq)]
enum RightTab {
    Console,
}

pub struct EditorState {
    pub logs: Vec<LogEntry>,
    pub ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>,
    pub metrics: PerformanceMetrics,
    pub prompt_panel: PromptPanel,
    pub gen_status: GenerationStatusPanel,
    pub loaded_character: bool,
    pub loaded_motion: bool,
    pub character_mesh: Option<serde_json::Value>,
    pub character_skeleton: Option<serde_json::Value>,
    pub motion_clip: Option<serde_json::Value>,
}

impl EditorState {
    pub fn new(ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>) -> Self {
        Self {
            logs: vec![LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: LogLevel::Info,
                message: String::from("Muse initialized"),
            }],
            ws_tx,
            metrics: PerformanceMetrics::default(),
            prompt_panel: PromptPanel::new(),
            gen_status: GenerationStatusPanel::new(),
            loaded_character: false,
            loaded_motion: false,
            character_mesh: None,
            character_skeleton: None,
            motion_clip: None,
        }
    }

    fn setup_style(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = BG_PANEL;
        style.visuals.panel_fill = BG_SIDEBAR;
        style.visuals.extreme_bg_color = BG_CANVAS;
        style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
        style.visuals.widgets.inactive.bg_fill = BG_CARD;
        style.visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
        style.visuals.widgets.active.bg_fill = ACCENT_STRONG;
        style.visuals.override_text_color = Some(TEXT);
        ctx.set_style(style);
    }

    pub fn draw(&mut self, ctx: &Context) {
        Self::setup_style(ctx);

        egui::TopBottomPanel::top("top_bar")
            .frame(Frame::none().fill(BG_CARD).inner_margin(Margin::symmetric(16.0, 0.0)))
            .height_range(44.0..=44.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label(RichText::new("Muse").strong().size(14.0).color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let fps_color = if self.metrics.fps > 30.0 { GREEN } else if self.metrics.fps > 15.0 { YELLOW } else { RED };
                        ui.label(RichText::new(format!("{:.0} FPS", self.metrics.fps)).font(FontId::monospace(11.0)).color(fps_color));
                    });
                });
            });

        SidePanel::right("right_panel")
            .frame(Frame::none().fill(BG_SIDEBAR).inner_margin(Margin::ZERO))
            .resizable(true)
            .default_width(280.0)
            .min_width(180.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    ui.style_mut().spacing.item_spacing.y = 0.0;
                    for log in &self.logs {
                        let (tag, color, bg) = match log.level {
                            LogLevel::Ok => ("OK", GREEN, GREEN.gamma_multiply(0.08)),
                            LogLevel::Info => ("INFO", ACCENT, ACCENT.gamma_multiply(0.08)),
                            LogLevel::Warn => ("WARN", YELLOW, YELLOW.gamma_multiply(0.08)),
                            LogLevel::Err => ("ERR", RED, RED.gamma_multiply(0.08)),
                        };
                        Frame::none()
                            .fill(bg)
                            .inner_margin(Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&log.timestamp).font(FontId::monospace(9.0)).color(TEXT_MUTED));
                                    ui.label(RichText::new(tag).font(FontId::monospace(9.0)).color(color));
                                    ui.label(RichText::new(&log.message).font(FontId::monospace(11.0)).color(TEXT));
                                });
                            });
                    }
                });
            });

        CentralPanel::default()
            .frame(Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |_ui| {});

        self.prompt_panel.draw(ctx);
        if self.prompt_panel.take_generate() {
            self.send_generate_request();
        }
        self.gen_status.draw(ctx);
    }

    fn send_generate_request(&mut self) {
        let job_type = match self.prompt_panel.mode {
            MuseMode::TextToCharacter => "text_to_mesh",
            MuseMode::TextToMotion => "text_to_motion",
            MuseMode::PoseStaging => "pose_interpolation",
            MuseMode::StyleTransfer => "style_transfer",
            MuseMode::Retarget => "retarget",
        };
        let mut params = serde_json::json!({
            "prompt": self.prompt_panel.prompt,
        });
        if let Some(seed) = self.prompt_panel.seed {
            params["seed"] = serde_json::json!(seed);
        }
        if self.prompt_panel.mode == MuseMode::StyleTransfer {
            params["style_prompt"] = serde_json::json!(self.prompt_panel.style_prompt);
        }
        let msg = ClientMessage {
            job_request: Some(crate::network::JobRequest {
                job_type: job_type.to_string(),
                params,
            }),
        };
        let serialized = serde_json::to_string(&msg).expect("Failed to serialize ClientMessage");
        let guard = self.ws_tx.lock().expect("ws_tx lock poisoned");
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(serialized);
        }
        self.push_log(LogLevel::Info, &format!("Sent generate request: {}", job_type));
    }

    pub fn push_log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            message: msg.to_string(),
        });
        if self.logs.len() > 200 {
            self.logs.remove(0);
        }
    }
}
