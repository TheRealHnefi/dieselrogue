//! Custom game engine: a from-scratch reimplementation of the RLTK interface
//! the game was originally written against — a replacement, not a wrapper.
//! The API shape follows RLTK for familiarity, but the names are generic
//! (`Context`, `Console`, ...) because this interface is expected to diverge
//! from RLTK over time.
//!
//! Provides the types, algorithms, and rendering primitives the game needs:
//! geometry (`Point`, `PointF`), colors (`RGB`, `RGBA`), CP437 glyph mapping,
//! Bresenham lines, distance algorithms, a seeded PRNG, keyboard input types,
//! consoles and the `Context` object, REXPaint `.xp` parsing, embedded
//! resources, the minifb windowing/rendering backend, and the main loop.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops;
use std::sync::{Mutex, OnceLock};

// ─── Geometry ────────────────────────────────────────────────────────────────

/// Integer 2D point. Interface based on RLTK's `Point`.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    /// Create a point from a pair of coordinates. Accepts any type convertible
    /// to `i32` (e.g. `i32`, `usize`); failed conversions become 0.
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: TryInto<i32>,
    {
        Self {
            x: x.try_into().ok().unwrap_or(0),
            y: y.try_into().ok().unwrap_or(0),
        }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

// Point math overloads (same operator surface as RLTK's `Point`).

impl ops::Add<Point> for Point {
    type Output = Point;
    fn add(mut self, rhs: Point) -> Point {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl ops::Add<i32> for Point {
    type Output = Point;
    fn add(mut self, rhs: i32) -> Point {
        self.x += rhs;
        self.y += rhs;
        self
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(mut self, rhs: Point) -> Point {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl ops::Sub<i32> for Point {
    type Output = Point;
    fn sub(mut self, rhs: i32) -> Point {
        self.x -= rhs;
        self.y -= rhs;
        self
    }
}

impl ops::Mul<Point> for Point {
    type Output = Point;
    fn mul(mut self, rhs: Point) -> Point {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self
    }
}

impl ops::Mul<i32> for Point {
    type Output = Point;
    fn mul(mut self, rhs: i32) -> Point {
        self.x *= rhs;
        self.y *= rhs;
        self
    }
}

impl ops::Mul<f32> for Point {
    type Output = Point;
    fn mul(mut self, rhs: f32) -> Point {
        self.x = (self.x as f32 * rhs) as i32;
        self.y = (self.y as f32 * rhs) as i32;
        self
    }
}

impl ops::Div<Point> for Point {
    type Output = Point;
    fn div(mut self, rhs: Point) -> Point {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self
    }
}

impl ops::Div<i32> for Point {
    type Output = Point;
    fn div(mut self, rhs: i32) -> Point {
        self.x /= rhs;
        self.y /= rhs;
        self
    }
}

impl ops::Div<f32> for Point {
    type Output = Point;
    fn div(mut self, rhs: f32) -> Point {
        self.x = (self.x as f32 / rhs) as i32;
        self.y = (self.y as f32 / rhs) as i32;
        self
    }
}

impl ops::AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl ops::SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl ops::MulAssign for Point {
    fn mul_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x * other.x,
            y: self.y * other.y,
        };
    }
}

impl ops::DivAssign for Point {
    fn div_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x / other.x,
            y: self.y / other.y,
        };
    }
}

/// Floating-point 2D point. Interface based on RLTK's `PointF`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}

impl PointF {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Angle in radians. Interface based on RLTK's `Radians`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Radians(pub f32);

// ─── Glyph type ──────────────────────────────────────────────────────────────

/// CP437 glyph index (0–255). Interface based on RLTK's `FontCharType`.
pub type FontCharType = u8;

// ─── Colors ──────────────────────────────────────────────────────────────────

/// Named color constants. Interface based on RLTK's `NamedColor`, `RED`, etc.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum NamedColor {
    Black,
    White,
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
    Gray,
    DarkGray,
    LightGray,
    Orange,
    Pink,
    Purple,
    Brown,
    Lime,
    Navy,
    Teal,
    Olive,
    Maroon,
    Silver,
    Aqua,
    Fuchsia,
}

pub const BLACK: NamedColor = NamedColor::Black;
pub const WHITE: NamedColor = NamedColor::White;
pub const RED: NamedColor = NamedColor::Red;
pub const GREEN: NamedColor = NamedColor::Green;
pub const BLUE: NamedColor = NamedColor::Blue;
pub const CYAN: NamedColor = NamedColor::Cyan;
pub const MAGENTA: NamedColor = NamedColor::Magenta;
pub const YELLOW: NamedColor = NamedColor::Yellow;
pub const GRAY: NamedColor = NamedColor::Gray;
pub const DARKGRAY: NamedColor = NamedColor::DarkGray;
pub const LIGHTGRAY: NamedColor = NamedColor::LightGray;
pub const ORANGE: NamedColor = NamedColor::Orange;
pub const PINK: NamedColor = NamedColor::Pink;
pub const PURPLE: NamedColor = NamedColor::Purple;
pub const BROWN: NamedColor = NamedColor::Brown;
pub const LIME: NamedColor = NamedColor::Lime;
pub const NAVY: NamedColor = NamedColor::Navy;
pub const TEAL: NamedColor = NamedColor::Teal;
pub const OLIVE: NamedColor = NamedColor::Olive;
pub const MAROON: NamedColor = NamedColor::Maroon;
pub const SILVER: NamedColor = NamedColor::Silver;
pub const AQUA: NamedColor = NamedColor::Aqua;
pub const FUCHSIA: NamedColor = NamedColor::Fuchsia;

/// RGB color with float channels in [0.0, 1.0]. Interface based on RLTK's `RGB`.
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RGB {
    pub fn named(color: NamedColor) -> Self {
        match color {
            NamedColor::Black   => Self { r: 0.0, g: 0.0, b: 0.0 },
            NamedColor::White   => Self { r: 1.0, g: 1.0, b: 1.0 },
            NamedColor::Red     => Self { r: 1.0, g: 0.0, b: 0.0 },
            NamedColor::Green   => Self { r: 0.0, g: 1.0, b: 0.0 },
            NamedColor::Blue    => Self { r: 0.0, g: 0.0, b: 1.0 },
            NamedColor::Cyan    => Self { r: 0.0, g: 1.0, b: 1.0 },
            NamedColor::Magenta => Self { r: 1.0, g: 0.0, b: 1.0 },
            NamedColor::Yellow  => Self { r: 1.0, g: 1.0, b: 0.0 },
            NamedColor::Gray    => Self { r: 0.5, g: 0.5, b: 0.5 },
            NamedColor::DarkGray => Self { r: 0.25, g: 0.25, b: 0.25 },
            NamedColor::LightGray => Self { r: 0.75, g: 0.75, b: 0.75 },
            NamedColor::Orange  => Self { r: 1.0, g: 0.647, b: 0.0 },
            NamedColor::Pink    => Self { r: 1.0, g: 0.753, b: 0.796 },
            NamedColor::Purple  => Self { r: 0.5, g: 0.0, b: 0.5 },
            NamedColor::Brown   => Self { r: 0.647, g: 0.165, b: 0.165 },
            NamedColor::Lime    => Self { r: 0.0, g: 1.0, b: 0.0 },
            NamedColor::Navy    => Self { r: 0.0, g: 0.0, b: 0.5 },
            NamedColor::Teal    => Self { r: 0.0, g: 0.5, b: 0.5 },
            NamedColor::Olive   => Self { r: 0.5, g: 0.5, b: 0.0 },
            NamedColor::Maroon  => Self { r: 0.5, g: 0.0, b: 0.0 },
            NamedColor::Silver  => Self { r: 0.753, g: 0.753, b: 0.753 },
            NamedColor::Aqua    => Self { r: 0.0, g: 1.0, b: 1.0 },
            NamedColor::Fuchsia => Self { r: 1.0, g: 0.0, b: 1.0 },
        }
    }

    /// Create from 8-bit channel values (0–255).
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    /// Create from float channel values (0.0–1.0).
    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

/// RGBA color with float channels and alpha. Interface based on RLTK's `RGBA`.
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RGBA {
    /// Create from 8-bit channel values (0–255).
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create from float channel values (0.0–1.0).
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to an RGB (drops alpha).
    pub fn to_rgb(self) -> RGB {
        RGB { r: self.r, g: self.g, b: self.b }
    }
}

impl From<RGB> for RGBA {
    fn from(item: RGB) -> Self {
        Self::from_f32(item.r, item.g, item.b, 1.0)
    }
}

impl From<RGBA> for RGB {
    fn from(item: RGBA) -> Self {
        Self::from_f32(item.r, item.g, item.b)
    }
}

// ─── CP437 mapping ───────────────────────────────────────────────────────────

/// Convert a Unicode character to its CP437 glyph index.
/// ASCII printable characters map to themselves; extended characters use the
/// CP437 table. A character with no CP437 glyph that the game renders is
/// substituted with the closest ASCII glyph (`×` becomes `x`); anything else
/// unknown returns `'?'` (0x3F).
pub fn to_cp437(c: char) -> FontCharType {
    // Fast path: ASCII printable maps directly.
    if c as u32 >= 0x20 && c as u32 <= 0x7E {
        return c as u8;
    }

    match c {
        // CP437 control-range glyphs (bytes 1–31)
        '\u{263A}' => 1,   // ☺
        '\u{263B}' => 2,   // ☻
        '\u{2665}' => 3,   // ♥
        '\u{2666}' => 4,   // ♦
        '\u{2663}' => 5,   // ♣
        '\u{2660}' => 6,   // ♠
        '\u{2022}' => 7,   // •
        '\u{25D8}' => 8,   // ◘
        '\u{25CB}' => 9,   // ○
        '\u{25D9}' => 10,  // ◙
        '\u{2642}' => 11,  // ♂
        '\u{2640}' => 12,  // ♀
        '\u{266A}' => 13,  // ♪
        '\u{266B}' => 14,  // ♫
        '\u{263C}' => 15,  // ☼
        '\u{25BA}' => 16,  // ►
        '\u{25C4}' => 17,  // ◄
        '\u{2195}' => 18,  // ↕
        '\u{203C}' => 19,  // ‼
        '\u{00B6}' => 20,  // ¶
        '\u{00A7}' => 21,  // §
        '\u{25AC}' => 22,  // ▬
        '\u{21A8}' => 23,  // ↨
        '\u{2191}' => 24,  // ↑
        '\u{2193}' => 25,  // ↓
        '\u{2192}' => 26,  // →
        '\u{2190}' => 27,  // ←
        '\u{221F}' => 28,  // ∟
        '\u{2194}' => 29,  // ↔
        '\u{25B2}' => 30,  // ▲
        '\u{25BC}' => 31,  // ▼

        // Latin-1 supplement (bytes 128–175)
        '\u{00C7}' => 128, // Ç
        '\u{00FC}' => 129, // ü
        '\u{00E9}' => 130, // é
        '\u{00E2}' => 131, // â
        '\u{00E4}' => 132, // ä
        '\u{00E0}' => 133, // à
        '\u{00E5}' => 134, // å
        '\u{00E7}' => 135, // ç
        '\u{00EA}' => 136, // ê
        '\u{00EB}' => 137, // ë
        '\u{00E8}' => 138, // è
        '\u{00EF}' => 139, // ï
        '\u{00EE}' => 140, // î
        '\u{00EC}' => 141, // ì
        '\u{00C4}' => 142, // Ä
        '\u{00C5}' => 143, // Å
        '\u{00C9}' => 144, // É
        '\u{00E6}' => 145, // æ
        '\u{00C6}' => 146, // Æ
        '\u{00F4}' => 147, // ô
        '\u{00F6}' => 148, // ö
        '\u{00F2}' => 149, // ò
        '\u{00FB}' => 150, // û
        '\u{00F9}' => 151, // ù
        '\u{00FF}' => 152, // ÿ
        '\u{00D6}' => 153, // Ö
        '\u{00DC}' => 154, // Ü
        '\u{00A2}' => 155, // ¢
        '\u{00A3}' => 156, // £
        '\u{00A5}' => 157, // ¥
        '\u{20A7}' => 158, // ₧
        '\u{0192}' => 159, // ƒ
        '\u{00E1}' => 160, // á
        '\u{00ED}' => 161, // í
        '\u{00F3}' => 162, // ó
        '\u{00FA}' => 163, // ú
        '\u{00F1}' => 164, // ñ
        '\u{00D1}' => 165, // Ñ
        '\u{00AA}' => 166, // ª
        '\u{00BA}' => 167, // º
        '\u{00BF}' => 168, // ¿
        '\u{2310}' => 169, // ⌐
        '\u{00AC}' => 170, // ¬
        '\u{00BD}' => 171, // ½
        '\u{00BC}' => 172, // ¼
        '\u{00A1}' => 173, // ¡
        '\u{00AB}' => 174, // «
        '\u{00BB}' => 175, // »

        // Shades and box drawing (bytes 176–223)
        '\u{2591}' => 176, // ░
        '\u{2592}' => 177, // ▒
        '\u{2593}' => 178, // ▓
        '\u{2502}' => 179, // │
        '\u{2524}' => 180, // ┤
        '\u{2561}' => 181, // ╡
        '\u{2562}' => 182, // ╢
        '\u{2556}' => 183, // ╖
        '\u{2555}' => 184, // ╕
        '\u{2563}' => 185, // ╣
        '\u{2551}' => 186, // ║
        '\u{2557}' => 187, // ╗
        '\u{255D}' => 188, // ╝
        '\u{255C}' => 189, // ╜
        '\u{255B}' => 190, // ╛
        '\u{2510}' => 191, // ┐
        '\u{2514}' => 192, // └
        '\u{2534}' => 193, // ┴
        '\u{252C}' => 194, // ┬
        '\u{251C}' => 195, // ├
        '\u{2500}' => 196, // ─
        '\u{253C}' => 197, // ┼
        '\u{255E}' => 198, // ╞
        '\u{255F}' => 199, // ╟
        '\u{255A}' => 200, // ╚
        '\u{2554}' => 201, // ╔
        '\u{2569}' => 202, // ╩
        '\u{2566}' => 203, // ╦
        '\u{2560}' => 204, // ╠
        '\u{2550}' => 205, // ═
        '\u{256C}' => 206, // ╬
        '\u{2567}' => 207, // ╧
        '\u{2568}' => 208, // ╨
        '\u{2564}' => 209, // ╤
        '\u{2565}' => 210, // ╥
        '\u{2559}' => 211, // ╙
        '\u{2558}' => 212, // ╘
        '\u{2552}' => 213, // ╒
        '\u{2553}' => 214, // ╓
        '\u{256B}' => 215, // ╫
        '\u{256A}' => 216, // ╪
        '\u{2518}' => 217, // ┘
        '\u{250C}' => 218, // ┌
        '\u{2588}' => 219, // █
        '\u{2584}' => 220, // ▄
        '\u{258C}' => 221, // ▌
        '\u{2590}' => 222, // ▐
        '\u{2580}' => 223, // ▀

        // Greek and math symbols (bytes 224–255)
        '\u{03B1}' => 224, // α
        '\u{00DF}' => 225, // ß
        '\u{0393}' => 226, // Γ
        '\u{03C0}' => 227, // π
        '\u{03A3}' => 228, // Σ
        '\u{03C3}' => 229, // σ
        '\u{00B5}' => 230, // µ
        '\u{03C4}' => 231, // τ
        '\u{03A6}' => 232, // Φ
        '\u{0398}' => 233, // Θ
        '\u{03A9}' => 234, // Ω
        '\u{03B4}' => 235, // δ
        '\u{221E}' => 236, // ∞
        '\u{03C6}' => 237, // φ
        '\u{03B5}' => 238, // ε
        '\u{2229}' => 239, // ∩
        '\u{2261}' => 240, // ≡
        '\u{00B1}' => 241, // ±
        '\u{2265}' => 242, // ≥
        '\u{2264}' => 243, // ≤
        '\u{2320}' => 244, // ⌠
        '\u{2321}' => 245, // ⌡
        '\u{00F7}' => 246, // ÷
        '\u{2248}' => 247, // ≈
        '\u{00B0}' => 248, // °
        '\u{2219}' => 249, // ∙
        '\u{00B7}' => 250, // ·
        '\u{221A}' => 251, // √
        '\u{207F}' => 252, // ⁿ
        '\u{00B2}' => 253, // ²
        '\u{25A0}' => 254, // ■
        '\u{00A0}' => 255, // NBSP

        // Typographic substitutions: characters with no CP437 glyph that the
        // game renders, mapped to the closest ASCII glyph.
        '\u{00D7}' => b'x', // × multiplication sign (resolution labels: "1920×1080")

        // Fallback for unknown characters
        _ => b'?',
    }
}

// ─── Distance algorithms ─────────────────────────────────────────────────────

/// Distance calculation algorithm. Interface based on RLTK's `DistanceAlg`.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum DistanceAlg {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

impl DistanceAlg {
    /// Euclidean distance between two points.
    pub fn distance2d(&self, a: Point, b: Point) -> f32 {
        match self {
            DistanceAlg::Pythagoras => {
                let dx = (a.x - b.x) as f32;
                let dy = (a.y - b.y) as f32;
                (dx * dx + dy * dy).sqrt()
            }
            DistanceAlg::Manhattan => ((a.x - b.x).abs() + (a.y - b.y).abs()) as f32,
            DistanceAlg::Chebyshev => ((a.x - b.x).abs().max((a.y - b.y).abs())) as f32,
        }
    }
}

// ─── Line algorithms ─────────────────────────────────────────────────────────

/// Line drawing algorithm. Interface based on RLTK's `LineAlg`.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum LineAlg {
    Bresenham,
}

/// Returns all integer points on the line from `a` to `b` inclusive,
/// using the specified algorithm. Interface based on RLTK's `line2d`.
pub fn line2d(alg: LineAlg, a: Point, b: Point) -> Vec<Point> {
    match alg {
        LineAlg::Bresenham => bresenham_line(a, b),
    }
}

/// Standard integer Bresenham line algorithm covering all octants.
fn bresenham_line(a: Point, b: Point) -> Vec<Point> {
    let mut points = Vec::new();
    let (mut x, mut y) = (a.x, a.y);
    let (x1, y1) = (b.x, b.y);

    let dx = (x1 - x).abs();
    let dy = (y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        points.push(Point { x, y });
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    points
}

// ─── Random number generator ─────────────────────────────────────────────────

/// Trait for types that can be used with [`RandomNumberGenerator::range`].
/// Uses `i128` as the intermediate type so signed and unsigned integers
/// (including negative ranges) are handled correctly.
pub trait SampleRange: Copy + PartialOrd {
    fn from_i128(v: i128) -> Self;
    fn to_i128(self) -> i128;
}

impl SampleRange for i32 {
    fn from_i128(v: i128) -> Self { v as i32 }
    fn to_i128(self) -> i128 { self as i128 }
}

impl SampleRange for i64 {
    fn from_i128(v: i128) -> Self { v as i64 }
    fn to_i128(self) -> i128 { self as i128 }
}

impl SampleRange for u32 {
    fn from_i128(v: i128) -> Self { v as u32 }
    fn to_i128(self) -> i128 { self as i128 }
}

impl SampleRange for u64 {
    fn from_i128(v: i128) -> Self { v as u64 }
    fn to_i128(self) -> i128 { self as i128 }
}

impl SampleRange for usize {
    fn from_i128(v: i128) -> Self { v as usize }
    fn to_i128(self) -> i128 { self as i128 }
}

impl SampleRange for isize {
    fn from_i128(v: i128) -> Self { v as isize }
    fn to_i128(self) -> i128 { self as i128 }
}

/// Seeded PRNG using xorshift64*. Interface based on RLTK's `RandomNumberGenerator`.
///
/// Deterministic: the same seed produces the same sequence. `range(min, max)`
/// returns a value in `[min, max)` (max exclusive).
pub struct RandomNumberGenerator {
    state: u64,
}

impl RandomNumberGenerator {
    /// Create with a seed derived from the system clock.
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEADBEEFCAFEF00D);
        Self::seeded(seed)
    }

    /// Create with an explicit seed.
    pub fn seeded(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xDEADBEEFCAFEF00D } else { seed },
        }
    }

    /// Next raw u64 value (xorshift64*).
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Random value in `[min, max)` (max exclusive).
    pub fn range<T: SampleRange>(&mut self, min: T, max: T) -> T {
        let lo = min.to_i128();
        let hi = max.to_i128();
        assert!(lo < hi, "range: min ({}) must be less than max ({})", lo, hi);
        let span = (hi - lo) as u64;
        let result = lo + (self.next_u64() % span) as i128;
        T::from_i128(result)
    }
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Keyboard input ──────────────────────────────────────────────────────────

