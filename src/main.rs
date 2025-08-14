mod gui;
use eframe::egui::ViewportBuilder;
mod distributions;
mod mcs;
mod message;
use eframe::run_native;
use gui::MyEguiApp;
mod errors;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_fullscreen(true),

        ..Default::default()
    };
    let _ = run_native(
        "SANDHIYA",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    );
}
