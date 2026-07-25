use std::fs;
use rltk::VirtualKeyCode;

const SETTINGS_PATH: &str = "settings.json";

// ── Keybindings ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Bindings {
    pub wait:      VirtualKeyCode,
    pub get_item:  VirtualKeyCode,
    pub disembark: VirtualKeyCode,
    pub inventory: VirtualKeyCode,
    pub equipment: VirtualKeyCode,
    pub ability:   VirtualKeyCode,
    pub juke:      VirtualKeyCode,
    pub look:      VirtualKeyCode,
    pub open_menu: VirtualKeyCode,
    pub freelook:  VirtualKeyCode,
}

impl Default for Bindings {
    fn default() -> Self {
        Bindings {
            wait:      VirtualKeyCode::Numpad5,
            get_item:  VirtualKeyCode::G,
            disembark: VirtualKeyCode::D,
            inventory: VirtualKeyCode::I,
            equipment: VirtualKeyCode::E,
            ability:   VirtualKeyCode::A,
            juke:      VirtualKeyCode::J,
            look:      VirtualKeyCode::L,
            open_menu: VirtualKeyCode::Escape,
            freelook:  VirtualKeyCode::F,
        }
    }
}

pub fn key_to_str(key: VirtualKeyCode) -> &'static str {
    match key {
        VirtualKeyCode::A => "A", VirtualKeyCode::B => "B", VirtualKeyCode::C => "C",
        VirtualKeyCode::D => "D", VirtualKeyCode::E => "E", VirtualKeyCode::F => "F",
        VirtualKeyCode::G => "G", VirtualKeyCode::H => "H", VirtualKeyCode::I => "I",
        VirtualKeyCode::J => "J", VirtualKeyCode::K => "K", VirtualKeyCode::L => "L",
        VirtualKeyCode::M => "M", VirtualKeyCode::N => "N", VirtualKeyCode::O => "O",
        VirtualKeyCode::P => "P", VirtualKeyCode::Q => "Q", VirtualKeyCode::R => "R",
        VirtualKeyCode::S => "S", VirtualKeyCode::T => "T", VirtualKeyCode::U => "U",
        VirtualKeyCode::V => "V", VirtualKeyCode::W => "W", VirtualKeyCode::X => "X",
        VirtualKeyCode::Y => "Y", VirtualKeyCode::Z => "Z",
        VirtualKeyCode::Key0 => "Key0", VirtualKeyCode::Key1 => "Key1",
        VirtualKeyCode::Key2 => "Key2", VirtualKeyCode::Key3 => "Key3",
        VirtualKeyCode::Key4 => "Key4", VirtualKeyCode::Key5 => "Key5",
        VirtualKeyCode::Key6 => "Key6", VirtualKeyCode::Key7 => "Key7",
        VirtualKeyCode::Key8 => "Key8", VirtualKeyCode::Key9 => "Key9",
        VirtualKeyCode::Numpad0 => "Numpad0", VirtualKeyCode::Numpad1 => "Numpad1",
        VirtualKeyCode::Numpad2 => "Numpad2", VirtualKeyCode::Numpad3 => "Numpad3",
        VirtualKeyCode::Numpad4 => "Numpad4", VirtualKeyCode::Numpad5 => "Numpad5",
        VirtualKeyCode::Numpad6 => "Numpad6", VirtualKeyCode::Numpad7 => "Numpad7",
        VirtualKeyCode::Numpad8 => "Numpad8", VirtualKeyCode::Numpad9 => "Numpad9",
        VirtualKeyCode::F1  => "F1",  VirtualKeyCode::F2  => "F2",
        VirtualKeyCode::F3  => "F3",  VirtualKeyCode::F4  => "F4",
        VirtualKeyCode::F5  => "F5",  VirtualKeyCode::F6  => "F6",
        VirtualKeyCode::F7  => "F7",  VirtualKeyCode::F8  => "F8",
        VirtualKeyCode::F9  => "F9",  VirtualKeyCode::F10 => "F10",
        VirtualKeyCode::F11 => "F11", VirtualKeyCode::F12 => "F12",
        VirtualKeyCode::Return   => "Return",   VirtualKeyCode::Space    => "Space",
        VirtualKeyCode::Escape   => "Escape",   VirtualKeyCode::Tab      => "Tab",
        VirtualKeyCode::Back     => "Back",
        VirtualKeyCode::Left     => "Left",     VirtualKeyCode::Right    => "Right",
        VirtualKeyCode::Up       => "Up",       VirtualKeyCode::Down     => "Down",
        VirtualKeyCode::Home     => "Home",     VirtualKeyCode::End      => "End",
        VirtualKeyCode::PageUp   => "PageUp",   VirtualKeyCode::PageDown => "PageDown",
        VirtualKeyCode::Insert   => "Insert",   VirtualKeyCode::Delete   => "Delete",
        VirtualKeyCode::Period      => "Period",     VirtualKeyCode::Comma      => "Comma",
        VirtualKeyCode::Slash       => "Slash",      VirtualKeyCode::Backslash  => "Backslash",
        VirtualKeyCode::Semicolon   => "Semicolon",  VirtualKeyCode::Apostrophe => "Apostrophe",
        VirtualKeyCode::Grave       => "Grave",      VirtualKeyCode::LBracket   => "LBracket",
        VirtualKeyCode::RBracket    => "RBracket",   VirtualKeyCode::Minus      => "Minus",
        VirtualKeyCode::Equals      => "Equals",
        _ => "Unknown",
    }
}