/// Virtual key codes. Interface based on RLTK's `VirtualKeyCode` (which mirrors winit's).
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum VirtualKeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Top-row numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Numpad numbers
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    // Whitespace / editing
    Return, Space, Escape, Tab, Back,
    // Navigation
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown, Insert, Delete,
    // Punctuation
    Period, Comma, Slash, Backslash, Semicolon, Apostrophe,
    Grave, LBracket, RBracket, Minus, Equals,
    // Modifiers
    LShift, RShift, LControl, RControl, LAlt, RAlt,
    // Numpad operators
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadDecimal, NumpadEnter,
    // Other
    CapsLock, ScrollLock, NumLock, PrintScreen, Pause,
    Apps, Power, Sleep, Wake,
}

// ─── Error type ──────────────────────────────────────────────────────────────

/// Error type returned by builder/main_loop. Interface based on RLTK's `BError`.
#[derive(Debug)]
pub struct BError {
    pub message: String,
}

impl BError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl std::fmt::Display for BError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BError {}

// ─── Console cell ────────────────────────────────────────────────────────────

/// A single console cell: a glyph with foreground and background colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub glyph: FontCharType,
    pub fg: RGBA,
    pub bg: RGBA,
}

/// One drawable tile on a fancy (sparse) console.
///
/// Fancy consoles accumulate overlapping tiles instead of overwriting cells —
/// matching RLTK's `FlexiConsole` — so a later tile alpha-blends over earlier
/// tiles (and the consoles below) at render time. This is what keeps e.g. the
/// aiming cursor visible beneath the semi-transparent targeting-range tint.
struct SparseTile {
    x: i32,
    y: i32,
    z_order: i32,
    glyph: FontCharType,
    fg: RGBA,
    bg: RGBA,
}

/// A console. Interface based on RLTK's console types (bracket-terminal's
/// `SimpleConsole` and `FlexiConsole`); reimplemented from scratch.
///
/// Dense consoles (`with_dimensions` / `with_simple_console`) hold a grid of
/// cells that `set` overwrites, and `cls` fills with opaque black. Fancy
/// consoles (`with_fancy_console`) are sparse: `set`/`set_fancy`/`print` push
/// tiles that blend over each other in z-order (stable for ties), `cls` drops
/// them all, and untouched cells stay fully transparent so lower consoles show
/// through — matching RLTK's `FlexiConsole`.
pub struct Console {
    width: i32,
    height: i32,
    /// Dense cell grid (dense consoles only; empty for fancy consoles).
    tiles: Vec<Cell>,
    /// Pushed tiles in draw order (fancy consoles only).
    sparse_tiles: Vec<SparseTile>,
    fancy: bool,
    /// Index into the context's font list, used when rendering.
    font_index: usize,
}

impl Console {
    fn new(width: i32, height: i32, fancy: bool) -> Self {
        let tiles = if fancy {
            Vec::new()
        } else {
            vec![Self::default_cell(false); (width * height) as usize]
        };
        Self {
            width,
            height,
            tiles,
            sparse_tiles: Vec::new(),
            fancy,
            font_index: 0,
        }
    }

    fn default_cell(fancy: bool) -> Cell {
        if fancy {
            // Transparent: space glyph, fully transparent fg and bg
            Cell {
                glyph: 32,
                fg: RGBA::from_f32(0.0, 0.0, 0.0, 0.0),
                bg: RGBA::from_f32(0.0, 0.0, 0.0, 0.0),
            }
        } else {
            // Opaque: space glyph, white fg on black bg
            Cell {
                glyph: 32,
                fg: RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                bg: RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            }
        }
    }

    fn try_at(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    /// Set a single cell. Out-of-bounds coordinates are silently ignored.
    ///
    /// On fancy (sparse) consoles this pushes a tile that blends over earlier
    /// tiles at render time; it does not replace them.
    pub fn set(&mut self, x: i32, y: i32, fg: RGBA, bg: RGBA, glyph: FontCharType) {
        if self.fancy {
            if self.try_at(x, y).is_some() {
                self.sparse_tiles.push(SparseTile { x, y, z_order: 0, glyph, fg, bg });
            }
        } else if let Some(idx) = self.try_at(x, y) {
            self.tiles[idx] = Cell { glyph, fg, bg };
        }
    }

    /// Push a tile with an explicit z-order. On fancy consoles the tile blends
    /// over earlier tiles in z-order (stable for ties); on dense consoles it
    /// behaves as a plain `set`. Out-of-bounds coordinates are silently ignored.
    pub fn set_fancy(&mut self, x: i32, y: i32, z_order: i32, fg: RGBA, bg: RGBA, glyph: FontCharType) {
        if self.fancy {
            if self.try_at(x, y).is_some() {
                self.sparse_tiles.push(SparseTile { x, y, z_order, glyph, fg, bg });
            }
        } else {
            self.set(x, y, fg, bg, glyph);
        }
    }

    /// Print a string starting at (x, y), converting each char to CP437.
    pub fn print_color(&mut self, mut x: i32, y: i32, fg: RGBA, bg: RGBA, output: &str) {
        for c in output.chars() {
            self.set(x, y, fg, bg, to_cp437(c));
            x += 1;
        }
    }

    /// Print a string centered horizontally on the console at row y.
    pub fn print_color_centered(&mut self, y: i32, fg: RGBA, bg: RGBA, text: &str) {
        let len = text.chars().count() as i32;
        let x = self.width / 2 - len / 2;
        self.print_color(x, y, fg, bg, text);
    }

    /// Clear the console. Fancy consoles drop all pushed tiles (transparent);
    /// dense consoles fill with opaque black.
    pub fn cls(&mut self) {
        if self.fancy {
            self.sparse_tiles.clear();
        } else {
            let default = Self::default_cell(false);
            for tile in &mut self.tiles {
                *tile = default;
            }
        }
    }

    /// Draw a filled box with a single-line border.
    pub fn draw_box(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        // Fill interior with space, white fg / black bg (using fg's alpha)
        for y in sy..sy + height {
            for x in sx..sx + width {
                self.set(
                    x, y,
                    RGBA::from_f32(1.0, 1.0, 1.0, fg.a),
                    RGBA::from_f32(0.0, 0.0, 0.0, fg.a),
                    32,
                );
            }
        }
        // Border
        self.set(sx, sy, fg, bg, to_cp437('┌'));
        self.set(sx + width, sy, fg, bg, to_cp437('┐'));
        self.set(sx, sy + height, fg, bg, to_cp437('└'));
        self.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
        for x in sx + 1..sx + width {
            self.set(x, sy, fg, bg, to_cp437('─'));
            self.set(x, sy + height, fg, bg, to_cp437('─'));
        }
        for y in sy + 1..sy + height {
            self.set(sx, y, fg, bg, to_cp437('│'));
            self.set(sx + width, y, fg, bg, to_cp437('│'));
        }
    }

    /// Draw a hollow box with a double-line border (no interior fill).
    pub fn draw_hollow_box_double(&mut self, sx: i32, sy: i32, width: i32, height: i32, fg: RGBA, bg: RGBA) {
        self.set(sx, sy, fg, bg, to_cp437('╔'));
        self.set(sx + width, sy, fg, bg, to_cp437('╗'));
        self.set(sx, sy + height, fg, bg, to_cp437('╚'));
        self.set(sx + width, sy + height, fg, bg, to_cp437('╝'));
        for x in sx + 1..sx + width {
            self.set(x, sy, fg, bg, to_cp437('═'));
            self.set(x, sy + height, fg, bg, to_cp437('═'));
        }
        for y in sy + 1..sy + height {
            self.set(sx, y, fg, bg, to_cp437('║'));
            self.set(sx + width, y, fg, bg, to_cp437('║'));
        }
    }

    /// Get a cell for testing/inspection. Fancy consoles report the topmost
    /// tile at the cell (highest z-order, latest draw among ties), or the
    /// transparent default where no tile was pushed.
    pub fn get(&self, x: i32, y: i32) -> Option<Cell> {
        self.try_at(x, y)?;
        if self.fancy {
            let mut top: Option<&SparseTile> = None;
            for tile in &self.sparse_tiles {
                if tile.x == x && tile.y == y {
                    match top {
                        Some(t) if t.z_order > tile.z_order => {}
                        _ => top = Some(tile),
                    }
                }
            }
            Some(match top {
                Some(t) => Cell { glyph: t.glyph, fg: t.fg, bg: t.bg },
                None => Self::default_cell(true),
            })
        } else {
            let idx = (y * self.width + x) as usize;
            Some(self.tiles[idx])
        }
    }

    /// Console width in cells.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Console height in cells.
    pub fn height(&self) -> i32 {
        self.height
    }
}

// ─── Context object (interface based on RLTK's BTerm) ────────────────────────

/// The main context object passed to [`GameState::tick`]. Holds the consoles,
/// input state, and display metrics. Interface based on RLTK's context object
/// (bracket-terminal's `BTerm`); reimplemented from scratch.
///
/// All drawing methods are generic over coordinate types (`TryInto<i32>`) and
/// color types (`Into<RGBA>`), matching the RLTK API so call sites don't need
/// explicit casts.
pub struct Context {
    pub width_pixels: u32,
    pub height_pixels: u32,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub active_console: usize,
    pub key: Option<VirtualKeyCode>,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub quitting: bool,
    consoles: Vec<Console>,
    /// Loaded font atlases; consoles reference these by index.
    fonts: Vec<Font>,
    /// On-screen size of one console cell, in pixels.
    tile_width: u32,
    tile_height: u32,
    /// Software framebuffer (0xRRGGBB per pixel), presented to the window each frame.
    framebuffer: Vec<u32>,
    /// The window, once the builder has opened it. `None` for headless contexts.
    window: Option<minifb::Window>,
    /// Window title; kept so the window can be recreated (e.g. when toggling
    /// fullscreen or changing the font size at runtime).
    title: String,
    /// Whether the window is fullscreen (borderless, pinned to the screen's
    /// top-left corner).
    fullscreen: bool,
    /// Base path used to load resources (fonts) at runtime.
    resource_path: String,
    /// Window size in console cells; the pixel size is this times the tile size.
    console_width: u32,
    console_height: u32,
}

impl Context {
    /// Create a context for testing (the builder is the normal path).
    pub fn new_for_test(console_width: i32, console_height: i32, num_consoles: usize, fancy_indices: &[usize]) -> Self {
        let consoles = (0..num_consoles)
            .map(|i| Console::new(console_width, console_height, fancy_indices.contains(&i)))
            .collect();
        let width_pixels = (console_width * 8) as u32;
        let height_pixels = (console_height * 8) as u32;
        Self {
            width_pixels,
            height_pixels,
            fps: 0.0,
            frame_time_ms: 0.0,
            active_console: 0,
            key: None,
            shift: false,
            control: false,
            alt: false,
            quitting: false,
            consoles,
            fonts: Vec::new(),
            tile_width: 8,
            tile_height: 8,
            framebuffer: vec![0; (width_pixels * height_pixels) as usize],
            window: None,
            title: "Test".to_string(),
            fullscreen: false,
            resource_path: "resources".to_string(),
            console_width: console_width as u32,
            console_height: console_height as u32,
        }
    }

    /// Number of consoles.
    pub fn num_consoles(&self) -> usize {
        self.consoles.len()
    }

    /// Access a console by index (for testing/inspection).
    pub fn console(&self, idx: usize) -> &Console {
        &self.consoles[idx]
    }

    /// Switch the active console for subsequent drawing calls.
    pub fn set_active_console(&mut self, id: usize) {
        self.active_console = id;
    }

    /// Clear the active console.
    pub fn cls(&mut self) {
        self.consoles[self.active_console].cls();
    }

