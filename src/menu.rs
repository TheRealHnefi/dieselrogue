use rltk::{VirtualKeyCode};
use specs::prelude::*;
use super::{RunState, GameLog, Name};

pub struct Menu {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<MenuRow>,
    pub selected_row: usize,
    pub target: Option<Entity>
}

pub struct MenuRow {
    pub hotkey: VirtualKeyCode,
    pub text: String,
    pub action: MenuFunction
}

type MenuFunction = fn (menu: &Menu, ecs: &mut World) -> RunState;

impl Menu {
    pub fn new_main() -> Self {
        let quit_row = MenuRow {
            hotkey: VirtualKeyCode::Q,
            text: "(Q) Quit".to_string(),
            action: Menu::action_quit
        };
        let close_row = MenuRow {
            hotkey: VirtualKeyCode::C,
            text: "(C) Close Menu".to_string(),
            action: Menu::action_close
        };

        Self {
            x: 35,
            y: 20,
            rows: vec![quit_row, close_row],
            selected_row: 0,
            target: None
        }
    }

    pub fn new_target_menu(x: i32, y: i32, target: Entity) -> Self {
        let examine_row = MenuRow {
            hotkey: VirtualKeyCode::E,
            text: "(E) Examine".to_string(),
            action: Menu::action_examine
        };

        Menu {
            x: x + 1,
            y: y,
            rows: vec![examine_row],
            selected_row: 0,
            target: Some(target)
        }
    }

    pub fn action_quit(&self, _ecs: &mut World) -> RunState {
        ::std::process::exit(0);
    }

    pub fn action_close(&self, _ecs: &mut World) -> RunState {
        return RunState::AwaitingInput;
    }

    pub fn action_examine(menu: &Menu, ecs: &mut World) -> RunState {
        let mut game_log = ecs.fetch_mut::<GameLog>();
        match menu.target {
            Some(entity) => {
                let names = ecs.read_storage::<Name>();
                match names.get(entity) {
                    Some(name) => {
                        game_log.entries.push(name.value.to_string());
                    }
                    None => {
                        game_log.entries.push("Nameless entity".to_string());
                    }
                }
            }
            None => {
                game_log.entries.push("Empty space".to_string());
            }
        }
        return RunState::AwaitingInput;
    }
}