pub fn key_from_str(s: &str) -> Option<VirtualKeyCode> {
    match s {
        "A" => Some(VirtualKeyCode::A), "B" => Some(VirtualKeyCode::B), "C" => Some(VirtualKeyCode::C),
        "D" => Some(VirtualKeyCode::D), "E" => Some(VirtualKeyCode::E), "F" => Some(VirtualKeyCode::F),
        "G" => Some(VirtualKeyCode::G), "H" => Some(VirtualKeyCode::H), "I" => Some(VirtualKeyCode::I),
        "J" => Some(VirtualKeyCode::J), "K" => Some(VirtualKeyCode::K), "L" => Some(VirtualKeyCode::L),
        "M" => Some(VirtualKeyCode::M), "N" => Some(VirtualKeyCode::N), "O" => Some(VirtualKeyCode::O),
        "P" => Some(VirtualKeyCode::P), "Q" => Some(VirtualKeyCode::Q), "R" => Some(VirtualKeyCode::R),
        "S" => Some(VirtualKeyCode::S), "T" => Some(VirtualKeyCode::T), "U" => Some(VirtualKeyCode::U),
        "V" => Some(VirtualKeyCode::V), "W" => Some(VirtualKeyCode::W), "X" => Some(VirtualKeyCode::X),
        "Y" => Some(VirtualKeyCode::Y), "Z" => Some(VirtualKeyCode::Z),
        "Key0" => Some(VirtualKeyCode::Key0), "Key1" => Some(VirtualKeyCode::Key1),
        "Key2" => Some(VirtualKeyCode::Key2), "Key3" => Some(VirtualKeyCode::Key3),
        "Key4" => Some(VirtualKeyCode::Key4), "Key5" => Some(VirtualKeyCode::Key5),
        "Key6" => Some(VirtualKeyCode::Key6), "Key7" => Some(VirtualKeyCode::Key7),
        "Key8" => Some(VirtualKeyCode::Key8), "Key9" => Some(VirtualKeyCode::Key9),
        "Numpad0" => Some(VirtualKeyCode::Numpad0), "Numpad1" => Some(VirtualKeyCode::Numpad1),
        "Numpad2" => Some(VirtualKeyCode::Numpad2), "Numpad3" => Some(VirtualKeyCode::Numpad3),
        "Numpad4" => Some(VirtualKeyCode::Numpad4), "Numpad5" => Some(VirtualKeyCode::Numpad5),
        "Numpad6" => Some(VirtualKeyCode::Numpad6), "Numpad7" => Some(VirtualKeyCode::Numpad7),
        "Numpad8" => Some(VirtualKeyCode::Numpad8), "Numpad9" => Some(VirtualKeyCode::Numpad9),
        "F1"  => Some(VirtualKeyCode::F1),  "F2"  => Some(VirtualKeyCode::F2),
        "F3"  => Some(VirtualKeyCode::F3),  "F4"  => Some(VirtualKeyCode::F4),
        "F5"  => Some(VirtualKeyCode::F5),  "F6"  => Some(VirtualKeyCode::F6),
        "F7"  => Some(VirtualKeyCode::F7),  "F8"  => Some(VirtualKeyCode::F8),
        "F9"  => Some(VirtualKeyCode::F9),  "F10" => Some(VirtualKeyCode::F10),
        "F11" => Some(VirtualKeyCode::F11), "F12" => Some(VirtualKeyCode::F12),
        "Return"   => Some(VirtualKeyCode::Return),   "Space"    => Some(VirtualKeyCode::Space),
        "Escape"   => Some(VirtualKeyCode::Escape),   "Tab"      => Some(VirtualKeyCode::Tab),
        "Back"     => Some(VirtualKeyCode::Back),
        "Left"     => Some(VirtualKeyCode::Left),     "Right"    => Some(VirtualKeyCode::Right),
        "Up"       => Some(VirtualKeyCode::Up),       "Down"     => Some(VirtualKeyCode::Down),
        "Home"     => Some(VirtualKeyCode::Home),     "End"      => Some(VirtualKeyCode::End),
        "PageUp"   => Some(VirtualKeyCode::PageUp),   "PageDown" => Some(VirtualKeyCode::PageDown),
        "Insert"   => Some(VirtualKeyCode::Insert),   "Delete"   => Some(VirtualKeyCode::Delete),
        "Period"     => Some(VirtualKeyCode::Period),     "Comma"     => Some(VirtualKeyCode::Comma),
        "Slash"      => Some(VirtualKeyCode::Slash),      "Backslash" => Some(VirtualKeyCode::Backslash),
        "Semicolon"  => Some(VirtualKeyCode::Semicolon),  "Apostrophe"=> Some(VirtualKeyCode::Apostrophe),
        "Grave"      => Some(VirtualKeyCode::Grave),      "LBracket"  => Some(VirtualKeyCode::LBracket),
        "RBracket"   => Some(VirtualKeyCode::RBracket),   "Minus"     => Some(VirtualKeyCode::Minus),
        "Equals"     => Some(VirtualKeyCode::Equals),
        _ => None,
    }
}

