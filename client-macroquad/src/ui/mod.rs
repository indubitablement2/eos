use egui::Widget;

#[derive(Debug, Default)]
pub struct UiState {
    frame_time: bool,
}
impl UiState {
    pub fn init() -> Self {
        egui_macroquad::cfg(|egui_ctx| {
            egui_ctx.set_style(egui::Style {
                override_font_id: None,
                override_text_style: None,
                text_styles: egui::style::default_text_styles(),
                wrap: None,
                spacing: egui::style::Spacing::default(),
                interaction: egui::style::Interaction::default(),
                visuals: egui::style::Visuals::default(),
                animation_time: 1.0 / 12.0,
                debug: Default::default(),
                explanation_tooltips: false,
            });
        });

        Self { frame_time: false }
    }

    pub fn draw(&mut self, egui_ctx: &egui::Context) {
        egui::Window::new("❤")
            .resizable(false)
            .title_bar(false)
            .show(egui_ctx, |ui| {
                let (text, tooltip) = if self.frame_time {
                    (
                        format!("{:.04}", macroquad::prelude::get_frame_time()),
                        "Frame time",
                    )
                } else {
                    (format!("{:.01}", macroquad::prelude::get_fps()), "Fps")
                };
                let label = egui::Label::new(text)
                    .sense(egui::Sense::click_and_drag())
                    .ui(ui)
                    .on_hover_text(tooltip);

                if label.clicked() {
                    self.frame_time = !self.frame_time;
                }
            });
    }
}
