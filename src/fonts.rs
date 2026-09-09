//! CJK fonts — system TTF/TTC with face index (no extra crates).
use eframe::egui;
use std::path::PathBuf;

pub fn install_cjk_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    for (path, index) in cjk_candidates() {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        if data.len() < 2048 {
            continue;
        }
        let mut fd = egui::FontData::from_owned(data);
        fd.index = index;
        fonts.font_data.insert("cjk".to_owned(), fd);
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            let entry = fonts.families.entry(family).or_default();
            entry.retain(|n| n != "cjk");
            entry.insert(0, "cjk".to_owned());
        }
        ctx.set_fonts(fonts);
        eprintln!("CJK font loaded: {} (index {})", path.display(), index);
        return;
    }
    eprintln!("warning: no CJK font — install Microsoft YaHei / Noto Sans CJK");
}

fn cjk_candidates() -> Vec<(PathBuf, u32)> {
    let mut v = Vec::new();
    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        let base = PathBuf::from(windir);
        for (rel, idx) in [
            ("Fonts/msyh.ttf", 0u32),
            ("Fonts/simhei.ttf", 0),
            ("Fonts/simkai.ttf", 0),
            ("Fonts/msyh.ttc", 0),
            ("Fonts/msyhbd.ttc", 0),
            ("Fonts/simsun.ttc", 0),
            ("Fonts/msjhl.ttc", 0),
        ] {
            v.push((base.join(rel), idx));
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
            ("/System/Library/Fonts/Supplemental/Arial Unicode.ttf", 0),
        ] {
            v.push((PathBuf::from(p), idx));
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
            ("/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc", 0),
            (
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                0,
            ),
            (
                "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                0,
            ),
            ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
            ("/usr/share/fonts/truetype/arphic/uming.ttc", 0),
        ] {
            v.push((PathBuf::from(p), idx));
        }
    }
    v
}
