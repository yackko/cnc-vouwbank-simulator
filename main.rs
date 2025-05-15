mod app;
mod state;
mod ui;
mod logic;
mod db;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Ensure the assets directory is correctly located relative to the executable
    // This might be important for release builds if assets are not embedded.
    // For development with `cargo run`, it usually finds `assets/` in the project root.
    println!("Current working directory: {:?}", std::env::current_dir().unwrap_or_default());
    println!("Attempting to load image from: assets/drawing.png");


    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vouwbank Simulator",
        options,
        Box::new(|cc| Box::new(app::MyApp::new(cc))),
    )
}
