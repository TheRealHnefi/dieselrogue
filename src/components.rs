use rltk::Point;
use crate::Animation;

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

impl Direction {
    pub fn counter_clockwise(&self) -> Direction {
        match self {
            Direction::Up => Direction::UpLeft,
            Direction::UpRight => Direction::Up,
            Direction::Right => Direction::UpRight,
            Direction::DownRight => Direction::Right,
            Direction::Down => Direction::DownRight,
            Direction::DownLeft => Direction::Down,
            Direction::Left => Direction::DownLeft,
            Direction::UpLeft => Direction::Left,
        }
    }

    pub fn clockwise(&self) -> Direction {
        match self {
            Direction::Up => Direction::UpRight,
            Direction::UpRight => Direction::Right,
            Direction::Right => Direction::DownRight,
            Direction::DownRight => Direction::Down,
            Direction::Down => Direction::DownLeft,
            Direction::DownLeft => Direction::Left,
            Direction::Left => Direction::UpLeft,
            Direction::UpLeft => Direction::Up,
        }
    }
}

#[derive (Copy, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

impl Renderable {
    pub fn new_glyph(character: rltk::FontCharType) -> Self {
        Self {
            glyph: character,
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }

    pub fn new_char(character: char) -> Self {
        Self {
            glyph: rltk::to_cp437(character),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }
}

pub enum Effect {
    Damage {entity_id: usize, bodypart_index: usize, raw_damage: u32},
    OpenDoor(Point),
    DestroyWall(Point),
    Animation(Animation),
    Embark{pilot_id: usize, vehicle_id: usize},
    Disembark{pilot_id: usize, vehicle_id: usize}
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

