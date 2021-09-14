use rltk::{VirtualKeyCode};
use super::*;

/**
 * Menu overview:
 * Main menu - system commands (save, quit, load)
 * Action menu - in-world commands (use item, use ability, inspect, shoot)
 *   - Item menu - pick an inventory item to use
 *     - Item action menu - pick an action for that item
 *   - Ability menu - pick an ability to use
 */

pub struct Menu<RowType> {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<RowType>,
    pub selected_row: usize,
}

pub struct SystemRow {
    pub text: String,
    pub action: SystemMenuAction
}

pub struct ItemActionRow {
    pub text: String,
    pub action: ItemAction,
    pub item: Item
}

pub struct ItemRow {
    pub text: String,
    pub item: Item
}

type SystemMenuAction = fn (world: &mut World) -> RunState;

fn action_quit(_world: &mut World) -> RunState {
    ::std::process::exit(0);
}

pub fn main_menu() -> Menu<SystemRow> {
    let quit_row = SystemRow {
        text: "(Q) Quit".to_string(),
        action: action_quit
    };

    Menu {
        x: 35,
        y: 20,
        rows: vec![quit_row],
        selected_row: 0,
    }
}

pub fn item_menu(world: &World) -> Menu<ItemRow> {
    let mut item_rows = vec!();
    match world.get_player() {
        Ok(player) => {
            for item in &player.inventory {
                item_rows.push(ItemRow {
                    text: item.name.clone(),
                    item: item.clone()
                });
            }
        }
        Err(_) => ()
    }

    Menu {
        x: 35,
        y: 20,
        rows: item_rows,
        selected_row: 0
    }
}