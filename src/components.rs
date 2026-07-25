use rltk::Point;
use crate::Map;

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
pub struct Facing {
    pub direction: Direction
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

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum Intent {
    Idle,
    Move(Point),
    Turn(Direction),
    Melee(Point),
    GetItem,
    Throw(usize, Point), // (inventory index, map position)
    Drop(usize), // (inventory index)
    Equip(usize), // (inventory index)
    Unequip(SlotType)
}

pub enum Effect {
    Damage(usize) // (entity_id)
}

#[derive(Clone)]
pub enum ItemAction {
    Throw(fn (source_position: Point, target_position: Point, map: &Map) -> Option<Effect>),
    Equip,
    Drop
}

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum SlotType {
    PrimaryHand,
    SecondaryHand,
    Headwear,
    LeftLegwear,
    RightLegwear,
    Bodywear,
    LeftArmwear,
    RightArmwear
}