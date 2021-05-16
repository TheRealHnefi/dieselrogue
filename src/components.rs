use legion::*;

#[derive (Clone, Copy)]
pub struct Player {}

#[derive (Clone, Copy)]
pub struct Enemy {}

#[derive (Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive (Clone, Copy, PartialEq)]
pub struct Size {
    pub x: i32,
    pub y: i32
}

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

#[derive (PartialEq, Eq, Clone, Copy)]
pub enum ItemSlot {
    MainWeapon,
    OffhandWeapon,
    Head,
    Torso,
    Legs
}

#[derive (Clone, Copy, PartialEq)]
pub struct Facing {
    pub direction: Direction
}

#[derive (Clone, Copy)]
pub struct Vehicle {}

#[derive (Clone, Copy)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

#[derive (Clone)]
pub struct LargeRenderable {
    pub glyphs: Vec<rltk::FontCharType>,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

#[derive (Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive (Clone)]
pub struct Name {
    pub value: String
}

#[derive (Clone, Copy)]
pub struct BlocksTile {}

#[derive (Clone, Copy)]
pub struct Firearm {
    pub range: i32
}

#[derive (Clone, Copy)]
pub struct Equippable {
    pub equipped: bool,
    pub slot: ItemSlot
}

#[derive (Clone, Copy)]
pub struct Gettable {}

#[derive (Clone)]
pub struct Inventory {
    pub items: Vec<Entity>
}
