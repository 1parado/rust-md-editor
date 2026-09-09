//! rust-md-editor — desktop Markdown editor (egui)
mod preview;
mod app;

use app::MdEditorApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title("rust-md-editor"),
        ..Default::default()
    };

    let path = std::env::args().nth(1).map(std::path::PathBuf::from);

    eframe::run_native(
        "rust-md-editor",
        native_options,
        Box::new(move |cc| Ok(Box::new(MdEditorApp::new(cc, path)))),
    )
}