    /// Set a single cell on the active console.
    pub fn set<COLOR, COLOR2, GLYPH, X, Y>(&mut self, x: X, y: Y, fg: COLOR, bg: COLOR2, glyph: GLYPH)
    where
        COLOR: Into<RGBA>,
        COLOR2: Into<RGBA>,
        GLYPH: TryInto<FontCharType>,
        X: TryInto<i32>,
        Y: TryInto<i32>,
    {
        let x = x.try_into().ok().expect("set: x must be i32-convertible");
        let y = y.try_into().ok().expect("set: y must be i32-convertible");
        let glyph = glyph.try_into().ok().expect("set: glyph must be FontCharType-convertible");
        self.consoles[self.active_console].set(x, y, fg.into(), bg.into(), glyph);
    }

    /// Print colored text on the active console.
    pub fn print_color<S, COLOR, COLOR2, X, Y>(&mut self, x: X, y: Y, fg: COLOR, bg: COLOR2, output: S)
    where
        S: ToString,
        COLOR: Into<RGBA>,
        COLOR2: Into<RGBA>,
        X: TryInto<i32>,
        Y: TryInto<i32>,
    {
        let x = x.try_into().ok().expect("print_color: x must be i32-convertible");
        let y = y.try_into().ok().expect("print_color: y must be i32-convertible");
        self.consoles[self.active_console].print_color(x, y, fg.into(), bg.into(), &output.to_string());
    }

    /// Print colored text centered horizontally on the active console.
    pub fn print_color_centered<S, COLOR, COLOR2, Y>(&mut self, y: Y, fg: COLOR, bg: COLOR2, text: S)
    where
        S: ToString,
        COLOR: Into<RGBA>,
        COLOR2: Into<RGBA>,
        Y: TryInto<i32>,
    {
        let y = y.try_into().ok().expect("print_color_centered: y must be i32-convertible");
        self.consoles[self.active_console].print_color_centered(y, fg.into(), bg.into(), &text.to_string());
    }

    /// Draw a filled box with a single-line border on the active console.
    pub fn draw_box<COLOR, COLOR2, X, Y, W, H>(&mut self, x: X, y: Y, width: W, height: H, fg: COLOR, bg: COLOR2)
    where
        COLOR: Into<RGBA>,
        COLOR2: Into<RGBA>,
        X: TryInto<i32>,
        Y: TryInto<i32>,
        W: TryInto<i32>,
        H: TryInto<i32>,
    {
        let x = x.try_into().ok().expect("draw_box: x must be i32-convertible");
        let y = y.try_into().ok().expect("draw_box: y must be i32-convertible");
        let w = width.try_into().ok().expect("draw_box: width must be i32-convertible");
        let h = height.try_into().ok().expect("draw_box: height must be i32-convertible");
        self.consoles[self.active_console].draw_box(x, y, w, h, fg.into(), bg.into());
    }

    /// Draw a hollow box with a double-line border on the active console.
    pub fn draw_hollow_box_double<COLOR, COLOR2, X, Y, W, H>(&mut self, x: X, y: Y, width: W, height: H, fg: COLOR, bg: COLOR2)
    where
        COLOR: Into<RGBA>,
        COLOR2: Into<RGBA>,
        X: TryInto<i32>,
        Y: TryInto<i32>,
        W: TryInto<i32>,
        H: TryInto<i32>,
    {
        let x = x.try_into().ok().expect("draw_hollow_box_double: x must be i32-convertible");
        let y = y.try_into().ok().expect("draw_hollow_box_double: y must be i32-convertible");
        let w = width.try_into().ok().expect("draw_hollow_box_double: width must be i32-convertible");
        let h = height.try_into().ok().expect("draw_hollow_box_double: height must be i32-convertible");
        self.consoles[self.active_console].draw_hollow_box_double(x, y, w, h, fg.into(), bg.into());
    }

    /// Set a cell with explicit RGBA colors (alpha blending) on the active console.
    ///
    /// The Y coordinate is adjusted by -1 to compensate for RLTK's flexible-console
    /// Y inversion (`height - y` vs `height - 1 - y`). Game code adds +1 to Y when
    /// calling `set_fancy`, so the net cell Y is `position.y - 1`.
    ///
    /// On fancy consoles this pushes a tile that blends over earlier tiles in
    /// z-order (stable for ties) at render time, matching RLTK's `FlexiConsole`.
    #[allow(clippy::too_many_arguments)] // argument list mirrors the original console API
    pub fn set_fancy<COLOR, COLOR2, GLYPH, ANGLE>(
        &mut self,
        position: PointF,
        z_order: i32,
        rotation: ANGLE,
        _scale: PointF,
        fg: COLOR,
        bg: COLOR2,
        glyph: GLYPH,
    )
    where
        COLOR: Into<RGBA>,
        COLOR2: Into<RGBA>,
        GLYPH: TryInto<FontCharType>,
        ANGLE: Into<Radians>,
    {
        let _ = rotation.into();
        let x = position.x as i32;
        let y = position.y as i32 - 1;
        let glyph = glyph.try_into().ok().expect("set_fancy: glyph must be FontCharType-convertible");
        self.consoles[self.active_console].set_fancy(x, y, z_order, fg.into(), bg.into(), glyph);
    }
}

// ─── GameState trait ─────────────────────────────────────────────────────────

/// Implement this trait on your state struct so the engine knows what to call each tick.
/// Interface based on RLTK's `GameState`.
pub trait GameState {
    fn tick(&mut self, ctx: &mut Context);
}

// ─── Embedded resources ──────────────────────────────────────────────────────

/// Declares an embedded resource: a `const` holding a file's bytes, baked into the
/// executable. The path is relative to the file invoking the macro.
/// Interface based on RLTK's `embedded_resource!`.
#[macro_export]
macro_rules! embedded_resource {
    ($resource_name:ident, $filename:expr) => {
        const $resource_name: &'static [u8] = include_bytes!($filename);
    };
}

/// Registers an embedded resource under a lookup path, so resource loaders can find
/// it by name. The resource must be declared with `embedded_resource!`.
/// Interface based on RLTK's `link_resource!`.
#[macro_export]
macro_rules! link_resource {
    ($resource_name:ident, $filename:expr) => {
        $crate::register_resource($filename, $resource_name);
    };
}

fn resource_registry() -> &'static Mutex<HashMap<String, &'static [u8]>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, &'static [u8]>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Normalizes a resource path to forward slashes so lookups are platform-independent.
fn normalize_resource_path(path: &str) -> String {
    if std::path::MAIN_SEPARATOR == '/' {
        path.to_string()
    } else {
        path.replace(std::path::MAIN_SEPARATOR, "/")
    }
}

/// Registers `bytes` as an embedded resource, retrievable via `embedded_resource(path)`.
pub fn register_resource(path: &str, bytes: &'static [u8]) {
    resource_registry()
        .lock()
        .unwrap()
        .insert(normalize_resource_path(path), bytes);
}

/// Returns the bytes of a previously registered embedded resource, if any.
pub fn embedded_resource(path: &str) -> Option<&'static [u8]> {
    resource_registry()
        .lock()
        .unwrap()
        .get(&normalize_resource_path(path))
        .copied()
}

/// Loads a resource by path: embedded bytes first, then the filesystem.
pub fn load_resource(path: &str) -> std::io::Result<Vec<u8>> {
    if let Some(bytes) = embedded_resource(path) {
        return Ok(bytes.to_vec());
    }
    std::fs::read(path)
}

// ─── REXPaint (.xp) files ────────────────────────────────────────────────────

/// Parser for REXPaint `.xp` image files. Interface based on RLTK's `rex`.
///
/// File layout (after gzip decompression, all integers little-endian):
///   i32  version (-1)
///   u32  layer count
///   per layer: u32 width, u32 height, then `width * height` cells stored
///   column-major (every row of column 0, then every row of column 1, ...)
///   per cell: u32 glyph, u8 fg_r, u8 fg_g, u8 fg_b, u8 bg_r, u8 bg_g, u8 bg_b
pub mod rex {
    use std::io::{self, Read, Write};

    /// 8-bit RGB color as stored in a `.xp` file. Interface based on RLTK's `rex::XpColor`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct XpColor {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    impl XpColor {
        /// Deepest black.
        pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
        /// Hot pink marks a transparent cell background.
        pub const TRANSPARENT: Self = Self { r: 255, g: 0, b: 255 };

        pub const fn new(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b }
        }

        /// Whether this color is treated as transparent.
        pub fn is_transparent(self) -> bool {
            self == Self::TRANSPARENT
        }
    }

    /// One character cell of a layer. Interface based on RLTK's `rex::XpCell`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct XpCell {
        /// Glyph code point (CP437 in practice).
        pub ch: u32,
        pub fg: XpColor,
        pub bg: XpColor,
    }

    /// A single layer of a REXPaint image. Interface based on RLTK's `rex::XpLayer`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct XpLayer {
        pub width: usize,
        pub height: usize,
        /// Cells in column-major order: (x, y) is at index `x * height + y`.
        pub cells: Vec<XpCell>,
    }

    impl XpLayer {
        /// Creates an empty (black foreground/background, glyph 0) layer.
        pub fn new(width: usize, height: usize) -> XpLayer {
            XpLayer {
                width,
                height,
                cells: vec![
                    XpCell {
                        ch: 0,
                        fg: XpColor::BLACK,
                        bg: XpColor::BLACK,
                    };
                    width * height
                ],
            }
        }

        /// Gets the cell at (x, y), or `None` if out of range.
        pub fn get(&self, x: usize, y: usize) -> Option<&XpCell> {
            if x < self.width && y < self.height {
                Some(&self.cells[x * self.height + y])
            } else {
                None
            }
        }

        /// Gets a mutable reference to the cell at (x, y), or `None` if out of range.
        pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut XpCell> {
            if x < self.width && y < self.height {
                Some(&mut self.cells[x * self.height + y])
            } else {
                None
            }
        }
    }

    /// A REXPaint image: a stack of layers. Interface based on RLTK's `rex::XpFile`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct XpFile {
        pub version: i32,
        pub layers: Vec<XpLayer>,
    }

    fn read_u8<R: Read>(r: &mut R) -> io::Result<u8> {
        let mut b = [0u8; 1];
        r.read_exact(&mut b)?;
        Ok(b[0])
    }

    fn read_u32_le<R: Read>(r: &mut R) -> io::Result<u32> {
        let mut b = [0u8; 4];
        r.read_exact(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }

    fn read_i32_le<R: Read>(r: &mut R) -> io::Result<i32> {
        let mut b = [0u8; 4];
        r.read_exact(&mut b)?;
        Ok(i32::from_le_bytes(b))
    }

    impl XpFile {
        /// Creates a new image with one empty layer of the given size.
        pub fn new(width: usize, height: usize) -> XpFile {
            XpFile {
                version: -1,
                layers: vec![XpLayer::new(width, height)],
            }
        }

        /// Loads a `.xp` file from the resource system: embedded bytes first,
        /// then the filesystem at `path`.
        pub fn from_resource(path: &str) -> io::Result<XpFile> {
            let bytes = crate::load_resource(path)?;
            XpFile::from_bytes(&bytes)
        }

        /// Parses a gzip-compressed `.xp` file from raw bytes.
        pub fn from_bytes(bytes: &[u8]) -> io::Result<XpFile> {
            let mut decoder = flate2::read::GzDecoder::new(bytes);
            XpFile::read(&mut decoder)
        }

        /// Reads a `.xp` image from a decompressed byte stream.
        pub fn read<R: Read>(r: &mut R) -> io::Result<XpFile> {
            let version = read_i32_le(r)?;
            let num_layers = read_u32_le(r)?;
            let mut layers = Vec::with_capacity(num_layers as usize);
            for _ in 0..num_layers {
                let width = read_u32_le(r)? as usize;
                let height = read_u32_le(r)? as usize;
                let mut cells = Vec::with_capacity(width * height);
                for _ in 0..width {
                    // Column-major: x is the outer loop, y the inner one.
                    for _ in 0..height {
                        let ch = read_u32_le(r)?;
                        let fg = XpColor::new(read_u8(r)?, read_u8(r)?, read_u8(r)?);
                        let bg = XpColor::new(read_u8(r)?, read_u8(r)?, read_u8(r)?);
                        cells.push(XpCell { ch, fg, bg });
                    }
                }
                layers.push(XpLayer {
                    width,
                    height,
                    cells,
                });
            }
            Ok(XpFile { version, layers })
        }

        /// Writes the image to a byte stream (uncompressed). Inverse of
        /// [`XpFile::read`].
        pub fn write<W: Write>(&self, w: &mut W) -> io::Result<()> {
            w.write_all(&self.version.to_le_bytes())?;
            w.write_all(&(self.layers.len() as u32).to_le_bytes())?;
            for layer in &self.layers {
                w.write_all(&(layer.width as u32).to_le_bytes())?;
                w.write_all(&(layer.height as u32).to_le_bytes())?;
                for x in 0..layer.width {
                    // Column-major: x is the outer loop, y the inner one.
                    for y in 0..layer.height {
                        let cell = &layer.cells[x * layer.height + y];
                        w.write_all(&cell.ch.to_le_bytes())?;
                        w.write_all(&[cell.fg.r, cell.fg.g, cell.fg.b])?;
                        w.write_all(&[cell.bg.r, cell.bg.g, cell.bg.b])?;
                    }
                }
            }
            Ok(())
        }

        /// Serializes the image as gzip-compressed `.xp` bytes, ready to be
        /// written to a file. Inverse of [`XpFile::from_bytes`].
        pub fn to_gzipped_bytes(&self) -> io::Result<Vec<u8>> {
            let mut raw = Vec::new();
            self.write(&mut raw)?;
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&raw)?;
            encoder.finish()
        }
    }
}

// ─── Windowing and rendering backend (minifb) ────────────────────────────────

/// A font atlas loaded from a CP437 PNG: a 16x16 grid of glyphs stored as
/// tightly-packed RGBA8 pixels.
#[derive(Debug)]
struct Font {
    /// RGBA8 pixels, `16 * glyph_width` pixels wide.
    pixels: Vec<u8>,
    glyph_width: u32,
    glyph_height: u32,
}

impl Font {
    /// Decodes a font PNG and verifies it is a 16x16 grid of `glyph_width` x
    /// `glyph_height` glyphs.
    fn from_png_bytes(bytes: &[u8], glyph_width: u32, glyph_height: u32) -> Result<Font, BError> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| BError::new(format!("Unable to decode font image: {}", e)))?
            .to_rgba8();
        let expected_w = 16 * glyph_width;
        let expected_h = 16 * glyph_height;
        if img.width() != expected_w || img.height() != expected_h {
            return Err(BError::new(format!(
                "Font image is {}x{} px, expected {}x{} (16x16 grid of {}x{} px glyphs)",
                img.width(),
                img.height(),
                expected_w,
                expected_h,
                glyph_width,
                glyph_height
            )));
        }
        Ok(Font {
            pixels: img.into_raw(),
            glyph_width,
            glyph_height,
        })
    }

    /// Returns the RGBA texel of `glyph` at glyph-local coordinates (px, py).
    /// See [`texel_visible`] for how texels become glyph pixels when rendering.
    fn pixel(&self, glyph: FontCharType, px: u32, py: u32) -> [u8; 4] {
        let tx = (glyph % 16) as u32;
        let ty = (glyph / 16) as u32;
        let x = tx * self.glyph_width + px;
        let y = ty * self.glyph_height + py;
        let stride = 16 * self.glyph_width;
        let off = ((y * stride + x) * 4) as usize;
        [
            self.pixels[off],
            self.pixels[off + 1],
            self.pixels[off + 2],
            self.pixels[off + 3],
        ]
    }
}

/// Finds a font's bytes: embedded resources first (exact path, then
/// `resource_path/path`), then the filesystem (`resource_path/path`, then `path`).
fn load_font_bytes(path: &str, resource_path: &str) -> Option<Vec<u8>> {
    let prefixed = format!("{}/{}", resource_path.trim_end_matches('/'), path);
    embedded_resource(path)
        .map(|b| b.to_vec())
        .or_else(|| embedded_resource(&prefixed).map(|b| b.to_vec()))
        .or_else(|| std::fs::read(&prefixed).ok())
        .or_else(|| std::fs::read(path).ok())
}

