use egui::{
    vec2, Align, CentralPanel, Color32, FontId, Frame, Layout, Margin, RichText, ScrollArea,
    SidePanel, Stroke, TextEdit, Ui,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

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
    pub compile_time_ms: u32,
    pub complexity: u32,
    pub march_steps: u32,
    pub entities: usize,
    pub tick_ms: u32,
}

pub struct Template {
    pub name: &'static str,
    pub equation: &'static str,
    pub tag: &'static str,
}

#[derive(PartialEq)]
pub enum RightTab {
    Logs,
    Performance,
    Uniforms,
}

pub struct EditorState {
    pub equation: String,
    pub logs: Vec<LogEntry>,
    pub ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>,
    pub templates: Vec<Template>,
    pub active_tab: RightTab,
    pub metrics: PerformanceMetrics,
    pub uniforms: Vec<(String, String)>,
    pub error_msg: Option<String>,
}

const BG_CANVAS: Color32 = Color32::from_rgb(8, 10, 14);
const BG_SHELL: Color32 = Color32::from_rgb(13, 15, 20);
const BG_SIDEBAR: Color32 = Color32::from_rgb(15, 17, 23);
const BG_CARD: Color32 = Color32::from_rgb(18, 21, 28);
const BORDER_COLOR: Color32 = Color32::from_rgb(30, 33, 48);

const ACCENT_VIOLET: Color32 = Color32::from_rgb(127, 119, 221);
const ACCENT_VIOLET_STRONG: Color32 = Color32::from_rgb(83, 74, 183);
const ACCENT_VIOLET_FILL: Color32 = Color32::from_rgb(26, 22, 48);
const ACCENT_VIOLET_TEXT: Color32 = Color32::from_rgb(196, 191, 255);

const COLOR_SUCCESS: Color32 = Color32::from_rgb(29, 158, 117);
const COLOR_WARNING: Color32 = Color32::from_rgb(239, 159, 39);
const COLOR_ERROR: Color32 = Color32::from_rgb(226, 75, 74);

