mod network;
mod chat;
mod message;

use crate::chat::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Chat",
        options.clone(),
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}