//! Load system CJK fonts so Chinese/Japanese/Korean render correctly.
use eframe::egui;

/// Candidate font files per OS (first readable wins).
fn cjk_font_candidates() -> Vec<std::path::PathBuf> {
    let mut v = Vec::new();
    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        for name in [
            "Fonts\\msyh.ttc",
            "Fonts\\msyh.ttf",
            "Fonts\\msyhbd.ttc",
            "Fonts\\simhei.ttf",
            "Fonts\\simsun.ttc",
            "Fonts\\NotoSansSC-Regular.otf",
            "Fonts\\SourceHanSansSC-Regular.otf",
        ] {
            v.push(std::path::PathBuf::from(format!("{windir}\\{name}")));
        }
    }
    #[cfg(target_os = "macos")]
    {
        for p in [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Supplemental/Songti.ttc",
        ] {
            v.push(std::path::PathBuf::from(p));
        }
    }
    #[cfg(target_os = "linux")]
    {
        for p in [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ] {
            v.push(std::path::PathBuf::from(p));
        }
    }
    v
}

pub fn install_cjk_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let mut loaded = false;
    for path in cjk_font_candidates() {
        if let Ok(data) = std::fs::read(&path) {
            // Skip tiny/invalid reads
            if data.len() < 1024 {
                continue;
            }
            fonts.font_data.insert(
                "cjk".to_owned(),
                egui::FontData::from_owned(data),
            );
            // Prefer CJK for proportional + monospace fallback chain
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
            loaded = true;
            eprintln!("loaded CJK font: {}", path.display());
            break;
        }
    }

    if !loaded {
        eprintln!("warning: no system CJK font found; Chinese may show as □");
    }

    ctx.set_fonts(fonts);
}
