//! rust-md-editor — desktop Markdown editor (egui)
mod app;
mod fonts;
mod preview;

use app::MdEditorApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([760.0, 500.0])
            .with_title("rust-md-editor")
            .with_transparent(false),
        ..Default::default()
    };

    let path = std::env::args().nth(1).map(std::path::PathBuf::from);

    eframe::run_native(
        "rust-md-editor",
        native_options,
        Box::new(move |cc| {
            fonts::install_cjk_fonts(&cc.egui_ctx);
            Ok(Box::new(MdEditorApp::new(cc, path)))
        }),
    )
}
