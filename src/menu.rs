use rltk::{VirtualKeyCode};
use specs::prelude::*;
use super::RunState;

pub struct Menu {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<MenuRow>,
    pub selected_row: usize
}

pub struct MenuRow {
    pub hotkey: VirtualKeyCode,
    pub text: String,
    pub action: MenuFunction
}

// Things menufunction needs to be able to signal:
// * Go up a menu level
// * Close the menu entirely
// * Change things in the world
type MenuFunction = fn (ecs: &mut World) -> RunState;

impl Menu {
    pub fn new_main() -> Self {
        fn quit_function(_ecs: &mut World) -> RunState {
            ::std::process::exit(0);
        }

        fn close_function(_ecs: &mut World) -> RunState {
            return RunState::AwaitingInput;
        }

        let quit_row = MenuRow {
            hotkey: VirtualKeyCode::Q,
            text: "(Q) Quit".to_string(),
            action: quit_function
        };
        let close_row = MenuRow {
            hotkey: VirtualKeyCode::C,
            text: "(C) Close Menu".to_string(),
            action: close_function
        };

        Self {
            x: 35,
            y: 20,
            rows: vec![close_row, quit_row],
            selected_row: 0
        }
    }
}