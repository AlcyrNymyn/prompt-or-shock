pub mod app;
pub mod log_file_debounce_handler;
pub mod log_file_reader;
pub mod pishock_client;

use app::PromptOrShockApp;
use eframe::{App, egui};

#[tokio::main]
async fn main() {
    let mut app = PromptOrShockApp::default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_simple_native("Prompt or Shock", options, move |ctx, frame| {
        app.update(ctx, frame);
    })
    .expect("???");
}
