#[derive (PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft
}

#[derive (Copy, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

impl Renderable {
    pub fn new() -> Self {
        Self {
            glyph: rltk::to_cp437('?'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }

    pub fn new_glyph(character: char) -> Self {
        Self {
            glyph: rltk::to_cp437(character),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }
}

pub enum Effect {
    Damage {entity_id: usize, bodypart_index: usize, raw_damage: u32},
}

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum SlotType {
    PrimaryHand,
    SecondaryHand,
    Headwear,
    Legwear,
    Footwear,
    Bodywear,
    LeftArmwear,
    RightArmwear
}


#[derive(Clone)]
pub enum ItemKind {
    Firearm {ammo: u32, max_ammo: u32, damage: u32},
    Misc
}
