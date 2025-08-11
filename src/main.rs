mod gui;
use eframe::egui::ViewportBuilder;
mod distributions;
mod mcs;
use eframe::run_native;
use gui::MyEguiApp;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };
    let _ = run_native(
        "SANDHYA",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    );
}