/// Maps a minifb key to the engine's `VirtualKeyCode`. Returns `None` for keys
/// with no equivalent (the OS "super" keys).
fn map_key(key: minifb::Key) -> Option<VirtualKeyCode> {
    use minifb::Key as K;
    use VirtualKeyCode as V;
    Some(match key {
        K::Key0 => V::Key0,
        K::Key1 => V::Key1,
        K::Key2 => V::Key2,
        K::Key3 => V::Key3,
        K::Key4 => V::Key4,
        K::Key5 => V::Key5,
        K::Key6 => V::Key6,
        K::Key7 => V::Key7,
        K::Key8 => V::Key8,
        K::Key9 => V::Key9,
        K::A => V::A,
        K::B => V::B,
        K::C => V::C,
        K::D => V::D,
        K::E => V::E,
        K::F => V::F,
        K::G => V::G,
        K::H => V::H,
        K::I => V::I,
        K::J => V::J,
        K::K => V::K,
        K::L => V::L,
        K::M => V::M,
        K::N => V::N,
        K::O => V::O,
        K::P => V::P,
        K::Q => V::Q,
        K::R => V::R,
        K::S => V::S,
        K::T => V::T,
        K::U => V::U,
        K::V => V::V,
        K::W => V::W,
        K::X => V::X,
        K::Y => V::Y,
        K::Z => V::Z,
        K::F1 => V::F1,
        K::F2 => V::F2,
        K::F3 => V::F3,
        K::F4 => V::F4,
        K::F5 => V::F5,
        K::F6 => V::F6,
        K::F7 => V::F7,
        K::F8 => V::F8,
        K::F9 => V::F9,
        K::F10 => V::F10,
        K::F11 => V::F11,
        K::F12 => V::F12,
        K::F13 => V::F13,
        K::F14 => V::F14,
        K::F15 => V::F15,
        K::Down => V::Down,
        K::Left => V::Left,
        K::Right => V::Right,
        K::Up => V::Up,
        K::Apostrophe => V::Apostrophe,
        K::Backquote => V::Grave,
        K::Backslash => V::Backslash,
        K::Comma => V::Comma,
        K::Equal => V::Equals,
        K::LeftBracket => V::LBracket,
        K::Minus => V::Minus,
        K::Period => V::Period,
        K::RightBracket => V::RBracket,
        K::Semicolon => V::Semicolon,
        K::Slash => V::Slash,
        K::Backspace => V::Back,
        K::Delete => V::Delete,
        K::End => V::End,
        K::Enter => V::Return,
        K::Escape => V::Escape,
        K::Home => V::Home,
        K::Insert => V::Insert,
        K::Menu => V::Apps,
        K::PageDown => V::PageDown,
        K::PageUp => V::PageUp,
        K::Pause => V::Pause,
        K::Space => V::Space,
        K::Tab => V::Tab,
        K::NumLock => V::NumLock,
        K::CapsLock => V::CapsLock,
        K::ScrollLock => V::ScrollLock,
        K::LeftShift => V::LShift,
        K::RightShift => V::RShift,
        K::LeftCtrl => V::LControl,
        K::RightCtrl => V::RControl,
        K::NumPad0 => V::Numpad0,
        K::NumPad1 => V::Numpad1,
        K::NumPad2 => V::Numpad2,
        K::NumPad3 => V::Numpad3,
        K::NumPad4 => V::Numpad4,
        K::NumPad5 => V::Numpad5,
        K::NumPad6 => V::Numpad6,
        K::NumPad7 => V::Numpad7,
        K::NumPad8 => V::Numpad8,
        K::NumPad9 => V::Numpad9,
        K::NumPadDot => V::NumpadDecimal,
        K::NumPadSlash => V::NumpadDivide,
        K::NumPadAsterisk => V::NumpadMultiply,
        K::NumPadMinus => V::NumpadSubtract,
        K::NumPadPlus => V::NumpadAdd,
        K::NumPadEnter => V::NumpadEnter,
        K::LeftAlt => V::LAlt,
        K::RightAlt => V::RAlt,
        _ => return None,
    })
}

/// Whether a font texel counts as a glyph pixel, matching RLTK's shader
/// (bracket-terminal's): visible when it is neither near-black (all channels
/// <= 0.1) nor nearly transparent. Font sheets without an alpha channel
/// (white glyphs on an opaque black background) rely on the brightness test.
fn texel_visible(texel: [u8; 4]) -> bool {
    (texel[0] > 25 || texel[1] > 25 || texel[2] > 25) && texel[3] > 25
}

