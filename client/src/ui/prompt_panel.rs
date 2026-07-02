use egui::{Color32, FontId, RichText, Stroke, TextEdit};

use super::style::*;

#[derive(Debug, PartialEq)]
pub enum MuseMode {
    TextToCharacter,
    TextToMotion,
    PoseStaging,
    StyleTransfer,
    Retarget,
}

pub struct PromptPanel {
    pub visible: bool,
    pub mode: MuseMode,
    pub prompt: String,
    pub style_prompt: String,
    pub seed: Option<u64>,
    pub generate_clicked: bool,
}

impl PromptPanel {
    pub fn new() -> Self {
        Self {
            visible: true,
            mode: MuseMode::TextToCharacter,
            prompt: String::new(),
            style_prompt: String::new(),
            seed: None,
            generate_clicked: false,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new("Muse Generator")
            .id(egui::Id::new("muse_prompt"))
            .default_width(320.0)
            .collapsible(true)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mode:").font(FontId::monospace(10.0)).color(TEXT_MUTED));
                    egui::ComboBox::from_id_source("mode")
                        .selected_text(format!("{:?}", self.mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.mode, MuseMode::TextToCharacter, "Text → Character");
                            ui.selectable_value(&mut self.mode, MuseMode::TextToMotion, "Text → Motion");
                            ui.selectable_value(&mut self.mode, MuseMode::PoseStaging, "Pose Staging");
                            ui.selectable_value(&mut self.mode, MuseMode::StyleTransfer, "Style Transfer");
                            ui.selectable_value(&mut self.mode, MuseMode::Retarget, "Retarget");
                        });
                });

                ui.add_space(8.0);

                match self.mode {
                    MuseMode::TextToCharacter | MuseMode::TextToMotion => {
                        ui.label(RichText::new("Prompt:").font(FontId::monospace(10.0)).color(TEXT));
                        let resp = TextEdit::multiline(&mut self.prompt)
                            .font(FontId::monospace(12.0))
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .hint_text("Describe what you want...")
                            .show(ui);
                        if resp.response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl)
                        {
                            self.generate_clicked = true;
                        }
                    }
                    MuseMode::StyleTransfer => {
                        ui.label(RichText::new("Source style prompt:").font(FontId::monospace(10.0)).color(TEXT));
                        TextEdit::multiline(&mut self.style_prompt)
                            .font(FontId::monospace(12.0))
                            .desired_rows(1)
                            .desired_width(f32::INFINITY)
                            .hint_text("Style to apply...")
                            .show(ui);
                        ui.add_space(4.0);
                        ui.label(RichText::new("Content prompt:").font(FontId::monospace(10.0)).color(TEXT));
                        TextEdit::multiline(&mut self.prompt)
                            .font(FontId::monospace(12.0))
                            .desired_rows(1)
                            .desired_width(f32::INFINITY)
                            .hint_text("Animation to restyle...")
                            .show(ui);
                    }
                    MuseMode::PoseStaging => {
                        ui.label("Set Pose A and Pose B in the viewport gizmos, then request in-betweening.");
                    }
                    MuseMode::Retarget => {
                        ui.label("Drop an external animation file (BVH/FBX/GLB) to retarget.");
                    }
                }

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Seed:").font(FontId::monospace(10.0)).color(TEXT_MUTED));
                    let mut seed_str = self.seed.map(|s| s.to_string()).unwrap_or_default();
                    if TextEdit::singleline(&mut seed_str)
                        .font(FontId::monospace(10.0))
                        .desired_width(80.0)
                        .hint_text("optional")
                        .show(ui)
                        .response
                        .changed()
                    {
                        self.seed = seed_str.parse::<u64>().ok();
                    }
                });

                ui.add_space(8.0);

                if ui
                    .add_sized(
                        [ui.available_width(), 32.0],
                        egui::Button::new(
                            RichText::new("Generate")
                                .font(FontId::monospace(12.0))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(ACCENT_STRONG)
                        .stroke(Stroke::new(0.5, ACCENT_STRONG)),
                    )
                    .clicked()
                {
                    self.generate_clicked = true;
                }
            });
    }

    pub fn take_generate(&mut self) -> bool {
        let val = self.generate_clicked;
        self.generate_clicked = false;
        val
    }
}
