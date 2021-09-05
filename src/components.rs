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

#[derive (Copy, Clone)]
pub struct Intent {
    pub action: Action
}

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum Action {
    Idle,
    Move(Point),
    Turn(Direction),
    Melee(Point),
    GetItem
}

pub enum Effect {
    Damage(usize) // damage(entity_id)
}

type ItemActionEffect = fn (source_position: Point, target_position: Point, map: &Map) -> Option<Effect>;

#[derive(Clone)]
pub struct ItemAction {
    pub label: String,
    pub targeting: TargetingType,
    pub cost: UsageCost,
    pub effect: ItemActionEffect
}

#[derive(Clone)]
pub enum TargetingType {
    Position
}

#[derive(Clone)]
pub enum UsageCost {
    Free,
    Consume
}