impl EditorState {
    pub fn new(ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>) -> Self {
        Self {
            equation: String::from("sqrt(x**2 + y**2 + z**2) - 10.0"),
            logs: vec![LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: LogLevel::Info,
                message: String::from("Rationalist terminal initialized"),
            }],
            ws_tx,
            active_tab: RightTab::Logs,
            metrics: PerformanceMetrics::default(),
            uniforms: Vec::new(),
            error_msg: None,
            templates: vec![
                Template {
                    name: "Torus knot",
                    equation: "sin(3*phi) * (2+cos(2*phi))",
                    tag: "geometry",
                },
                Template {
                    name: "Sphere hole",
                    equation: "Max(sqrt(x**2 + y**2 + z**2) - 10.0, -(sqrt(x**2 + y**2) - 4.0))",
                    tag: "geometry",
                },
                Template {
                    name: "Gyroid",
                    equation: "sin(x)*cos(y) + sin(y)*cos(z) + sin(z)*cos(x)",
                    tag: "advanced",
                },
                Template {
                    name: "Orbital Tracking",
                    equation: "sqrt((x - state.x)**2 + (y - state.y)**2 + (z - state.z)**2) - 5.0",
                    tag: "physics",
                },
                Template {
                    name: "Mandelbulb",
                    equation: "pow(length(p), 8.0) - 1.0",
                    tag: "advanced",
                },
                Template {
                    name: "Infinite Cylinder",
                    equation: "sqrt(x**2 + y**2) - 5.0",
                    tag: "geometry",
                },
            ],
        }
    }

    fn setup_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = BG_SHELL;
        style.visuals.panel_fill = BG_SIDEBAR;
        style.visuals.extreme_bg_color = BG_CANVAS;
        style.visuals.widgets.noninteractive.bg_fill = BG_SHELL;
        style.visuals.widgets.noninteractive.fg_stroke =
            Stroke::new(1.0, Color32::from_rgb(138, 143, 168));
        style.visuals.widgets.inactive.bg_fill = BG_CARD;
        style.visuals.widgets.hovered.bg_fill = ACCENT_VIOLET_FILL;
        style.visuals.widgets.active.bg_fill = ACCENT_VIOLET_STRONG;
        style.visuals.override_text_color = Some(Color32::from_rgb(138, 143, 168));
        ctx.set_style(style);
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        Self::setup_style(ctx);

        egui::TopBottomPanel::top("top_bar")
            .frame(
                Frame::none()
                    .fill(BG_CARD)
                    .inner_margin(Margin::symmetric(16.0, 0.0)),
            )
            .height_range(44.0..=44.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label(
                        RichText::new("Ration")
                            .strong()
                            .size(14.0)
                            .color(Color32::WHITE),
                    );
                    ui.label(RichText::new("Δ").strong().size(16.0).color(ACCENT_VIOLET));
                    ui.label(
                        RichText::new("list")
                            .strong()
                            .size(14.0)
                            .color(Color32::WHITE),
                    );

                    ui.add_space(32.0);

                    let mut nav_btn = |label: &str, active: bool| {
                        let color = if active {
                            ACCENT_VIOLET_TEXT
                        } else {
                            Color32::from_rgb(138, 143, 168)
                        };
                        let fill = if active {
                            BORDER_COLOR
                        } else {
                            Color32::TRANSPARENT
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(label)
                                        .font(FontId::monospace(11.0))
                                        .color(color),
                                )
                                .fill(fill),
                            )
                            .clicked()
                        {
                            // change view
                        }
                    };
                    nav_btn("editor", true);
                    nav_btn("scenes", false);
                    nav_btn("physics", false);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{:.1} FPS", self.metrics.fps))
                                .font(FontId::monospace(10.0)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("shader live")
                                .font(FontId::monospace(10.0))
                                .color(COLOR_SUCCESS),
                        );
                        ui.label(RichText::new("●").size(8.0).color(COLOR_SUCCESS));
                    });
                });
            });

        SidePanel::left("left_sidebar")
            .frame(
                Frame::none()
                    .fill(BG_SIDEBAR)
                    .inner_margin(Margin::same(16.0)),
            )
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("PROJECT WORKSPACE")
                        .font(FontId::monospace(10.0))
                        .color(Color32::from_rgb(69, 72, 96)),
                );
                ui.label(
                    RichText::new("v0.4.2-stable")
                        .font(FontId::monospace(10.0))
                        .color(Color32::from_rgb(69, 72, 96)),
                );

                ui.add_space(24.0);
                ui.label(
                    RichText::new("TEMPLATES")
                        .font(FontId::monospace(10.0))
                        .color(Color32::from_rgb(69, 72, 96)),
                );
                ui.add_space(8.0);

                ScrollArea::vertical().show(ui, |ui| {
                    for t in &self.templates {
                        let active = self.equation == t.equation;
                        let frame = Frame::none()
                            .fill(if active { ACCENT_VIOLET_FILL } else { BG_CARD })
                            .stroke(Stroke::new(0.5, BORDER_COLOR))
                            .rounding(6.0)
                            .inner_margin(Margin::same(10.0));

                        let response = frame
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(t.name)
                                                .font(FontId::monospace(12.0))
                                                .color(ACCENT_VIOLET_TEXT),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                let (tag_bg, tag_fg) = match t.tag {
                                                    "geometry" => (
                                                        Color32::from_rgb(14, 31, 26),
                                                        COLOR_SUCCESS,
                                                    ),
                                                    "physics" => (
                                                        Color32::from_rgb(26, 18, 32),
                                                        Color32::from_rgb(212, 83, 126),
                                                    ),
                                                    _ => (
                                                        Color32::from_rgb(31, 21, 0),
                                                        COLOR_WARNING,
                                                    ),
                                                };
                                                ui.label(
                                                    RichText::new(t.tag)
                                                        .font(FontId::monospace(8.0))
                                                        .color(tag_fg)
                                                        .background_color(tag_bg),
                                                );
                                            },
                                        );
                                    });
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(t.equation)
                                            .font(FontId::monospace(10.0))
                                            .color(ACCENT_VIOLET_STRONG),
                                    );
                                });
                            })
                            .response;

                        if response.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                        if response.clicked() {
                            self.equation = t.equation.to_string();
                        }
                        ui.add_space(8.0);
                    }
                });
            });

        SidePanel::right("right_panel")
            .frame(Frame::none().fill(BG_SIDEBAR).inner_margin(Margin::ZERO))
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 0.0;
                    let tab_btn = |ui: &mut Ui, label: &str, tab: RightTab, current: &RightTab| {
                        let active = tab == *current;
                        let color = if active {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(138, 143, 168)
                        };
                        let btn = ui.add_sized(
                            [ui.available_width() / 3.0, 44.0],
                            egui::Button::new(
                                RichText::new(label)
                                    .font(FontId::monospace(10.0))
                                    .color(color),
                            )
                            .fill(Color32::TRANSPARENT),
                        );
                        if active {
                            let rect = btn.rect;
                            ui.painter().line_segment(
                                [rect.left_bottom(), rect.right_bottom()],
                                Stroke::new(1.5, ACCENT_VIOLET),
                            );
                        }
                        if btn.clicked() {
                            return Some(tab);
                        }
                        None
                    };

                    if let Some(t) = tab_btn(ui, "LOGS", RightTab::Logs, &self.active_tab) {
                        self.active_tab = t;
                    }
                    if let Some(t) =
                        tab_btn(ui, "PERFORMANCE", RightTab::Performance, &self.active_tab)
                    {
                        self.active_tab = t;
                    }
                    if let Some(t) = tab_btn(ui, "UNIFORMS", RightTab::Uniforms, &self.active_tab) {
                        self.active_tab = t;
                    }
                });

                ui.add_space(16.0);

                match self.active_tab {
                    RightTab::Logs => {
                        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            for log in &self.logs {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&log.timestamp)
                                            .font(FontId::monospace(10.0))
                                            .color(Color32::from_rgb(69, 72, 96)),
                                    );
                                    let (tag, color) = match log.level {
                                        LogLevel::Ok => ("OK", COLOR_SUCCESS),
                                        LogLevel::Info => ("INFO", ACCENT_VIOLET),
                                        LogLevel::Warn => ("WARN", COLOR_WARNING),
                                        LogLevel::Err => ("ERR", COLOR_ERROR),
                                    };
                                    ui.label(
                                        RichText::new(tag)
                                            .font(FontId::monospace(9.0))
                                            .color(color)
                                            .background_color(color.gamma_multiply(0.1)),
                                    );
                                    ui.label(
                                        RichText::new(&log.message)
                                            .font(FontId::monospace(11.0))
                                            .color(Color32::from_rgb(138, 143, 168)),
                                    );
                                });
                                ui.add_space(4.0);
                                ui.painter().hline(
                                    ui.available_rect_before_wrap().x_range(),
                                    ui.cursor().min.y,
                                    Stroke::new(0.5, Color32::from_rgb(18, 21, 28)),
                                );
                                ui.add_space(4.0);
                            }
                        });
                    }
                    RightTab::Performance => {
                        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                        egui::Grid::new("perf_grid")
                            .num_columns(2)
                            .spacing(vec2(8.0, 8.0))
                            .show(ui, |ui| {
                                let mut metric =
                                    |ui: &mut Ui, val: String, label: &str, color: Color32| {
                                        Frame::none()
                                            .fill(BG_SHELL)
                                            .stroke(Stroke::new(0.5, BORDER_COLOR))
                                            .rounding(6.0)
                                            .inner_margin(Margin::symmetric(12.0, 10.0))
                                            .show(ui, |ui| {
                                                ui.vertical(|ui| {
                                                    ui.label(
                                                        RichText::new(val)
                                                            .font(FontId::monospace(20.0))
                                                            .color(color),
                                                    );
                                                    ui.label(
                                                        RichText::new(label)
                                                            .font(FontId::monospace(10.0))
                                                            .color(Color32::from_rgb(69, 72, 96)),
                                                    );
                                                });
                                            });
                                    };
                                metric(
                                    ui,
                                    format!("{:.1}", self.metrics.fps),
                                    "fps",
                                    COLOR_SUCCESS,
                                );
                                metric(
                                    ui,
                                    format!("{}ms", self.metrics.compile_time_ms),
                                    "compile",
                                    Color32::WHITE,
                                );
                                ui.end_row();
                                metric(
                                    ui,
                                    format!("{}", self.metrics.complexity),
                                    "complexity",
                                    COLOR_WARNING,
                                );
                                metric(
                                    ui,
                                    format!("{}", self.metrics.march_steps),
                                    "march steps",
                                    Color32::WHITE,
                                );
                                ui.end_row();
                                metric(
                                    ui,
                                    format!("{}", self.metrics.entities),
                                    "entities",
                                    COLOR_SUCCESS,
                                );
                                metric(
                                    ui,
                                    format!("{}ms", self.metrics.tick_ms),
                                    "physics tick",
                                    Color32::WHITE,
                                );
                                ui.end_row();
                            });
                    }
                    RightTab::Uniforms => {
                        ScrollArea::vertical().show(ui, |ui| {
                            for (key, val) in &self.uniforms {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(key)
                                            .font(FontId::monospace(11.0))
                                            .color(Color32::from_rgb(107, 112, 128)),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(val)
                                                .font(FontId::monospace(11.0))
                                                .color(ACCENT_VIOLET),
                                        );
                                    });
                                });
                                ui.add_space(4.0);
                                ui.painter().hline(
                                    ui.available_rect_before_wrap().x_range(),
                                    ui.cursor().min.y,
                                    Stroke::new(0.5, Color32::from_rgb(18, 21, 28)),
                                );
                                ui.add_space(4.0);
                            }
                        });
                    }
                }
            });

        CentralPanel::default()
            .frame(Frame::none().fill(BG_CANVAS))
            .show(ctx, |ui| {
                // Visualizer Overlay
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("sdf(p) → raymarcher → 1920×1080")
                            .font(FontId::monospace(10.0))
                            .color(Color32::from_rgb(69, 72, 96)),
                    );
                });

                // Visualizer Overlay
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        let mut badge = |ui: &mut Ui, text: &str, color: Color32| {
                            Frame::none()
                                .fill(BG_SIDEBAR)
                                .stroke(Stroke::new(0.5, BORDER_COLOR))
                                .inner_margin(Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(text)
                                            .font(FontId::monospace(10.0))
                                            .color(color),
                                    );
                                });
                        };
                        badge(ui, "● live", COLOR_SUCCESS);
                        badge(
                            ui,
                            &format!("{} steps", self.metrics.march_steps),
                            Color32::WHITE,
                        );
                        badge(ui, "WGSL 1.0", Color32::WHITE);
                        ui.add_space(16.0);
                    });
                });

                // Visualizer Overlay - Bottom Left
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.add_space(140.0); // Space for equation panel
                    ui.label(
                        RichText::new(format!("entities: {} · fields: 2", self.metrics.entities))
                            .font(FontId::monospace(10.0))
                            .color(Color32::from_rgb(69, 72, 96)),
                    );
                    ui.add_space(16.0);
                });

                // Visualizer Overlay - Bottom Right
                ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                    ui.add_space(140.0);
                    ui.horizontal(|ui| {
                        let mut cam_btn = |ui: &mut Ui, icon: &str| {
                            ui.add_sized(
                                [28.0, 28.0],
                                egui::Button::new(RichText::new(icon).size(12.0))
                                    .fill(BG_SIDEBAR)
                                    .stroke(Stroke::new(0.5, BORDER_COLOR)),
                            );
                        };
                        cam_btn(ui, "⬈");
                        cam_btn(ui, "🔍");
                        cam_btn(ui, "⟲");
                        ui.add_space(16.0);
                    });
                });

                // Equation Panel at the bottom
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    Frame::none()
                        .fill(BG_SHELL)
                        .stroke(Stroke::new(0.5, BORDER_COLOR))
                        .inner_margin(Margin::symmetric(16.0, 14.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("SDF EQUATION — map(p: vec3)")
                                        .font(FontId::monospace(10.0))
                                        .color(Color32::from_rgb(69, 72, 96)),
                                );
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("f(x,y,z) =")
                                            .font(FontId::monospace(13.0))
                                            .color(ACCENT_VIOLET_STRONG),
                                    );

                                    let mut edit = TextEdit::multiline(&mut self.equation)
                                        .font(FontId::monospace(13.0))
                                        .desired_rows(2)
                                        .desired_width(f32::INFINITY);

                                    if self.error_msg.is_some() {}

                                    ui.add(edit);

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui
                                            .add_sized(
                                                [80.0, 56.0],
                                                egui::Button::new(
                                                    RichText::new("COMPILE")
                                                        .strong()
                                                        .color(ACCENT_VIOLET_TEXT),
                                                )
                                                .fill(ACCENT_VIOLET_FILL)
                                                .stroke(Stroke::new(0.5, ACCENT_VIOLET_STRONG)),
                                            )
                                            .clicked()
                                        {
                                            self.compile_and_send();
                                        }
                                    });
                                });

                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    let chips = [
                                        "length(p)",
                                        "sqrt(x**2+z**2)",
                                        "sin(x)*cos(y)",
                                        "max(a,b)",
                                        "min(a,b)",
                                        "abs(p)",
                                        "state.x",
                                    ];
                                    for chip in chips {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(chip)
                                                        .font(FontId::monospace(10.0))
                                                        .color(ACCENT_VIOLET_STRONG),
                                                )
                                                .fill(BG_SIDEBAR)
                                                .stroke(Stroke::new(0.5, BORDER_COLOR)),
                                            )
                                            .clicked()
                                        {
                                            self.equation.push_str(chip);
                                        }
                                    }
                                });

                                if let Some(err) = &self.error_msg {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(err)
                                            .font(FontId::monospace(11.0))
                                            .color(COLOR_ERROR),
                                    );
                                }
                            });
                        });
                });
            });
    }

    fn compile_and_send(&mut self) {
        let payload = serde_json::json!({ "equation": self.equation.trim() }).to_string();
        let tx = {
            let guard = self.ws_tx.lock().unwrap();
            guard.clone()
        };

        if let Some(tx) = tx {
            match tx.send(payload) {
                Ok(_) => self.push_log(LogLevel::Ok, "structure broadcasted"),
                Err(e) => self.push_log(LogLevel::Err, &format!("uplink failed: {e}")),
            }
        } else {
            self.push_log(LogLevel::Warn, "relay disconnected");
        }
    }

    pub fn push_log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            message: msg.to_string(),
        });
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}
