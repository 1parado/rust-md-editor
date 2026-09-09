//! Chinese / CJK font setup so glyphs render (not tofu □).
use eframe::egui;

/// Install system Chinese fonts into egui.
pub fn install_cjk_fonts(ctx: &egui::Context) {
    match egui_chinese_font::setup_chinese_fonts(ctx) {
        Ok(()) => {
            eprintln!("CJK fonts: egui-chinese-font OK");
            return;
        }
        Err(e) => {
            eprintln!("egui-chinese-font failed: {e:?}; trying file fallback");
        }
    }
    install_cjk_fonts_from_files(ctx);
}

fn install_cjk_fonts_from_files(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    for (path, index) in cjk_candidates() {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        if data.len() < 1024 {
            continue;
        }
        let mut fd = egui::FontData::from_owned(data);
        fd.index = index;
        fonts.font_data.insert("cjk".to_owned(), fd);
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "cjk".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("cjk".to_owned());
        ctx.set_fonts(fonts);
        eprintln!("CJK fonts: fallback {} (index {index})", path.display());
        return;
    }
    eprintln!("warning: no CJK font found — Chinese will show as □");
}

fn cjk_candidates() -> Vec<(std::path::PathBuf, u32)> {
    let mut v = Vec::new();
    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        for (rel, idx) in [
            (r"Fonts\msyh.ttf", 0u32),
            (r"Fonts\simhei.ttf", 0),
            (r"Fonts\simkai.ttf", 0),
            (r"Fonts\msyh.ttc", 0),
            (r"Fonts\simsun.ttc", 0),
            (r"Fonts\msyhbd.ttc", 0),
        ] {
            v.push((std::path::PathBuf::from(format!("{windir}\{rel}")), idx));
        }
    }
    #[cfg(target_os = "macos")]
    {
        for (p, idx) in [
            ("/Library/Fonts/Arial Unicode.ttf", 0u32),
            ("/System/Library/Fonts/STHeiti Light.ttc", 0),
            ("/System/Library/Fonts/PingFang.ttc", 0),
            ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
            ("/System/Library/Fonts/Supplemental/Songti.ttc", 0),
        ] {
            v.push((std::path::PathBuf::from(p), idx));
        }
    }
    #[cfg(target_os = "linux")]
    {
        for (p, idx) in [
            (
                "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
                0u32,
            ),
            ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0),
            (
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                0,
            ),
            (
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                0,
            ),
            ("/usr/share/fonts/truetype/arphic/uming.ttc", 0),
        ] {
            v.push((std::path::PathBuf::from(p), idx));
        }
    }
    v
}
