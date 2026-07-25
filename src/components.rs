use specs::prelude::*;
use specs_derive::*;

#[derive (Component)]
pub struct Player {}

#[derive (Component, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive (Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    UP,
    UPRIGHT,
    RIGHT,
    DOWNRIGHT,
    DOWN,
    DOWNLEFT,
    LEFT,
    UPLEFT
}

#[derive (Component, Debug)]
pub struct Facing {
    pub direction: Direction
}

#[derive (Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

