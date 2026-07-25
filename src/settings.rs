use std::fs;

const SETTINGS_PATH: &str = "settings.json";

#[derive(Clone, Copy, PartialEq)]
pub enum FontSize {
    Small,   //  8×8  → 1280×720
    Medium,  // 10×10 → 1600×900
    Large,   // 12×12 → 1920×1080
    XLarge,  // 16×16 → 2560×1440
}

impl FontSize {
    pub fn tile_px(self) -> u32 {
        match self {
            FontSize::Small   =>  8,
            FontSize::Medium  => 10,
            FontSize::Large   => 12,
            FontSize::XLarge  => 16,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FontSize::Small   => "Small  (1280×720)",
            FontSize::Medium  => "Medium (1600×900)",
            FontSize::Large   => "Large  (1920×1080)",
            FontSize::XLarge  => "XLarge (2560×1440)",
        }
    }

    /// The font bitmap file to load for this tier.
    pub fn font_file(self) -> &'static str {
        match self {
            FontSize::Small | FontSize::XLarge => "CGA8x8thick.png",
            FontSize::Medium => "rexpaint_cp437_10x10.png",
            FontSize::Large => "Alloy_curses_12x12.png",
        }
    }

    /// The native glyph size declared in the font bitmap.
    pub fn font_native_px(self) -> u32 {
        match self {
            FontSize::Small | FontSize::XLarge => 8,
            FontSize::Medium => 10,
            FontSize::Large => 12,
        }
    }

    pub fn next(self) -> FontSize {
        match self {
            FontSize::Small   => FontSize::Medium,
            FontSize::Medium  => FontSize::Large,
            FontSize::Large   => FontSize::XLarge,
            FontSize::XLarge  => FontSize::Small,
        }
    }

    fn from_px(px: u32) -> FontSize {
        match px {
            8  => FontSize::Small,
            12 => FontSize::Large,
            16 => FontSize::XLarge,
            _  => FontSize::Medium,
        }
    }
}

pub struct Settings {
    pub font_size: FontSize,
}

impl Settings {
    pub fn load() -> Self {
        let font_size = fs::read_to_string(SETTINGS_PATH)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["font_px"].as_u64())
            .map(|px| FontSize::from_px(px as u32))
            .unwrap_or(FontSize::Medium);
        Settings { font_size }
    }

    pub fn save(&self) {
        let json = serde_json::json!({ "font_px": self.font_size.tile_px() });
        let _ = fs::write(SETTINGS_PATH, serde_json::to_string_pretty(&json).unwrap());
    }
}
