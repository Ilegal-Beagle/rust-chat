mod network;
mod chat;
mod message;
mod tenor;

use crate::chat::App;
use tokio;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 400.0]),
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().expect("could not create runtime");
    let rt_handle = rt.handle().clone();

    eframe::run_native(
        "Rust Chat",
        options.clone(),
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(rt_handle)))
        }),
    )
}