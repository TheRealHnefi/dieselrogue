use rltk::{Rltk, RGB};
use super::*;

/**
 * Menu overview:
 * Main menu - system commands (save, quit, load)
 * Action menu - in-world commands (use item, use ability, inspect, shoot)
 *   - Item menu - pick an inventory item to use
 *     - Item action menu - pick an action for that item
 *   - Ability menu - pick an ability to use
 */

pub trait Menu {
    fn select_next(&mut self);
    fn select_previous(&mut self);
    fn get_action(&self) -> MenuAction;
    fn draw(&self, context: &mut Rltk);
}

pub trait MenuRow {
    fn get_action(&self) -> MenuAction;
    fn get_text(&self) -> String;
}

//type MenuAction = fn (&mut State) -> RunState;
pub enum MenuAction {
    Simple(fn (&mut State) -> RunState),
    Item(Item, fn (Item, &mut State) -> RunState)
}

pub struct MenuPanel<T: MenuRow> {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<T>,
    pub selected_row: usize,
}

pub struct SystemRow {
    pub text: String,
    pub action: fn (state: &mut State) -> RunState
}

pub struct ItemRow {
    pub text: String,
    pub item: Item
}

pub struct ItemActionRow {
    pub text: String,
    pub item: Item,
    pub action: ItemAction
}

impl MenuRow for SystemRow {
    fn get_action(&self) -> MenuAction {
        return MenuAction::Simple(self.action);    
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }
}

fn show_inventory_item_menu_action(item: Item, state: &mut State) -> RunState {
    let menu = inventory_action_menu(item);
    state.menu_stack.push(Box::new(menu));

    return RunState::AwaitingMenuInput;
}

fn test(number: usize, state: &mut State) -> RunState {
    return RunState::AwaitingMenuInput;
}

impl MenuRow for ItemRow {
    fn get_action(&self) -> MenuAction {
        return MenuAction::Item(self.item.clone(), show_inventory_item_menu_action);
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }
}

fn temp_throw_action(state: &mut State) -> RunState {
    RunState::AwaitingInput
}

impl MenuRow for ItemActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action {
            ItemAction::Throw(_) => {
                MenuAction::Simple(temp_throw_action)
            }
        }
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }
}

fn action_quit(_state: &mut State) -> RunState {
    ::std::process::exit(0);
}

fn action_open_item_menu(state: &mut State) -> RunState {
    state.menu_stack.push(Box::new(item_menu(&state.world)));
    return RunState::AwaitingMenuInput;
}

pub fn main_menu() -> MenuPanel<SystemRow> {
    let quit_row = SystemRow {
        text: "Quit".to_string(),
        action: action_quit
    };

    let useitem_row = SystemRow {
        text: "Use item".to_string(),
        action: action_open_item_menu
    };

    MenuPanel {
        x: 35,
        y: 20,
        rows: vec![quit_row, useitem_row],
        selected_row: 0,
    }
}

pub fn item_menu(world: &World) -> MenuPanel<ItemRow> {
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

    MenuPanel {
        x: 35,
        y: 20,
        rows: item_rows,
        selected_row: 0
    }
}

pub fn inventory_action_menu(item: Item) -> MenuPanel<ItemActionRow> {
    let mut action_rows = vec!();
    for inventory_action in &item.inventory_actions {
        action_rows.push(ItemActionRow {
            action: inventory_action.clone(),
            item: item.clone(),
            text: "Throw for now".to_string()
        });
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows: action_rows,
        selected_row: 0
    }
}

impl<RowType> Menu for MenuPanel<RowType> where RowType: MenuRow {
    fn select_next(&mut self){
        self.selected_row += 1;
        if self.selected_row > self.rows.len() - 1 {
            self.selected_row = 0;
        }
    }

    fn select_previous(&mut self) {
        if self.selected_row == 0 {
            self.selected_row = self.rows.len() - 1;
        } else {
            self.selected_row -= 1;
        }
    }

    fn get_action(&self) -> MenuAction {
        assert!(self.rows.len() >= self.selected_row);
        return self.rows[self.selected_row].get_action();
    }

    fn draw(&self, context: &mut Rltk) {
        let mut width = 0;
        for row in &self.rows {
            if row.get_text().len() > width {
                width = row.get_text().len();
            }
        }
        context.draw_box(self.x, self.y, width + 3, self.rows.len() + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
        for (i, row) in self.rows.iter().enumerate() {
            if self.selected_row == i {
                context.print_color(self.x + 2, self.y + 1 + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::MAGENTA), row.get_text());
            } else {
                context.print_color(self.x + 2, self.y + 1 + i as i32, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), row.get_text());
            }
        }
    }
}
