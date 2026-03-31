//! GUI application for Reality Simulator
//!
//! Run with: cargo run --bin reality-sim-gui

mod app;
mod panels;
mod renderer;

use app::SimulatorApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Reality Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Reality Simulator",
        native_options,
        Box::new(|cc| Ok(Box::new(SimulatorApp::new(cc)))),
    )
}
