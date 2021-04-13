use rltk::{VirtualKeyCode};
use specs::prelude::*;
use super::RunState;

pub struct Menu {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<MenuRow>
}

pub struct MenuRow {
    pub hotkey: VirtualKeyCode,
    pub text: String,
    pub functor: MenuFunction
}

// Things menufunction needs to be able to signal:
// * Go up a menu level
// * Close the menu entirely
// * Change things in the world
type MenuFunction = fn (ecs: &mut World) -> RunState;