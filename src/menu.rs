use rltk::{VirtualKeyCode};
use super::*;

/**
 * Menu overview:
 * Each menu contains one of several types of commands.
 * System menu can save, quit, load etc.
 * Action menu performs an in-world action.
 * Target menu allows targeting bodyparts, inspecting etc - passive interaction with a target.
 */

pub struct SystemMenu {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<SystemMenuRow>,
    pub selected_row: usize,
}

pub struct SystemMenuRow {
    pub hotkey: VirtualKeyCode,
    pub text: String,
    pub action: SystemMenuAction
}

type SystemMenuAction = fn (menu: &SystemMenu, ecs: &mut World) -> RunState;

impl SystemMenu {
    pub fn main_menu() -> Self {
        let quit_row = SystemMenuRow {
            hotkey: VirtualKeyCode::Q,
            text: "(Q) Quit".to_string(),
            action: SystemMenu::action_quit
        };

        Self {
            x: 35,
            y: 20,
            rows: vec![quit_row],
            selected_row: 0,
        }
    }

    fn action_quit(&self, _world: &mut World) -> RunState {
        ::std::process::exit(0);
    }
}

pub struct ActionMenu {
    pub x: i32,
    pub y: i32,
    pub item_rows: Vec<ItemActionMenuRow>,
    pub selected_row: usize,
    pub target: Option<Entity>
}

pub struct ItemActionMenuRow {
    pub hotkey: VirtualKeyCode,
    pub text: String,
    pub action: ItemAction
}

impl ActionMenu {
    pub fn all_actions(world: &World) -> Self {
        let item_actions = get_item_actions(world);
        let mut item_action_rows = vec!();
        for item_action in &item_actions {
            let label = match item_action {
                ItemAction::Throw(_) => {
                    "Throw"
                }
            };
            item_action_rows.push(ItemActionMenuRow {
                hotkey: VirtualKeyCode::Space,
                text: label.to_string(),
                action: item_action.clone()
            })
        }

        Self {
            x: 35,
            y: 20,
            item_rows: item_action_rows,
            selected_row: 0,
            target: None
        }
    }
}