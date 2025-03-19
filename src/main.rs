use eframe::egui;
// 移除未使用的导入
// use std::path::PathBuf;

mod app;
mod editor;
mod syntax;
mod theme;
mod ui;
mod config;

fn main() -> Result<(), eframe::Error> {
    // initialize logger
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Notion++",
        options,
        Box::new(|cc| Box::new(app::NotionApp::new(cc)))
    )
} 