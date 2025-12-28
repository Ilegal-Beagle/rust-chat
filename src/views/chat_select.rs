use crate::App;
use egui;

impl App {
    pub fn render_select(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Rooms");
        });
    }
}