/// Packs 8-bit RGB into minifb's 0xRRGGBB framebuffer format.
fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Clamps an f32 color component (0.0..=1.0) to a u8.
fn color_to_u8(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Blends a source color over a destination framebuffer pixel (source-over,
/// non-premultiplied alpha).
fn blend_over(dst: u32, sr: u8, sg: u8, sb: u8, alpha: f32) -> u32 {
    if alpha <= 0.0 {
        return dst;
    }
    if alpha >= 1.0 {
        return pack_rgb(sr, sg, sb);
    }
    let dr = ((dst >> 16) & 0xFF) as f32;
    let dg = ((dst >> 8) & 0xFF) as f32;
    let db = (dst & 0xFF) as f32;
    let inv = 1.0 - alpha;
    let r = (sr as f32 * alpha + dr * inv).round() as u32;
    let g = (sg as f32 * alpha + dg * inv).round() as u32;
    let b = (sb as f32 * alpha + db * inv).round() as u32;
    (r << 16) | (g << 8) | b
}

/// Renders one console cell into the framebuffer at cell position (cx, cy),
/// alpha-blending over whatever is already there. Matches RLTK's shader
/// (bracket-terminal's): a font texel counts as a glyph pixel when it is
/// neither near-black nor nearly transparent; glyph pixels take the foreground
/// color modulated by the texel (alpha = fg.a * texel.a), all other pixels
/// take the background color (alpha = bg.a). Nearest-neighbor scaled to the
/// tile size.
#[allow(clippy::too_many_arguments)] // hot internal path; grouping would obscure it
fn render_cell(
    framebuffer: &mut [u32],
    fb_width: usize,
    fb_height: usize,
    tile_width: u32,
    tile_height: u32,
    font: &Font,
    cell: &Cell,
    cx: i32,
    cy: i32,
) {
    if cx < 0 || cy < 0 {
        return;
    }
    let (gw, gh) = (font.glyph_width, font.glyph_height);
    let px0 = cx as u32 * tile_width;
    let py0 = cy as u32 * tile_height;
    if px0 >= fb_width as u32 || py0 >= fb_height as u32 {
        return;
    }
    for gy in 0..tile_height {
        if py0 + gy >= fb_height as u32 {
            break;
        }
        let sy = gy * gh / tile_height;
        for gx in 0..tile_width {
            if px0 + gx >= fb_width as u32 {
                break;
            }
            let sx = gx * gw / tile_width;
            let [tr, tg, tb, ta] = font.pixel(cell.glyph, sx, sy);
            let glyph_visible = texel_visible([tr, tg, tb, ta]);
            let idx = (py0 + gy) as usize * fb_width + (px0 + gx) as usize;
            if glyph_visible {
                let alpha = cell.fg.a * (ta as f32 / 255.0);
                if alpha <= 0.0 {
                    continue;
                }
                let (fr, fgc, fb) = (
                    color_to_u8(cell.fg.r * (tr as f32 / 255.0)),
                    color_to_u8(cell.fg.g * (tg as f32 / 255.0)),
                    color_to_u8(cell.fg.b * (tb as f32 / 255.0)),
                );
                framebuffer[idx] = blend_over(framebuffer[idx], fr, fgc, fb, alpha);
            } else if cell.bg.a > 0.0 {
                let (br, bgc, bb) = (
                    color_to_u8(cell.bg.r),
                    color_to_u8(cell.bg.g),
                    color_to_u8(cell.bg.b),
                );
                framebuffer[idx] = blend_over(framebuffer[idx], br, bgc, bb, cell.bg.a);
            }
        }
    }
}

impl Context {
    /// Renders all consoles into the framebuffer, in order. Earlier consoles are
    /// drawn first; later consoles blend over them using their cells' alpha.
    /// Glyphs are tinted with the cell foreground color, masked by the font
    /// pixel's alpha, and nearest-neighbor scaled to the tile size.
    ///
    /// Fancy consoles are sparse: only the tiles pushed since the last `cls`
    /// are drawn, in stable z-order (insertion order for ties), each blending
    /// over earlier tiles and the consoles below.
    fn render_framebuffer(&mut self) {
        let fb_width = self.width_pixels as usize;
        let fb_height = self.height_pixels as usize;
        for px in self.framebuffer.iter_mut() {
            *px = 0;
        }
        for ci in 0..self.consoles.len() {
            let console = &self.consoles[ci];
            let font = match self.fonts.get(console.font_index) {
                Some(f) => f,
                None => continue,
            };
            let (tw, th) = (self.tile_width, self.tile_height);
            if console.fancy {
                let mut order: Vec<usize> = (0..console.sparse_tiles.len()).collect();
                order.sort_by_key(|&i| console.sparse_tiles[i].z_order);
                for &i in &order {
                    let tile = &console.sparse_tiles[i];
                    let cell = Cell { glyph: tile.glyph, fg: tile.fg, bg: tile.bg };
                    render_cell(
                        &mut self.framebuffer, fb_width, fb_height, tw, th,
                        font, &cell, tile.x, tile.y,
                    );
                }
            } else {
                for cy in 0..console.height {
                    for cx in 0..console.width {
                        let cell = &console.tiles[(cy * console.width + cx) as usize];
                        render_cell(
                            &mut self.framebuffer, fb_width, fb_height, tw, th,
                            font, cell, cx, cy,
                        );
                    }
                }
            }
        }
    }

    /// Presents the framebuffer to the window. No-op for headless contexts.
    fn present(&mut self) {
        if let Some(window) = &mut self.window {
            let _ = window.update_with_buffer(
                &self.framebuffer,
                self.width_pixels as usize,
                self.height_pixels as usize,
            );
        }
    }

    /// Polls the window for input, updating `key` (the key pressed this frame),
    /// the modifier flags, and `quitting` (set when the window is closed).
    fn process_input(&mut self) {
        let window = match &self.window {
            Some(w) => w,
            None => return,
        };
        if !window.is_open() {
            self.quitting = true;
            return;
        }
        self.shift =
            window.is_key_down(minifb::Key::LeftShift) || window.is_key_down(minifb::Key::RightShift);
        self.control = window.is_key_down(minifb::Key::LeftCtrl)
            || window.is_key_down(minifb::Key::RightCtrl);
        self.alt =
            window.is_key_down(minifb::Key::LeftAlt) || window.is_key_down(minifb::Key::RightAlt);
        self.key = None;
        for key in window.get_keys_pressed(minifb::KeyRepeat::Yes) {
            if let Some(vk) = map_key(key) {
                self.key = Some(vk);
            }
        }
    }

    /// Creates a window for the current configuration (title, pixel size,
    /// fullscreen mode).
    fn new_window(&self, width_pixels: u32, height_pixels: u32) -> Result<minifb::Window, BError> {
        let mut window = minifb::Window::new(
            &self.title,
            width_pixels as usize,
            height_pixels as usize,
            window_options(self.fullscreen),
        )
        .map_err(|e| BError::new(format!("Unable to create window: {}", e)))?;
        if self.fullscreen {
            // Pin the decoration-less window to the top-left corner so it
            // covers the screen.
            window.set_position(0, 0);
        }
        window.set_target_fps(60);
        Ok(window)
    }

    /// Opens the window (used by the builder).
    fn open_window(&mut self) -> Result<(), BError> {
        let window = self.new_window(self.width_pixels, self.height_pixels)?;
        self.window = Some(window);
        Ok(())
    }

    /// Toggles fullscreen without restarting: recreates the window with or
    /// without decorations, pinned to the screen's top-left corner when
    /// fullscreen. On a headless context only the flag is stored.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), BError> {
        if fullscreen == self.fullscreen {
            return Ok(());
        }
        let old = self.fullscreen;
        self.fullscreen = fullscreen;
        if self.window.is_some() {
            match self.new_window(self.width_pixels, self.height_pixels) {
                Ok(window) => self.window = Some(window),
                Err(e) => {
                    self.fullscreen = old;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Replaces the font atlas used by all consoles and resizes the window to
    /// the new tile size, without restarting. The tile size may differ from
    /// the glyph size (glyphs are scaled when rendering). Console cell grids
    /// are unaffected (their dimensions are in cells) and the framebuffer is
    /// cleared.
    pub fn set_font(
        &mut self,
        path: &str,
        glyph_width: u32,
        glyph_height: u32,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<(), BError> {
        let bytes = load_font_bytes(path, &self.resource_path)
            .ok_or_else(|| BError::new(format!("Unable to load font: {}", path)))?;
        let font = Font::from_png_bytes(&bytes, glyph_width, glyph_height)?;
        let width_pixels = self.console_width * tile_width;
        let height_pixels = self.console_height * tile_height;
        // minifb windows are fixed-size: build the replacement window first so
        // a failure leaves the context unchanged.
        let window = if self.window.is_some() {
            Some(self.new_window(width_pixels, height_pixels)?)
        } else {
            None
        };
        self.fonts = vec![font];
        for console in &mut self.consoles {
            console.font_index = 0;
        }
        self.tile_width = tile_width;
        self.tile_height = tile_height;
        self.width_pixels = width_pixels;
        self.height_pixels = height_pixels;
        self.framebuffer = vec![0; (width_pixels * height_pixels) as usize];
        self.window = window;
        Ok(())
    }
}

struct BuilderFont {
    path: String,
    width: u32,
    height: u32,
}

struct BuilderConsole {
    width: i32,
    height: i32,
    fancy: bool,
    font: String,
}

/// Builder for [`Context`]. Interface based on RLTK's context builder.
pub struct ContextBuilder {
    consoles: Vec<BuilderConsole>,
    fonts: Vec<BuilderFont>,
    /// Console dimensions in characters; used to size the window.
    width: u32,
    height: u32,
    /// Tile size in pixels; defaults to the first font's glyph size.
    tile_width: Option<u32>,
    tile_height: Option<u32>,
    title: String,
    resource_path: String,
    fullscreen: bool,
}

/// Window options for the given fullscreen request. minifb has no
/// exclusive-fullscreen mode; fullscreen is emulated with a decoration-less
/// window pinned to the screen's top-left corner (see [`ContextBuilder::build`]).
fn window_options(fullscreen: bool) -> minifb::WindowOptions {
    let mut options = minifb::WindowOptions { resize: false, ..Default::default() };
    if fullscreen {
        // No borders, caption bar, or window controls. On Windows the key is
        // `none`: minifb then creates a WS_POPUP window, the only style with
        // no non-client area at all. (`borderless` alone still yields
        // WS_OVERLAPPED, which Win32 draws with a caption bar; `title =
        // false` alone keeps the border.) The other two flags cover the X11
        // and macOS backends, which ignore `none`.
        options.none = true;
        options.borderless = true;
        options.title = false;
        options.scale = minifb::Scale::FitScreen;
    }
    options
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            consoles: Vec::new(),
            fonts: Vec::new(),
            width: 80,
            height: 50,
            tile_width: None,
            tile_height: None,
            title: "Game".to_string(),
            resource_path: "resources".to_string(),
            fullscreen: false,
        }
    }

    /// Adds a simple (opaque) console layer. Consoles are rendered in the order added.
    pub fn with_simple_console<X, Y, S>(mut self, width: X, height: Y, font: S) -> Self
    where
        X: TryInto<i32>,
        Y: TryInto<i32>,
        S: ToString,
    {
        self.consoles.push(BuilderConsole {
            width: width.try_into().ok().expect("console width must be i32-convertible"),
            height: height.try_into().ok().expect("console height must be i32-convertible"),
            fancy: false,
            font: font.to_string(),
        });
        self
    }

    /// Adds a fancy (alpha-blended) console layer, rendered over the consoles
    /// added before it.
    pub fn with_fancy_console<X, Y, S>(mut self, width: X, height: Y, font: S) -> Self
    where
        X: TryInto<i32>,
        Y: TryInto<i32>,
        S: ToString,
    {
        self.consoles.push(BuilderConsole {
            width: width.try_into().ok().expect("console width must be i32-convertible"),
            height: height.try_into().ok().expect("console height must be i32-convertible"),
            fancy: true,
            font: font.to_string(),
        });
        self
    }

    /// Sets the console dimensions in characters, used to size the window.
    pub fn with_dimensions<T>(mut self, width: T, height: T) -> Self
    where
        T: TryInto<u32>,
    {
        self.width = width.try_into().ok().expect("Must be convertible to a u32");
        self.height = height.try_into().ok().expect("Must be convertible to a u32");
        self
    }

    /// Overrides the tile size in pixels (defaults to the first font's glyph size).
    pub fn with_tile_dimensions<T>(mut self, width: T, height: T) -> Self
    where
        T: TryInto<u32>,
    {
        self.tile_width = Some(width.try_into().ok().expect("Must be convertible to a u32"));
        self.tile_height = Some(height.try_into().ok().expect("Must be convertible to a u32"));
        self
    }

    /// Sets the window title.
    pub fn with_title<S: ToString>(mut self, title: S) -> Self {
        self.title = title.to_string();
        self
    }

    /// Sets the path used to find resources (fonts) on disk.
    pub fn with_resource_path<S: ToString>(mut self, path: S) -> Self {
        self.resource_path = path.to_string();
        self
    }

    /// Registers a font: a CP437 PNG holding a 16x16 grid of `width` x `height`
    /// pixel glyphs.
    pub fn with_font<S: ToString, T>(mut self, font_path: S, width: T, height: T) -> Self
    where
        T: TryInto<u32>,
    {
        self.fonts.push(BuilderFont {
            path: font_path.to_string(),
            width: width.try_into().ok().expect("font width must be u32-convertible"),
            height: height.try_into().ok().expect("font height must be u32-convertible"),
        });
        self
    }

    /// Requests a fullscreen window. minifb has no exclusive-fullscreen mode;
    /// this opens a window without borders or controls, pinned to the
    /// top-left corner of the screen and scaled to fit it.
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Builds the context and opens the window.
    pub fn build(self) -> Result<Context, BError> {
        let mut ctx = self.build_context()?;
        ctx.open_window()?;
        Ok(ctx)
    }

    /// Builds the context without opening a window (used by `build` and by tests).
    fn build_context(self) -> Result<Context, BError> {
        let mut fonts = Vec::with_capacity(self.fonts.len());
        for bf in &self.fonts {
            let bytes = load_font_bytes(&bf.path, &self.resource_path)
                .ok_or_else(|| BError::new(format!("Unable to load font: {}", bf.path)))?;
            fonts.push(Font::from_png_bytes(&bytes, bf.width, bf.height)?);
        }
        let (tile_width, tile_height) = match (self.tile_width, self.tile_height) {
            (Some(w), Some(h)) => (w, h),
            _ => match fonts.first() {
                Some(f) => (f.glyph_width, f.glyph_height),
                None => (8, 8),
            },
        };
        let mut consoles = Vec::with_capacity(self.consoles.len());
        for bc in &self.consoles {
            let font_index = self.fonts.iter().position(|f| f.path == bc.font).unwrap_or(0);
            let mut console = Console::new(bc.width, bc.height, bc.fancy);
            console.font_index = font_index;
            consoles.push(console);
        }
        if consoles.is_empty() {
            consoles.push(Console::new(self.width as i32, self.height as i32, false));
        }
        let width_pixels = self.width * tile_width;
        let height_pixels = self.height * tile_height;
        Ok(Context {
            width_pixels,
            height_pixels,
            fps: 0.0,
            frame_time_ms: 0.0,
            active_console: 0,
            key: None,
            shift: false,
            control: false,
            alt: false,
            quitting: false,
            consoles,
            fonts,
            tile_width,
            tile_height,
            framebuffer: vec![0; (width_pixels * height_pixels) as usize],
            window: None,
            title: self.title,
            fullscreen: self.fullscreen,
            resource_path: self.resource_path,
            console_width: self.width,
            console_height: self.height,
        })
    }
}

/// Runs the main loop: poll input, tick the game state, render, present — until
/// the state or the window requests quitting. Interface based on RLTK's `main_loop`.
pub fn main_loop<GS: GameState>(mut context: Context, mut state: GS) -> Result<(), BError> {
    while !context.quitting {
        let frame_start = std::time::Instant::now();
        context.process_input();
        state.tick(&mut context);
        context.render_framebuffer();
        context.present();
        let elapsed = frame_start.elapsed().as_secs_f32();
        context.frame_time_ms = elapsed * 1000.0;
        context.fps = if elapsed > 0.0 { 1.0 / elapsed } else { 0.0 };
    }
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Point ──

    #[test]
    fn point_new_and_zero() {
        assert_eq!(Point::new(3, -7), Point { x: 3, y: -7 });
        assert_eq!(Point::zero(), Point { x: 0, y: 0 });
    }

    #[test]
    fn point_copy_and_eq() {
        let a = Point::new(1, 2);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn point_new_accepts_convertible_types() {
        assert_eq!(Point::new(3usize, 4usize), Point { x: 3, y: 4 });
        assert_eq!(Point::new(5u8, 6u8), Point { x: 5, y: 6 });
    }

    #[test]
    fn point_operator_overloads() {
        let a = Point::new(6, 9);
        let b = Point::new(1, 3);
        assert_eq!(a + b, Point::new(7, 12));
        assert_eq!(a - b, Point::new(5, 6));
        assert_eq!(a * b, Point::new(6, 27));
        assert_eq!(a / b, Point::new(6, 3));
        assert_eq!(a + 1, Point::new(7, 10));
        assert_eq!(a - 1, Point::new(5, 8));
        assert_eq!(a * 2, Point::new(12, 18));
        assert_eq!(a / 2, Point::new(3, 4));
        assert_eq!(a * 0.5, Point::new(3, 4));
        assert_eq!(a / 2.0, Point::new(3, 4));
        let mut c = a;
        c += b;
        assert_eq!(c, Point::new(7, 12));
        c -= b;
        assert_eq!(c, a);
        c *= b;
        assert_eq!(c, Point::new(6, 27));
        c /= b;
        assert_eq!(c, a);
    }

    // ── PointF ──

    #[test]
    fn pointf_new() {
        let p = PointF::new(1.5, -2.5);
        assert_eq!(p.x, 1.5);
        assert_eq!(p.y, -2.5);
    }

    // ── RGB ──

    #[test]
    fn rgb_named_colors() {
        assert_eq!(RGB::named(BLACK), RGB { r: 0.0, g: 0.0, b: 0.0 });
        assert_eq!(RGB::named(WHITE), RGB { r: 1.0, g: 1.0, b: 1.0 });
        assert_eq!(RGB::named(RED), RGB { r: 1.0, g: 0.0, b: 0.0 });
        assert_eq!(RGB::named(YELLOW), RGB { r: 1.0, g: 1.0, b: 0.0 });
    }

    #[test]
    fn rgb_from_u8() {
        let c = RGB::from_u8(255, 128, 0);
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.g - 128.0 / 255.0).abs() < 0.001);
        assert!((c.b - 0.0).abs() < 0.001);
    }

    #[test]
    fn rgb_from_f32() {
        let c = RGB::from_f32(0.5, 0.25, 0.75);
        assert_eq!(c, RGB { r: 0.5, g: 0.25, b: 0.75 });
    }

    // ── RGBA ──

    #[test]
    fn rgba_from_u8() {
        let c = RGBA::from_u8(255, 0, 0, 128);
        assert!((c.r - 1.0).abs() < 0.001);
        assert!((c.a - 128.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn rgba_from_f32() {
        let c = RGBA::from_f32(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.a, 0.4);
    }

    #[test]
    fn rgba_to_rgb() {
        let c = RGBA::from_f32(0.5, 0.6, 0.7, 0.8);
        let rgb = c.to_rgb();
        assert_eq!(rgb, RGB { r: 0.5, g: 0.6, b: 0.7 });
    }

    // ── to_cp437 ──

    #[test]
    fn cp437_ascii_identity() {
        assert_eq!(to_cp437('A'), 65);
        assert_eq!(to_cp437('z'), 122);
        assert_eq!(to_cp437('0'), 48);
        assert_eq!(to_cp437(' '), 32);
        assert_eq!(to_cp437('~'), 126);
        assert_eq!(to_cp437('!'), 33);
        assert_eq!(to_cp437('.'), 46);
        assert_eq!(to_cp437('_'), 95);
        assert_eq!(to_cp437('-'), 45);
        assert_eq!(to_cp437('#'), 35);
        assert_eq!(to_cp437('\\'), 92);
        assert_eq!(to_cp437('/'), 47);
    }

    #[test]
    fn cp437_box_drawing() {
        assert_eq!(to_cp437('─'), 196);
        assert_eq!(to_cp437('│'), 179);
        assert_eq!(to_cp437('┌'), 218);
        assert_eq!(to_cp437('┐'), 191);
        assert_eq!(to_cp437('└'), 192);
        assert_eq!(to_cp437('┘'), 217);
        assert_eq!(to_cp437('├'), 195);
        assert_eq!(to_cp437('┤'), 180);
        assert_eq!(to_cp437('┬'), 194);
        assert_eq!(to_cp437('┴'), 193);
        assert_eq!(to_cp437('┼'), 197);
    }

    #[test]
    fn cp437_double_box_drawing() {
        assert_eq!(to_cp437('═'), 205);
        assert_eq!(to_cp437('║'), 186);
        assert_eq!(to_cp437('╔'), 201);
        assert_eq!(to_cp437('╗'), 187);
        assert_eq!(to_cp437('╚'), 200);
        assert_eq!(to_cp437('╝'), 188);
        assert_eq!(to_cp437('╠'), 204);
        assert_eq!(to_cp437('╣'), 185);
        assert_eq!(to_cp437('╦'), 203);
        assert_eq!(to_cp437('╩'), 202);
        assert_eq!(to_cp437('╬'), 206);
    }

    #[test]
    fn cp437_blocks() {
        assert_eq!(to_cp437('█'), 219);
        assert_eq!(to_cp437('░'), 176);
        assert_eq!(to_cp437('▒'), 177);
        assert_eq!(to_cp437('▓'), 178);
        assert_eq!(to_cp437('▀'), 223);
        assert_eq!(to_cp437('▄'), 220);
        assert_eq!(to_cp437('■'), 254);
    }

    #[test]
    fn cp437_arrows_and_symbols() {
        assert_eq!(to_cp437('►'), 16);
        assert_eq!(to_cp437('◄'), 17);
        assert_eq!(to_cp437('▲'), 30);
        assert_eq!(to_cp437('▼'), 31);
        assert_eq!(to_cp437('☺'), 1);
        assert_eq!(to_cp437('•'), 7);
        assert_eq!(to_cp437('↑'), 24);
        assert_eq!(to_cp437('↓'), 25);
        assert_eq!(to_cp437('→'), 26);
        assert_eq!(to_cp437('←'), 27);
    }

    #[test]
    fn cp437_unknown_returns_question_mark() {
        assert_eq!(to_cp437('🦀'), b'?');
        assert_eq!(to_cp437('€'), b'?');
    }

    #[test]
    fn cp437_typographic_substitutions() {
        // × has no CP437 glyph; the resolution labels ("1920×1080") render it as 'x'.
        assert_eq!(to_cp437('×'), b'x');
    }

    // ── DistanceAlg ──

    #[test]
    fn distance_pythagoras() {
        let a = Point::new(0, 0);
        let b = Point::new(3, 4);
        assert!((DistanceAlg::Pythagoras.distance2d(a, b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn distance_pythagoras_same_point() {
        let a = Point::new(5, 5);
        assert!((DistanceAlg::Pythagoras.distance2d(a, a)).abs() < 0.001);
    }

    #[test]
    fn distance_manhattan() {
        let a = Point::new(0, 0);
        let b = Point::new(3, 4);
        assert_eq!(DistanceAlg::Manhattan.distance2d(a, b), 7.0);
    }

    #[test]
    fn distance_chebyshev() {
        let a = Point::new(0, 0);
        let b = Point::new(3, 4);
        assert_eq!(DistanceAlg::Chebyshev.distance2d(a, b), 4.0);
    }

    // ── line2d / Bresenham ──

    #[test]
    fn bresenham_horizontal() {
        let line = line2d(LineAlg::Bresenham, Point::new(0, 0), Point::new(5, 0));
        assert_eq!(line, vec![
            Point::new(0, 0), Point::new(1, 0), Point::new(2, 0),
            Point::new(3, 0), Point::new(4, 0), Point::new(5, 0),
        ]);
    }

    #[test]
    fn bresenham_vertical() {
        let line = line2d(LineAlg::Bresenham, Point::new(0, 0), Point::new(0, 3));
        assert_eq!(line, vec![
            Point::new(0, 0), Point::new(0, 1), Point::new(0, 2), Point::new(0, 3),
        ]);
    }

    #[test]
    fn bresenham_diagonal() {
        let line = line2d(LineAlg::Bresenham, Point::new(0, 0), Point::new(3, 3));
        assert_eq!(line, vec![
            Point::new(0, 0), Point::new(1, 1), Point::new(2, 2), Point::new(3, 3),
        ]);
    }

    #[test]
    fn bresenham_single_point() {
        let line = line2d(LineAlg::Bresenham, Point::new(5, 5), Point::new(5, 5));
        assert_eq!(line, vec![Point::new(5, 5)]);
    }

    #[test]
    fn bresenham_shallow_slope() {
        // Slope < 1: more x steps than y steps
        let line = line2d(LineAlg::Bresenham, Point::new(0, 0), Point::new(5, 2));
        assert_eq!(line.len(), 6); // 6 points (0,0) through (5,2)
        assert_eq!(line[0], Point::new(0, 0));
        assert_eq!(line[5], Point::new(5, 2));
        // Each step advances x by 1
        for i in 1..line.len() {
            assert_eq!(line[i].x, line[i - 1].x + 1);
        }
    }

    #[test]
    fn bresenham_steep_slope() {
        // Slope > 1: more y steps than x steps
        let line = line2d(LineAlg::Bresenham, Point::new(0, 0), Point::new(2, 5));
        assert_eq!(line.len(), 6);
        assert_eq!(line[0], Point::new(0, 0));
        assert_eq!(line[5], Point::new(2, 5));
        // Each step advances y by 1
        for i in 1..line.len() {
            assert_eq!(line[i].y, line[i - 1].y + 1);
        }
    }

    #[test]
    fn bresenham_negative_direction() {
        let line = line2d(LineAlg::Bresenham, Point::new(5, 5), Point::new(0, 0));
        assert_eq!(line.len(), 6);
        assert_eq!(line[0], Point::new(5, 5));
        assert_eq!(line[5], Point::new(0, 0));
    }

    #[test]
    fn bresenham_anti_diagonal() {
        let line = line2d(LineAlg::Bresenham, Point::new(0, 3), Point::new(3, 0));
        assert_eq!(line, vec![
            Point::new(0, 3), Point::new(1, 2), Point::new(2, 1), Point::new(3, 0),
        ]);
    }

    // ── RandomNumberGenerator ──

    #[test]
    fn rng_deterministic() {
        let mut rng1 = RandomNumberGenerator::seeded(42);
        let mut rng2 = RandomNumberGenerator::seeded(42);
        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn rng_different_seeds_differ() {
        let mut rng1 = RandomNumberGenerator::seeded(1);
        let mut rng2 = RandomNumberGenerator::seeded(2);
        // Very likely to differ within a few calls
        let same = (0..10).all(|_| rng1.next_u64() == rng2.next_u64());
        assert!(!same, "different seeds should produce different sequences");
    }

    #[test]
    fn rng_range_i32() {
        let mut rng = RandomNumberGenerator::seeded(99);
        for _ in 0..1000 {
            let v = rng.range(0i32, 10i32);
            assert!((0..10).contains(&v), "range(0,10) returned {}", v);
        }
    }

    #[test]
    fn rng_range_usize() {
        let mut rng = RandomNumberGenerator::seeded(99);
        for _ in 0..1000 {
            let v = rng.range(0usize, 100usize);
            assert!(v < 100, "range(0,100) returned {}", v);
        }
    }

    #[test]
    fn rng_range_negative() {
        let mut rng = RandomNumberGenerator::seeded(7);
        for _ in 0..1000 {
            let v = rng.range(-50i32, 50i32);
            assert!((-50..50).contains(&v), "range(-50,50) returned {}", v);
        }
    }

    #[test]
    fn rng_range_single_value() {
        let mut rng = RandomNumberGenerator::seeded(1);
        assert_eq!(rng.range(5i32, 6i32), 5);
    }

    #[test]
    fn rng_zero_seed_does_not_panic() {
        let mut rng = RandomNumberGenerator::seeded(0);
        let _ = rng.range(0i32, 100i32);
    }

    // ── VirtualKeyCode ──

    #[test]
    fn virtual_key_code_equality() {
        assert_eq!(VirtualKeyCode::A, VirtualKeyCode::A);
        assert_ne!(VirtualKeyCode::A, VirtualKeyCode::B);
        assert_eq!(VirtualKeyCode::F1, VirtualKeyCode::F1);
    }

    #[test]
    fn virtual_key_code_copy() {
        let k = VirtualKeyCode::Return;
        let k2 = k; // Copy
        assert_eq!(k, k2);
    }

    // ── BError ──

    #[test]
    fn berror_display() {
        let err = BError::new("window creation failed");
        assert_eq!(format!("{}", err), "window creation failed");
    }

    // ── Radians ──

    #[test]
    fn radians_wrapper() {
        let r = Radians(0.0);
        assert_eq!(r.0, 0.0);
    }

    // ── Color conversions ──

    #[test]
    fn rgb_into_rgba() {
        let rgb = RGB::from_f32(0.5, 0.25, 0.75);
        let rgba: RGBA = rgb.into();
        assert_eq!(rgba.r, 0.5);
        assert_eq!(rgba.g, 0.25);
        assert_eq!(rgba.b, 0.75);
        assert_eq!(rgba.a, 1.0);
    }

    #[test]
    fn rgba_into_rgb() {
        let rgba = RGBA::from_f32(0.5, 0.25, 0.75, 0.5);
        let rgb: RGB = rgba.into();
        assert_eq!(rgb.r, 0.5);
        assert_eq!(rgb.g, 0.25);
        assert_eq!(rgb.b, 0.75);
    }

    // ── Console: set / get ──

    fn make_test_ctx() -> Context {
        Context::new_for_test(10, 5, 2, &[1])
    }

    #[test]
    fn console_set_and_get() {
        let mut ctx = make_test_ctx();
        let fg = RGB::named(RED);
        let bg = RGB::named(BLACK);
        ctx.set(3i32, 2i32, fg, bg, to_cp437('A'));
        let cell = ctx.console(0).get(3, 2).unwrap();
        assert_eq!(cell.glyph, to_cp437('A'));
        assert_eq!(cell.fg.r, 1.0);
        assert_eq!(cell.bg.r, 0.0);
    }

    #[test]
    fn console_set_accepts_usize_coords() {
        let mut ctx = make_test_ctx();
        ctx.set(3usize, 2usize, RGB::named(RED), RGB::named(BLACK), to_cp437('X'));
        let cell = ctx.console(0).get(3, 2).unwrap();
        assert_eq!(cell.glyph, to_cp437('X'));
    }

    #[test]
    fn console_set_out_of_bounds_ignored() {
        let mut ctx = make_test_ctx();
        ctx.set(-1i32, 0i32, RGB::named(RED), RGB::named(BLACK), to_cp437('A'));
        ctx.set(100i32, 0i32, RGB::named(RED), RGB::named(BLACK), to_cp437('A'));
        // No panic, no effect
    }

    // ── Console: print_color ──

    #[test]
    fn console_print_color() {
        let mut ctx = make_test_ctx();
        ctx.print_color(0i32, 0i32, RGB::named(WHITE), RGB::named(BLACK), "Hello");
        let console = ctx.console(0);
        assert_eq!(console.get(0, 0).unwrap().glyph, to_cp437('H'));
        assert_eq!(console.get(1, 0).unwrap().glyph, to_cp437('e'));
        assert_eq!(console.get(2, 0).unwrap().glyph, to_cp437('l'));
        assert_eq!(console.get(3, 0).unwrap().glyph, to_cp437('l'));
        assert_eq!(console.get(4, 0).unwrap().glyph, to_cp437('o'));
    }

    #[test]
    fn console_print_color_cp437_chars() {
        let mut ctx = make_test_ctx();
        ctx.print_color(0i32, 0i32, RGB::named(WHITE), RGB::named(BLACK), "►█░");
        let console = ctx.console(0);
        assert_eq!(console.get(0, 0).unwrap().glyph, to_cp437('►'));
        assert_eq!(console.get(1, 0).unwrap().glyph, to_cp437('█'));
        assert_eq!(console.get(2, 0).unwrap().glyph, to_cp437('░'));
    }

    #[test]
    fn console_print_color_substitutes_cp437_gaps() {
        // Regression test: the resolution labels ("1920×1080") use ×, which has
        // no CP437 glyph; it must render as 'x', not '?'.
        let mut ctx = make_test_ctx();
        ctx.print_color(0i32, 0i32, RGB::named(WHITE), RGB::named(BLACK), "1920×1080");
        let console = ctx.console(0);
        for (i, &glyph) in b"1920x1080".iter().enumerate() {
            assert_eq!(console.get(i as i32, 0).unwrap().glyph, glyph, "cell {}", i);
        }
    }

    #[test]
    fn console_print_color_accepts_string() {
        let mut ctx = make_test_ctx();
        let s = String::from("Test");
        ctx.print_color(0i32, 0i32, RGB::named(WHITE), RGB::named(BLACK), s);
        assert_eq!(ctx.console(0).get(0, 0).unwrap().glyph, to_cp437('T'));
    }

    // ── Console: print_color_centered ──

    #[test]
    fn console_print_color_centered() {
        let mut ctx = make_test_ctx(); // width=10
        ctx.print_color_centered(0i32, RGB::named(WHITE), RGB::named(BLACK), "Hi");
        // "Hi" is 2 chars, centered at width 10: x = 5 - 1 = 4
        let console = ctx.console(0);
        assert_eq!(console.get(4, 0).unwrap().glyph, to_cp437('H'));
        assert_eq!(console.get(5, 0).unwrap().glyph, to_cp437('i'));
    }

    #[test]
    fn console_print_color_centered_odd_length() {
        let mut ctx = make_test_ctx(); // width=10
        ctx.print_color_centered(0i32, RGB::named(WHITE), RGB::named(BLACK), "ABC");
        // "ABC" is 3 chars, centered: x = 5 - 1 = 4
        let console = ctx.console(0);
        assert_eq!(console.get(4, 0).unwrap().glyph, to_cp437('A'));
        assert_eq!(console.get(5, 0).unwrap().glyph, to_cp437('B'));
        assert_eq!(console.get(6, 0).unwrap().glyph, to_cp437('C'));
    }

    // ── Console: cls ──

    #[test]
    fn console_cls_main_is_opaque() {
        let mut ctx = make_test_ctx();
        ctx.set(0i32, 0i32, RGB::named(RED), RGB::named(BLACK), to_cp437('A'));
        ctx.cls();
        let cell = ctx.console(0).get(0, 0).unwrap();
        assert_eq!(cell.glyph, 32); // space
        assert_eq!(cell.bg.a, 1.0); // opaque
    }

    #[test]
    fn console_cls_fancy_is_transparent() {
        let mut ctx = make_test_ctx();
        ctx.set_active_console(1);
        ctx.set(0i32, 0i32, RGB::named(RED), RGB::named(BLACK), to_cp437('A'));
        ctx.cls();
        let cell = ctx.console(1).get(0, 0).unwrap();
        assert_eq!(cell.glyph, 32); // space
        assert_eq!(cell.bg.a, 0.0); // transparent
        assert_eq!(cell.fg.a, 0.0); // transparent
    }

    // ── Console: draw_box ──

    #[test]
    fn console_draw_box() {
        let mut ctx = make_test_ctx();
        let fg = RGB::named(WHITE);
        let bg = RGB::named(BLACK);
        ctx.draw_box(1i32, 1i32, 4i32, 3i32, fg, bg);
        let console = ctx.console(0);
        // Corners
        assert_eq!(console.get(1, 1).unwrap().glyph, to_cp437('┌'));
        assert_eq!(console.get(5, 1).unwrap().glyph, to_cp437('┐'));
        assert_eq!(console.get(1, 4).unwrap().glyph, to_cp437('└'));
        assert_eq!(console.get(5, 4).unwrap().glyph, to_cp437('┘'));
        // Top/bottom edges
        assert_eq!(console.get(2, 1).unwrap().glyph, to_cp437('─'));
        assert_eq!(console.get(3, 4).unwrap().glyph, to_cp437('─'));
        // Side edges
        assert_eq!(console.get(1, 2).unwrap().glyph, to_cp437('│'));
        assert_eq!(console.get(5, 3).unwrap().glyph, to_cp437('│'));
        // Interior is filled with space
        assert_eq!(console.get(3, 2).unwrap().glyph, 32);
    }

    // ── Console: draw_hollow_box_double ──

    #[test]
    fn console_draw_hollow_box_double() {
        let mut ctx = make_test_ctx();
        let fg = RGB::named(WHITE);
        let bg = RGB::named(BLACK);
        ctx.draw_hollow_box_double(1i32, 1i32, 4i32, 3i32, fg, bg);
        let console = ctx.console(0);
        // Corners
        assert_eq!(console.get(1, 1).unwrap().glyph, to_cp437('╔'));
        assert_eq!(console.get(5, 1).unwrap().glyph, to_cp437('╗'));
        assert_eq!(console.get(1, 4).unwrap().glyph, to_cp437('╚'));
        assert_eq!(console.get(5, 4).unwrap().glyph, to_cp437('╝'));
        // Edges
        assert_eq!(console.get(2, 1).unwrap().glyph, to_cp437('═'));
        assert_eq!(console.get(1, 2).unwrap().glyph, to_cp437('║'));
        // Interior is NOT filled (stays at default)
        let interior = console.get(3, 2).unwrap();
        assert_eq!(interior.glyph, 32); // default space
    }

    // ── Console: set_active_console ──

    #[test]
    fn console_switching() {
        let mut ctx = make_test_ctx();
        ctx.set(0i32, 0i32, RGB::named(RED), RGB::named(BLACK), to_cp437('A'));
        assert_eq!(ctx.console(0).get(0, 0).unwrap().glyph, to_cp437('A'));
        assert_eq!(ctx.console(1).get(0, 0).unwrap().glyph, 32); // untouched

        ctx.set_active_console(1);
        ctx.set(0i32, 0i32, RGB::named(GREEN), RGB::named(BLACK), to_cp437('B'));
        assert_eq!(ctx.console(1).get(0, 0).unwrap().glyph, to_cp437('B'));
        assert_eq!(ctx.console(0).get(0, 0).unwrap().glyph, to_cp437('A')); // unchanged
    }

    // ── Console: set_fancy (alpha overlay) ──

    #[test]
    fn console_set_fancy_alpha() {
        let mut ctx = make_test_ctx();
        ctx.set_active_console(1); // fancy console
        let fg = RGBA::from_f32(1.0, 0.0, 0.0, 0.5); // semi-transparent red
        let bg = RGBA::from_f32(0.0, 0.0, 0.0, 0.0); // transparent
        // Game passes y+1 to compensate for RLTK Y inversion; our impl subtracts 1
        ctx.set_fancy(PointF::new(3.0, 3.0), 0, Radians(0.0), PointF::new(1.0, 1.0), fg, bg, to_cp437('►'));
        let cell = ctx.console(1).get(3, 2).unwrap(); // y = 3 - 1 = 2
        assert_eq!(cell.glyph, to_cp437('►'));
        assert!((cell.fg.a - 0.5).abs() < 0.001);
    }

    #[test]
    fn console_set_fancy_aligns_with_set() {
        let mut ctx = make_test_ctx();
        ctx.set_active_console(1);
        // set at (2, 3)
        ctx.set(2i32, 3i32, RGB::named(WHITE), RGB::named(BLACK), to_cp437('A'));
        // set_fancy at PointF(2, 4) should land on the same cell (y = 4 - 1 = 3)
        ctx.set_fancy(
            PointF::new(2.0, 4.0), 0, Radians(0.0), PointF::new(1.0, 1.0),
            RGBA::from_f32(1.0, 0.0, 0.0, 0.5), RGBA::from_f32(0.0, 0.0, 0.0, 0.0),
            to_cp437('B'),
        );
        let cell = ctx.console(1).get(2, 3).unwrap();
        assert_eq!(cell.glyph, to_cp437('B')); // set_fancy lands on top of set
    }

    // ── Console: sparse (fancy) compositing ──

    #[test]
    fn fancy_get_returns_topmost_tile() {
        let mut ctx = make_test_ctx(); // console 1 is fancy
        ctx.set_active_console(1);
        ctx.set(2i32, 1i32, RGB::named(RED), RGB::named(BLACK), to_cp437('A'));
        ctx.set(2i32, 1i32, RGB::named(GREEN), RGB::named(BLACK), to_cp437('B'));
        let cell = ctx.console(1).get(2, 1).unwrap();
        assert_eq!(cell.glyph, to_cp437('B')); // latest tile is on top
        // Untouched fancy cells read as the transparent default.
        let empty = ctx.console(1).get(0, 0).unwrap();
        assert_eq!(empty.glyph, 32);
        assert_eq!(empty.bg.a, 0.0);
        assert_eq!(empty.fg.a, 0.0);
    }

    #[test]
    fn fancy_tint_blends_over_cursor() {
        // Regression test for the invisible aiming cursor: the game draws the
        // cursor (an opaque full-block glyph) and then the targeting-range tint
        // (a space glyph with a semi-transparent background) over the same
        // fancy-console cell. RLTK's fancy consoles are sparse, so the tint
        // blends over the cursor instead of replacing it.
        let mut ctx = make_render_ctx(8); // console 0 dense, console 1 fancy
        // A dark "map" tile under the cursor, on console 0.
        ctx.set(1i32, 0i32, RGB::from_u8(30, 30, 30), RGB::named(BLACK), to_cp437('█'));
        // The aiming cursor on fancy console 1.
        ctx.set_active_console(1);
        ctx.set(1i32, 0i32, RGB::named(PINK), RGB::named(BLACK), to_cp437('█'));
        // The targeting-range tint over the same cell (space glyph, alpha-0.18 bg).
        let tint = RGBA::from_f32(0.4, 0.8, 0.5, 0.18);
        let transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
        ctx.set_fancy(
            PointF::new(1.0, 1.0), 0, Radians(0.0), PointF::new(1.0, 1.0),
            transparent, tint, to_cp437(' '),
        );
        ctx.render_framebuffer();
        // Center pixel of cell (1, 0): the pink cursor must survive under the tint.
        let px = ctx.framebuffer[4 * ctx.width_pixels as usize + 12];
        let r = (px >> 16) & 0xFF;
        let g = (px >> 8) & 0xFF;
        let b = px & 0xFF;
        assert!(r > 150, "cursor red channel should survive the tint, got {:#010x}", px);
        assert!(g > 100 && b > 100, "cursor should stay pinkish, got {:#010x}", px);
    }

    #[test]
    fn fancy_later_opaque_tile_covers_earlier() {
        let mut ctx = make_render_ctx(8);
        ctx.set_active_console(1);
        ctx.set(1i32, 0i32, RGB::named(RED), RGB::named(BLACK), to_cp437('█'));
        ctx.set(1i32, 0i32, RGB::named(BLUE), RGB::named(BLACK), to_cp437('█'));
        ctx.render_framebuffer();
        let px = ctx.framebuffer[4 * ctx.width_pixels as usize + 12];
        assert_eq!(px, 0x0000FF); // blue, drawn last, covers red
    }

    #[test]
    fn fancy_z_order_dominates_draw_order() {
        let mut ctx = make_render_ctx(8);
        ctx.set_active_console(1);
        // Drawn first, but higher z-order: red wins over the later blue tile.
        ctx.set_fancy(
            PointF::new(1.0, 1.0), 1, Radians(0.0), PointF::new(1.0, 1.0),
            RGB::named(RED), RGB::named(BLACK), to_cp437('█'),
        );
        ctx.set(1i32, 0i32, RGB::named(BLUE), RGB::named(BLACK), to_cp437('█')); // z_order 0
        ctx.render_framebuffer();
        let px = ctx.framebuffer[4 * ctx.width_pixels as usize + 12];
        assert_eq!(px, 0xFF0000);
    }

    // ── Context: input state ──

    #[test]
    fn context_input_fields() {
        let mut ctx = make_test_ctx();
        assert_eq!(ctx.key, None);
        assert!(!ctx.shift);
        assert!(!ctx.control);
        assert!(!ctx.alt);

        ctx.key = Some(VirtualKeyCode::A);
        ctx.shift = true;
        assert_eq!(ctx.key, Some(VirtualKeyCode::A));
        assert!(ctx.shift);
    }

    // ── GameState trait ──

    struct DummyState {
        ticks: u32,
    }

    impl GameState for DummyState {
        fn tick(&mut self, _ctx: &mut Context) {
            self.ticks += 1;
        }
    }

    #[test]
    fn gamestate_tick() {
        let mut state = DummyState { ticks: 0 };
        let mut ctx = make_test_ctx();
        state.tick(&mut ctx);
        state.tick(&mut ctx);
        assert_eq!(state.ticks, 2);
    }

    // ── Embedded resources ──

    #[test]
    fn register_and_lookup_embedded_resource() {
        register_resource("test/engine_unit.bin", &[1, 2, 3]);
        assert_eq!(
            embedded_resource("test/engine_unit.bin"),
            Some(&[1u8, 2, 3][..])
        );
        assert_eq!(embedded_resource("test/does_not_exist.bin"), None);
    }

    #[test]
    fn resource_path_normalized_to_forward_slashes() {
        let sep = std::path::MAIN_SEPARATOR;
        let path = format!("test{}engine{}normalized.bin", sep, sep);
        register_resource(&path, b"norm");
        assert_eq!(
            embedded_resource("test/engine/normalized.bin"),
            Some(&b"norm"[..])
        );
    }

    #[test]
    fn load_resource_embedded_hit() {
        register_resource("test/engine_load_embedded.bin", b"embedded-bytes");
        assert_eq!(
            load_resource("test/engine_load_embedded.bin").unwrap(),
            b"embedded-bytes"
        );
    }

    #[test]
    fn load_resource_falls_back_to_disk() {
        let path = std::env::temp_dir().join("dieselrogue_engine_test_resource.bin");
        std::fs::write(&path, b"disk-bytes").unwrap();
        let loaded = load_resource(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded, b"disk-bytes");
    }

    #[test]
    fn load_resource_missing_is_error() {
        assert!(load_resource("test/definitely_missing_9f3b2.bin").is_err());
    }

    #[test]
    fn embedded_and_link_resource_macros() {
        crate::embedded_resource!(TEST_MACRO_RESOURCE, "../resources/title.xp");
        crate::link_resource!(TEST_MACRO_RESOURCE, "test/title_macro.xp");
        let bytes = embedded_resource("test/title_macro.xp").unwrap();
        // The embedded resource is the real title screen: a full 160x90
        // single-layer .xp (the raw form is 16 + 160*90*10 bytes, gzipped).
        let xp = rex::XpFile::from_bytes(bytes).unwrap();
        assert_eq!(xp.layers.len(), 1);
        assert_eq!((xp.layers[0].width, xp.layers[0].height), (160, 90));
    }

    // ── rex::XpColor ──

    #[test]
    fn xpcolor_basics() {
        let c = rex::XpColor::new(10, 20, 30);
        assert_eq!((c.r, c.g, c.b), (10, 20, 30));
        assert_eq!(rex::XpColor::BLACK, rex::XpColor::new(0, 0, 0));
        assert!(rex::XpColor::TRANSPARENT.is_transparent());
        assert!(!rex::XpColor::BLACK.is_transparent());
    }

    // ── rex::XpLayer ──

    #[test]
    fn xplayer_new_and_get() {
        let mut layer = rex::XpLayer::new(4, 3);
        assert_eq!(layer.width, 4);
        assert_eq!(layer.height, 3);
        assert_eq!(layer.cells.len(), 12);
        assert_eq!(layer.get(0, 0).unwrap().ch, 0);
        assert_eq!(layer.get(3, 2).unwrap().ch, 0);
        assert!(layer.get(4, 0).is_none());
        assert!(layer.get(0, 3).is_none());

        // Column-major storage: (x, y) lives at index x * height + y.
        let cell = layer.get_mut(2, 1).unwrap();
        cell.ch = 65;
        assert_eq!(layer.cells[2 * 3 + 1].ch, 65);
        assert_eq!(layer.get(2, 1).unwrap().ch, 65);
        assert!(layer.get_mut(9, 9).is_none());
    }

    // ── rex::XpFile ──

    fn cell_bytes(ch: u32, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&ch.to_le_bytes());
        v.extend_from_slice(&[fg.0, fg.1, fg.2, bg.0, bg.1, bg.2]);
        v
    }

    /// Builds the decompressed bytes of a single-layer .xp file, with cells
    /// listed in file (column-major) order.
    fn build_xp_raw(width: u32, height: u32, cells: &[Vec<u8>]) -> Vec<u8> {
        let mut raw = Vec::new();
        raw.extend_from_slice(&(-1i32).to_le_bytes());
        raw.extend_from_slice(&1u32.to_le_bytes());
        raw.extend_from_slice(&width.to_le_bytes());
        raw.extend_from_slice(&height.to_le_bytes());
        for c in cells {
            raw.extend_from_slice(c);
        }
        raw
    }

    fn gzip_compress(raw: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, raw).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn xpfile_read_column_major() {
        // 2x2 layer; cells are listed in file (column-major) order:
        // (0,0), (0,1), (1,0), (1,1).
        let raw = build_xp_raw(
            2,
            2,
            &[
                cell_bytes(65, (1, 2, 3), (4, 5, 6)),
                cell_bytes(66, (7, 8, 9), (10, 11, 12)),
                cell_bytes(67, (13, 14, 15), (16, 17, 18)),
                cell_bytes(68, (19, 20, 21), (22, 23, 24)),
            ],
        );
        let xp = rex::XpFile::read(&mut &raw[..]).unwrap();
        assert_eq!(xp.version, -1);
        assert_eq!(xp.layers.len(), 1);
        let layer = &xp.layers[0];
        assert_eq!((layer.width, layer.height), (2, 2));
        assert_eq!(layer.get(0, 0).unwrap().ch, 65);
        assert_eq!(layer.get(0, 1).unwrap().ch, 66);
        assert_eq!(layer.get(1, 0).unwrap().ch, 67);
        assert_eq!(layer.get(1, 1).unwrap().ch, 68);
        let c = layer.get(1, 0).unwrap();
        assert_eq!(c.fg, rex::XpColor::new(13, 14, 15));
        assert_eq!(c.bg, rex::XpColor::new(16, 17, 18));
    }

    #[test]
    fn xpfile_from_bytes_gzip_roundtrip() {
        let raw = build_xp_raw(
            3,
            1,
            &[
                cell_bytes(223, (255, 0, 0), (0, 0, 255)),
                cell_bytes(32, (1, 1, 1), (2, 2, 2)),
                cell_bytes(219, (9, 9, 9), (8, 8, 8)),
            ],
        );
        let xp = rex::XpFile::from_bytes(&gzip_compress(&raw)).unwrap();
        assert_eq!(xp.layers[0].width, 3);
        assert_eq!(xp.layers[0].get(0, 0).unwrap().ch, 223);
        assert_eq!(xp.layers[0].get(2, 0).unwrap().bg, rex::XpColor::new(8, 8, 8));
    }

    #[test]
    fn xpfile_from_bytes_rejects_garbage() {
        assert!(rex::XpFile::from_bytes(b"not a gzip file").is_err());
    }

    #[test]
    fn xpfile_write_read_round_trip() {
        let mut xp = rex::XpFile::new(3, 2);
        {
            let cell = xp.layers[0].get_mut(1, 1).unwrap();
            cell.ch = 219;
            cell.fg = rex::XpColor::new(232, 193, 90);
            cell.bg = rex::XpColor::new(1, 2, 3);
        }
        let mut raw = Vec::new();
        xp.write(&mut raw).unwrap();
        let back = rex::XpFile::read(&mut &raw[..]).unwrap();
        assert_eq!(xp, back);
    }

    #[test]
    fn xpfile_gzip_round_trip() {
        let mut xp = rex::XpFile::new(4, 3);
        let cell = xp.layers[0].get_mut(2, 1).unwrap();
        cell.ch = 205;
        cell.fg = rex::XpColor::new(192, 192, 192);
        let bytes = xp.to_gzipped_bytes().unwrap();
        let back = rex::XpFile::from_bytes(&bytes).unwrap();
        assert_eq!(xp, back);
    }

    #[test]
    fn xpfile_read_truncated_is_error() {
        // Header promises 2x2 = 4 cells, but only 1 is present.
        let raw = build_xp_raw(2, 2, &[cell_bytes(65, (0, 0, 0), (0, 0, 0))]);
        assert!(rex::XpFile::read(&mut &raw[..]).is_err());
    }

    #[test]
    fn xpfile_new() {
        let xp = rex::XpFile::new(5, 7);
        assert_eq!(xp.version, -1);
        assert_eq!(xp.layers.len(), 1);
        assert_eq!((xp.layers[0].width, xp.layers[0].height), (5, 7));
        assert_eq!(xp.layers[0].cells.len(), 35);
    }

    #[test]
    fn title_xp_parses() {
        // The real title screen, embedded the same way the game embeds it.
        let bytes = include_bytes!("../resources/title.xp");
        let xp = rex::XpFile::from_bytes(bytes).unwrap();
        assert_eq!(xp.version, -1);
        assert_eq!(xp.layers.len(), 1);
        let layer = &xp.layers[0];
        assert_eq!((layer.width, layer.height), (160, 90));
        assert!(layer.get(160, 0).is_none());
        assert!(layer.get(0, 90).is_none());
        // Deco title (see src/bin/gen_title.rs): blank background, brass
        // lettering with a vertical gradient, silver ornament lines.
        assert_eq!(layer.get(0, 0).unwrap().ch, 0);
        let art = layer.get(24, 26).unwrap(); // D top bar, light brass
        assert_eq!(art.ch, 219);
        assert_eq!(art.fg, rex::XpColor::new(232, 193, 90));
        let art_bottom = layer.get(24, 32).unwrap(); // D bottom bar, dark brass
        assert_eq!(art_bottom.ch, 219);
        assert_eq!(art_bottom.fg, rex::XpColor::new(143, 107, 26));
        let line = layer.get(12, 24).unwrap(); // ornament line, silver
        assert_eq!(line.ch, 205);
        assert_eq!(line.fg, rex::XpColor::new(192, 192, 192));
        let diamond = layer.get(79, 24).unwrap(); // centered diamond, brass
        assert_eq!(diamond.ch, 4);
        assert_eq!(diamond.fg, rex::XpColor::new(201, 162, 39));
        let subtitle = layer.get(69, 37).unwrap(); // subtitle text, silver
        assert_eq!(subtitle.ch, 'a' as u32);
        assert_eq!(subtitle.fg, rex::XpColor::new(184, 184, 184));
    }

    #[test]
    fn xpfile_from_resource_uses_registry() {
        let raw = build_xp_raw(1, 1, &[cell_bytes(66, (3, 2, 1), (6, 5, 4))]);
        // Leak to get a 'static slice for registration (test-only).
        let leaked: &'static [u8] = Box::leak(gzip_compress(&raw).into_boxed_slice());
        register_resource("test/engine_from_resource.xp", leaked);
        let xp = rex::XpFile::from_resource("test/engine_from_resource.xp").unwrap();
        let cell = xp.layers[0].get(0, 0).unwrap();
        assert_eq!(cell.ch, 66);
        assert_eq!(cell.fg, rex::XpColor::new(3, 2, 1));
    }

    #[test]
    fn xpfile_from_resource_falls_back_to_disk() {
        let raw = build_xp_raw(
            1,
            2,
            &[
                cell_bytes(67, (0, 0, 0), (0, 0, 0)),
                cell_bytes(68, (1, 1, 1), (1, 1, 1)),
            ],
        );
        let path = std::env::temp_dir().join("dieselrogue_engine_test_fallback.xp");
        std::fs::write(&path, gzip_compress(&raw)).unwrap();
        let xp = rex::XpFile::from_resource(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(xp.layers[0].height, 2);
        assert_eq!(xp.layers[0].get(0, 1).unwrap().ch, 68);
    }

    // ── Font loading ──

    fn register_test_fonts() {
        crate::embedded_resource!(TEST_FONT_8X8, "../resources/CGA8x8thick.png");
        crate::embedded_resource!(TEST_FONT_10X10, "../resources/rexpaint_cp437_10x10.png");
        crate::link_resource!(TEST_FONT_8X8, "resources/CGA8x8thick.png");
        crate::link_resource!(TEST_FONT_10X10, "resources/rexpaint_cp437_10x10.png");
    }

    #[test]
    fn font_loads_from_png() {
        let font = Font::from_png_bytes(include_bytes!("../resources/CGA8x8thick.png"), 8, 8).unwrap();
        assert_eq!(font.glyph_width, 8);
        assert_eq!(font.glyph_height, 8);
        // Atlas is a 16x16 grid of 8x8 glyphs.
        assert_eq!(font.pixels.len(), (16 * 8) * (16 * 8) * 4);
        // The sheet is white-on-black with no alpha transparency, so glyph
        // visibility is determined by brightness, not alpha (see texel_visible).
        // Glyph 219 (full block) is solid: every texel is visible.
        for py in 0..8 {
            for px in 0..8 {
                assert!(texel_visible(font.pixel(219, px, py)));
            }
        }
        // Glyph 32 (space) is empty: every texel is invisible.
        for py in 0..8 {
            for px in 0..8 {
                assert!(!texel_visible(font.pixel(32, px, py)));
            }
        }
        // Glyph 'A' (65) has both visible and invisible texels.
        let mut visible = 0;
        let mut invisible = 0;
        for py in 0..8 {
            for px in 0..8 {
                if texel_visible(font.pixel(65, px, py)) {
                    visible += 1;
                } else {
                    invisible += 1;
                }
            }
        }
        assert!(visible > 0 && invisible > 0);
    }

    #[test]
    fn font_rejects_wrong_dimensions() {
        let bytes = include_bytes!("../resources/rexpaint_cp437_10x10.png");
        assert!(Font::from_png_bytes(bytes, 8, 8).is_err());
        assert!(Font::from_png_bytes(bytes, 10, 10).is_ok());
    }

    #[test]
    fn font_lookup_prefers_embedded_over_disk() {
        // Register embedded bytes under exactly the prefixed lookup key, with
        // different content on disk at the same path.
        let disk_dir = std::env::temp_dir().join("dieselrogue_font_pref");
        std::fs::create_dir_all(&disk_dir).unwrap();
        std::fs::write(disk_dir.join("pref_font.png"), b"disk-content").unwrap();
        let key = format!("{}/pref_font.png", disk_dir.to_str().unwrap());
        register_resource(&key, b"embedded-content");
        let bytes = load_font_bytes("pref_font.png", disk_dir.to_str().unwrap()).unwrap();
        std::fs::remove_dir_all(&disk_dir).unwrap();
        assert_eq!(bytes, b"embedded-content");
    }

    #[test]
    fn font_lookup_falls_back_to_disk() {
        let path = std::env::temp_dir().join("dieselrogue_engine_test_font.png");
        std::fs::write(&path, b"png-bytes").unwrap();
        let bytes = load_font_bytes(path.to_str().unwrap(), "resources").unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(bytes, b"png-bytes");
    }

    // ── ContextBuilder (headless) ──

    /// Builds a small headless context: 4x2 cells, a simple console 0 and a
    /// fancy console 1, using the real 8x8 font at the given tile size.
    fn make_render_ctx(tile: u32) -> Context {
        register_test_fonts();
        ContextBuilder::new()
            .with_simple_console(4, 2, "CGA8x8thick.png")
            .with_fancy_console(4, 2, "CGA8x8thick.png")
            .with_dimensions(4, 2)
            .with_font("CGA8x8thick.png", 8, 8)
            .with_tile_dimensions(tile, tile)
            .build_context()
            .unwrap()
    }

    #[test]
    fn builder_creates_consoles_and_loads_fonts() {
        register_test_fonts();
        let ctx = ContextBuilder::new()
            .with_fancy_console(160, 90, "CGA8x8thick.png")
            .with_fancy_console(160, 90, "CGA8x8thick.png")
            .with_dimensions(160, 90)
            .with_title("Test")
            .with_resource_path("resources")
            .with_font("CGA8x8thick.png", 8, 8)
            .with_tile_dimensions(8, 8)
            .build_context()
            .unwrap();
        assert_eq!(ctx.width_pixels, 1280);
        assert_eq!(ctx.height_pixels, 720);
        assert_eq!(ctx.num_consoles(), 2);
        assert_eq!(ctx.console(0).width(), 160);
        assert_eq!(ctx.console(1).height(), 90);
        assert_eq!(ctx.fonts.len(), 1);
        assert_eq!(ctx.fonts[0].glyph_width, 8);
        assert_eq!(ctx.tile_width, 8);
        assert_eq!(ctx.tile_height, 8);
        assert_eq!(ctx.framebuffer.len(), 1280 * 720);
        assert!(ctx.window.is_none());
    }

    #[test]
    fn builder_defaults_tile_size_to_font() {
        register_test_fonts();
        let ctx = ContextBuilder::new()
            .with_simple_console(10, 5, "rexpaint_cp437_10x10.png")
            .with_dimensions(10, 5)
            .with_font("rexpaint_cp437_10x10.png", 10, 10)
            .build_context()
            .unwrap();
        assert_eq!(ctx.tile_width, 10);
        assert_eq!(ctx.tile_height, 10);
        assert_eq!(ctx.width_pixels, 100);
        assert_eq!(ctx.height_pixels, 50);
    }

    #[test]
    fn builder_missing_font_is_error() {
        let result = ContextBuilder::new()
            .with_simple_console(10, 5, "no_such_font_9f3b2.png")
            .with_font("no_such_font_9f3b2.png", 8, 8)
            .build_context();
        assert!(result.is_err());
    }

    #[test]
    fn fullscreen_options_remove_decorations() {
        // Emulated fullscreen: no frame, no caption bar, no window controls.
        let opts = window_options(true);
        assert!(opts.none, "WS_POPUP window: no non-client area on Windows");
        assert!(opts.borderless);
        assert!(!opts.title, "caption bar and window controls must be gone");
        assert!(!opts.resize);
        assert!(matches!(opts.scale, minifb::Scale::FitScreen));
        let windowed = window_options(false);
        assert!(!windowed.borderless);
        assert!(!windowed.none);
        assert!(windowed.title);
    }

    #[test]
    fn set_fullscreen_headless_stores_flag() {
        // Headless context: the flag is stored, no window to recreate.
        let mut ctx = make_test_ctx();
        assert!(!ctx.fullscreen);
        assert!(ctx.window.is_none());
        ctx.set_fullscreen(true).unwrap();
        assert!(ctx.fullscreen);
        assert!(ctx.window.is_none());
        // A repeated toggle is a no-op and toggling back works.
        ctx.set_fullscreen(true).unwrap();
        ctx.set_fullscreen(false).unwrap();
        assert!(!ctx.fullscreen);
    }

    #[test]
    fn set_font_replaces_atlas_and_resizes() {
        register_test_fonts();
        let mut ctx = ContextBuilder::new()
            .with_simple_console(4, 2, "CGA8x8thick.png")
            .with_fancy_console(4, 2, "CGA8x8thick.png")
            .with_dimensions(4, 2)
            .with_font("CGA8x8thick.png", 8, 8)
            .build_context()
            .unwrap();
        assert_eq!(ctx.tile_width, 8);
        assert_eq!(ctx.width_pixels, 32);
        assert_eq!(ctx.height_pixels, 16);

        ctx.set_font("rexpaint_cp437_10x10.png", 10, 10, 10, 10).unwrap();
        assert_eq!(ctx.tile_width, 10);
        assert_eq!(ctx.tile_height, 10);
        assert_eq!(ctx.width_pixels, 40);
        assert_eq!(ctx.height_pixels, 20);
        assert_eq!(ctx.framebuffer.len(), 40 * 20);
        assert_eq!(ctx.fonts.len(), 1);
        assert!(ctx.consoles.iter().all(|c| c.font_index == 0));

        // Tile size may differ from the glyph size (scaled rendering).
        ctx.set_font("CGA8x8thick.png", 8, 8, 16, 16).unwrap();
        assert_eq!(ctx.tile_width, 16);
        assert_eq!(ctx.width_pixels, 64);
        assert_eq!(ctx.height_pixels, 32);
        assert_eq!(ctx.framebuffer.len(), 64 * 32);

        // A missing font leaves the context unchanged.
        assert!(ctx.set_font("no_such_font_9f3b2.png", 8, 8, 8, 8).is_err());
        assert_eq!(ctx.tile_width, 16);
        assert_eq!(ctx.width_pixels, 64);
        assert_eq!(ctx.height_pixels, 32);
    }

    // ── Framebuffer rendering ──

    #[test]
    fn pack_rgb_uses_minifb_format() {
        assert_eq!(pack_rgb(0x12, 0x34, 0x56), 0x123456);
        assert_eq!(pack_rgb(255, 255, 255), 0xFFFFFF);
        assert_eq!(pack_rgb(0, 0, 0), 0);
    }

    #[test]
    fn blend_over_source_over() {
        assert_eq!(blend_over(0x000000, 255, 255, 255, 1.0), 0xFFFFFF);
        assert_eq!(blend_over(0xFFFFFF, 0, 0, 0, 1.0), 0x000000);
        // Alpha 0 leaves the destination untouched.
        assert_eq!(blend_over(0x102030, 0, 0, 0, 0.0), 0x102030);
        // 50% blends: 255 * 0.5 = 127.5, rounds to 128.
        assert_eq!(blend_over(0x000000, 255, 0, 0, 0.5), 0x800000);
        assert_eq!(blend_over(0xFFFFFF, 0, 0, 0, 0.5), 0x808080);
    }

    #[test]
    fn render_draws_opaque_cell() {
        let mut ctx = make_render_ctx(8);
        // Main console: full-block glyph, white on opaque black at (0,0).
        ctx.set(
            0,
            0,
            RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            219u8,
        );
        ctx.render_framebuffer();
        // The framebuffer is 4*8 = 32 px wide; cell (0,0) covers rows/cols 0..8.
        for py in 0..8usize {
            for px in 0..8usize {
                assert_eq!(ctx.framebuffer[py * 32 + px], 0xFFFFFF, "pixel ({},{})", px, py);
            }
        }
        // Untouched cells render as opaque black (space glyph, black bg).
        assert_eq!(ctx.framebuffer[8], 0x000000);
        assert_eq!(ctx.framebuffer[7 * 32 + 15], 0x000000);
    }

    #[test]
    fn render_blends_fancy_console_over_main() {
        let mut ctx = make_render_ctx(8);
        // UI console (fancy): half-transparent white full block at (0,0).
        ctx.set_active_console(1);
        ctx.set(
            0,
            0,
            RGBA::from_f32(1.0, 1.0, 1.0, 0.5),
            RGBA::from_f32(0.0, 0.0, 0.0, 0.0),
            219u8,
        );
        ctx.render_framebuffer();
        // 50% white over opaque black = 0x808080.
        assert_eq!(ctx.framebuffer[0], 0x808080);
        assert_eq!(ctx.framebuffer[7 * 32 + 7], 0x808080);
    }

    #[test]
    fn render_blends_transparent_background() {
        let mut ctx = make_render_ctx(8);
        // Fancy console cell with a half-transparent red background and an
        // empty (space) glyph.
        ctx.set_active_console(1);
        ctx.set(
            0,
            0,
            RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            RGBA::from_f32(1.0, 0.0, 0.0, 0.5),
            32u8,
        );
        ctx.render_framebuffer();
        assert_eq!(ctx.framebuffer[0], 0x800000);
    }

    #[test]
    fn render_scales_glyphs_to_tile_size() {
        // The "XLarge" setting: an 8px font rendered into 16px tiles.
        let mut ctx = make_render_ctx(16);
        ctx.set(
            0,
            0,
            RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            219u8,
        );
        ctx.render_framebuffer();
        // Framebuffer is 4*16 = 64 px wide; cell (0,0) covers rows/cols 0..16.
        assert_eq!(ctx.framebuffer[15 * 64 + 15], 0xFFFFFF);
        // Cell (0,1) is untouched: opaque black.
        assert_eq!(ctx.framebuffer[16 * 64], 0x000000);
    }

    #[test]
    fn render_without_fonts_leaves_black_framebuffer() {
        let mut ctx = Context::new_for_test(4, 2, 1, &[]);
        ctx.set(
            0,
            0,
            RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            219u8,
        );
        ctx.render_framebuffer();
        assert!(ctx.framebuffer.iter().all(|&p| p == 0));
    }

    // ── Key mapping ──

    #[test]
    fn map_key_covers_game_keys() {
        use minifb::Key as K;
        assert_eq!(map_key(K::A), Some(VirtualKeyCode::A));
        assert_eq!(map_key(K::Z), Some(VirtualKeyCode::Z));
        assert_eq!(map_key(K::Key0), Some(VirtualKeyCode::Key0));
        assert_eq!(map_key(K::Key9), Some(VirtualKeyCode::Key9));
        assert_eq!(map_key(K::NumPad0), Some(VirtualKeyCode::Numpad0));
        assert_eq!(map_key(K::NumPad9), Some(VirtualKeyCode::Numpad9));
        assert_eq!(map_key(K::F1), Some(VirtualKeyCode::F1));
        assert_eq!(map_key(K::F12), Some(VirtualKeyCode::F12));
        assert_eq!(map_key(K::Up), Some(VirtualKeyCode::Up));
        assert_eq!(map_key(K::Down), Some(VirtualKeyCode::Down));
        assert_eq!(map_key(K::Left), Some(VirtualKeyCode::Left));
        assert_eq!(map_key(K::Right), Some(VirtualKeyCode::Right));
        assert_eq!(map_key(K::Enter), Some(VirtualKeyCode::Return));
        assert_eq!(map_key(K::Space), Some(VirtualKeyCode::Space));
        assert_eq!(map_key(K::Escape), Some(VirtualKeyCode::Escape));
        assert_eq!(map_key(K::Tab), Some(VirtualKeyCode::Tab));
        assert_eq!(map_key(K::Backspace), Some(VirtualKeyCode::Back));
        assert_eq!(map_key(K::Delete), Some(VirtualKeyCode::Delete));
        assert_eq!(map_key(K::Insert), Some(VirtualKeyCode::Insert));
        assert_eq!(map_key(K::Home), Some(VirtualKeyCode::Home));
        assert_eq!(map_key(K::End), Some(VirtualKeyCode::End));
        assert_eq!(map_key(K::PageUp), Some(VirtualKeyCode::PageUp));
        assert_eq!(map_key(K::PageDown), Some(VirtualKeyCode::PageDown));
        assert_eq!(map_key(K::LeftShift), Some(VirtualKeyCode::LShift));
        assert_eq!(map_key(K::RightShift), Some(VirtualKeyCode::RShift));
        assert_eq!(map_key(K::LeftCtrl), Some(VirtualKeyCode::LControl));
        assert_eq!(map_key(K::RightCtrl), Some(VirtualKeyCode::RControl));
        assert_eq!(map_key(K::LeftAlt), Some(VirtualKeyCode::LAlt));
        assert_eq!(map_key(K::RightAlt), Some(VirtualKeyCode::RAlt));
        assert_eq!(map_key(K::Backquote), Some(VirtualKeyCode::Grave));
        assert_eq!(map_key(K::Minus), Some(VirtualKeyCode::Minus));
        assert_eq!(map_key(K::Equal), Some(VirtualKeyCode::Equals));
        assert_eq!(map_key(K::LeftBracket), Some(VirtualKeyCode::LBracket));
        assert_eq!(map_key(K::RightBracket), Some(VirtualKeyCode::RBracket));
        assert_eq!(map_key(K::Semicolon), Some(VirtualKeyCode::Semicolon));
        assert_eq!(map_key(K::Apostrophe), Some(VirtualKeyCode::Apostrophe));
        assert_eq!(map_key(K::Backslash), Some(VirtualKeyCode::Backslash));
        assert_eq!(map_key(K::Comma), Some(VirtualKeyCode::Comma));
        assert_eq!(map_key(K::Period), Some(VirtualKeyCode::Period));
        assert_eq!(map_key(K::Slash), Some(VirtualKeyCode::Slash));
        assert_eq!(map_key(K::NumPadPlus), Some(VirtualKeyCode::NumpadAdd));
        assert_eq!(map_key(K::NumPadMinus), Some(VirtualKeyCode::NumpadSubtract));
        assert_eq!(map_key(K::NumPadAsterisk), Some(VirtualKeyCode::NumpadMultiply));
        assert_eq!(map_key(K::NumPadSlash), Some(VirtualKeyCode::NumpadDivide));
        assert_eq!(map_key(K::NumPadDot), Some(VirtualKeyCode::NumpadDecimal));
        assert_eq!(map_key(K::NumPadEnter), Some(VirtualKeyCode::NumpadEnter));
        assert_eq!(map_key(K::Menu), Some(VirtualKeyCode::Apps));
        assert_eq!(map_key(K::CapsLock), Some(VirtualKeyCode::CapsLock));
        assert_eq!(map_key(K::NumLock), Some(VirtualKeyCode::NumLock));
        assert_eq!(map_key(K::ScrollLock), Some(VirtualKeyCode::ScrollLock));
        assert_eq!(map_key(K::Pause), Some(VirtualKeyCode::Pause));
        // Keys with no VirtualKeyCode equivalent.
        assert_eq!(map_key(K::LeftSuper), None);
        assert_eq!(map_key(K::RightSuper), None);
    }

    // ── main_loop (headless) ──

    static ML_TICKS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static ML_PIXEL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    struct QuittingState;

    impl GameState for QuittingState {
        fn tick(&mut self, ctx: &mut Context) {
            let n = ML_TICKS.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if n == 2 {
                // The frame rendered after tick 1 must show the cell set in tick 1.
                ML_PIXEL.store(ctx.framebuffer[0], std::sync::atomic::Ordering::SeqCst);
            }
            ctx.set(
                0,
                0,
                RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
                219u8,
            );
            if n >= 3 {
                ctx.quitting = true;
            }
        }
    }

    #[test]
    fn main_loop_ticks_renders_and_quits() {
        ML_TICKS.store(0, std::sync::atomic::Ordering::SeqCst);
        ML_PIXEL.store(0, std::sync::atomic::Ordering::SeqCst);
        assert!(main_loop(make_render_ctx(8), QuittingState).is_ok());
        assert_eq!(ML_TICKS.load(std::sync::atomic::Ordering::SeqCst), 3);
        assert_eq!(ML_PIXEL.load(std::sync::atomic::Ordering::SeqCst), 0xFFFFFF);
    }

    #[test]
    fn title_screen_renders_to_framebuffer() {
        register_test_fonts();
        crate::embedded_resource!(TEST_TITLE_XP, "../resources/title.xp");
        crate::link_resource!(TEST_TITLE_XP, "title.xp");
        // Mirror the game's welcome screen: draw the title .xp onto console 0
        // exactly like ui::draw_welcome_screen does.
        let xp = rex::XpFile::from_resource("title.xp").unwrap();
        let mut ctx = ContextBuilder::new()
            .with_fancy_console(160, 90, "CGA8x8thick.png")
            .with_fancy_console(160, 90, "CGA8x8thick.png")
            .with_dimensions(160, 90)
            .with_font("CGA8x8thick.png", 8, 8)
            .with_tile_dimensions(8, 8)
            .build_context()
            .unwrap();
        ctx.cls();
        let layer = &xp.layers[0];
        for x in 0..160i32 {
            for y in 0..90i32 {
                if let Some(cell) = layer.get(x as usize, y as usize) {
                    ctx.set(
                        x,
                        y,
                        RGB::from_u8(cell.fg.r, cell.fg.g, cell.fg.b),
                        RGB::from_u8(cell.bg.r, cell.bg.g, cell.bg.b),
                        cell.ch as u8,
                    );
                }
            }
        }
        ctx.render_framebuffer();
        // The deco title renders: full-block art cells show as solid brass
        // (center pixel of art cell (24, 26), 8px tiles, light brass).
        let w = ctx.width_pixels as usize;
        assert_eq!(ctx.framebuffer[(26 * 8 + 4) * w + (24 * 8 + 4)], 0xE8C15A);
        let non_black = ctx.framebuffer.iter().filter(|&&p| p != 0).count();
        assert!(
            non_black > ctx.framebuffer.len() / 100,
            "only {} of {} pixels are non-black",
            non_black,
            ctx.framebuffer.len()
        );
    }
}