fn parse_binding<'a>(
    obj: Option<&'a serde_json::Map<String, serde_json::Value>>,
    action: &str,
    default: VirtualKeyCode,
) -> VirtualKeyCode {
    obj.and_then(|o| o.get(action))
        .and_then(|v| v.as_str())
        .and_then(key_from_str)
        .unwrap_or(default)
}

// ── RebindTarget ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RebindTarget {
    Wait, GetItem, Disembark, Inventory, Equipment, Ability, Juke, Look, OpenMenu, Freelook,
}

// ── FontSize ─────────────────────────────────────────────────────────────────

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

    pub fn font_file(self) -> &'static str {
        match self {
            FontSize::Small | FontSize::XLarge => "CGA8x8thick.png",
            FontSize::Medium => "rexpaint_cp437_10x10.png",
            FontSize::Large => "Alloy_curses_12x12.png",
        }
    }

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

// ── Settings ──────────────────────────────────────────────────────────────────

pub struct Settings {
    pub font_size: FontSize,
    pub fullscreen: bool,
    pub bindings: Bindings,
}

impl Settings {
    pub fn load() -> Self {
        let v: Option<serde_json::Value> = fs::read_to_string(SETTINGS_PATH)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());
        let font_size = v.as_ref()
            .and_then(|v| v["font_px"].as_u64())
            .map(|px| FontSize::from_px(px as u32))
            .unwrap_or(FontSize::Medium);
        let fullscreen = v.as_ref()
            .and_then(|v| v["fullscreen"].as_bool())
            .unwrap_or(false);
        let b = v.as_ref().and_then(|v| v["bindings"].as_object());
        let bindings = Bindings {
            wait:      parse_binding(b, "wait",      VirtualKeyCode::Numpad5),
            get_item:  parse_binding(b, "get_item",  VirtualKeyCode::G),
            disembark: parse_binding(b, "disembark", VirtualKeyCode::D),
            inventory: parse_binding(b, "inventory", VirtualKeyCode::I),
            equipment: parse_binding(b, "equipment", VirtualKeyCode::E),
            ability:   parse_binding(b, "ability",   VirtualKeyCode::A),
            juke:      parse_binding(b, "juke",      VirtualKeyCode::J),
            look:      parse_binding(b, "look",      VirtualKeyCode::L),
            open_menu: parse_binding(b, "open_menu", VirtualKeyCode::Escape),
            freelook:  parse_binding(b, "freelook",  VirtualKeyCode::F),
        };
        Settings { font_size, fullscreen, bindings }
    }

    pub fn save(&self) {
        let json = serde_json::json!({
            "font_px":    self.font_size.tile_px(),
            "fullscreen": self.fullscreen,
            "bindings": {
                "wait":      key_to_str(self.bindings.wait),
                "get_item":  key_to_str(self.bindings.get_item),
                "disembark": key_to_str(self.bindings.disembark),
                "inventory": key_to_str(self.bindings.inventory),
                "equipment": key_to_str(self.bindings.equipment),
                "ability":   key_to_str(self.bindings.ability),
                "juke":      key_to_str(self.bindings.juke),
                "look":      key_to_str(self.bindings.look),
                "open_menu": key_to_str(self.bindings.open_menu),
                "freelook":  key_to_str(self.bindings.freelook),
            }
        });
        let _ = fs::write(SETTINGS_PATH, serde_json::to_string_pretty(&json).unwrap());
    }
}
