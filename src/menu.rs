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
    fn no_selectable_exists(&self) -> bool;
    fn get_action(&self) -> MenuAction;
    fn draw(&self, context: &mut Rltk);
}

pub trait MenuRow {
    fn get_action(&self) -> MenuAction;
    fn get_text(&self) -> String;
    fn selectable(&self) -> bool;
}

pub enum MenuAction {
    Simple(fn (&mut State) -> RunState),
    Item(Item, fn (Item, &mut State) -> RunState)
}

pub struct MenuPanel<T: MenuRow> {
    pub x: i32,
    pub y: i32,
    pub rows: Vec<T>,
    pub selected_row: usize,
    no_selectable_rows: bool
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

pub struct ItemSlotRow {
    pub text: String,
    pub item: Option<Item>
}

impl MenuRow for SystemRow {
    fn get_action(&self) -> MenuAction {
        return MenuAction::Simple(self.action);
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }

    fn selectable(&self) -> bool {
        true
    }
}

impl MenuRow for ItemRow {
    fn get_action(&self) -> MenuAction {
        return MenuAction::Item(self.item.clone(), action_show_inventory_item_menu);
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }

    fn selectable(&self) -> bool {
        true
    }
}

impl MenuRow for ItemSlotRow {
    fn get_action(&self) -> MenuAction {
        match &self.item {
            Some(item) => return MenuAction::Item(item.clone(), unequip_action),
            None => return MenuAction::Simple(action_noop)
        }
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }

    fn selectable(&self) -> bool {
        match &self.item {
            Some(item) => !item.proxy,
            None => false
        }
    }
}


fn unequip_action(item: Item, state: &mut State) -> RunState {
    match state.world.get_player_mut() {
        Ok(player) => {
            for bodyslot in &player.body.item_slots {
                if bodyslot.slot_type == item.equip_slots[0] {
                    player.intent = Intent::Unequip(bodyslot.slot_type);
                    return RunState::Resolve;
                }
            }
            
            state.log.entries.push("Can not unequip item".to_string());
            return RunState::AwaitingMenuInput
        },
        Err(_) => {
            state.log.entries.push("Can not unequip item".to_string());
            return RunState::AwaitingMenuInput
        }
    }
}

fn equip_action(item: Item, state: &mut State) -> RunState {
    match state.world.get_player_mut() {
        Ok(player) => {
            let mut item_index = 0;
            for (index, inventory_item) in player.inventory.iter().enumerate() {
                if inventory_item == &item && player.body.can_equip(item.clone()) {
                    item_index = index;
                    break;
                }
            }
            player.intent = Intent::Equip(item_index);
            return RunState::Resolve;
        },
        Err(_) => {
            state.log.entries.push("Can not equip item".to_string());
            return RunState::AwaitingMenuInput
        }
    }
}

fn throw_action(item: Item, state: &mut State) -> RunState {
    match state.world.get_player() {
        Ok(player) => {
            state.cursor_pos = player.position;
            state.item_being_used = Some(item);
        },
        Err(_) => ()
    }
    RunState::AwaitingPositionalTargetingInput
}

fn drop_action(item: Item, state: &mut State) -> RunState {
    match state.world.get_player_mut() {
        Ok(player) => {
            let mut item_index = 0;
            for (index, inventory_item) in player.inventory.iter().enumerate() {
                if inventory_item == &item {
                    item_index = index;
                    break;
                }
            }
            player.intent = Intent::Drop(item_index);
        },
        Err(_) => ()
    }
    RunState::Resolve
}

impl MenuRow for ItemActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action {
            ItemAction::Throw(_) => {
                MenuAction::Item(self.item.clone(), throw_action)
            },
            ItemAction::Equip => {
                MenuAction::Item(self.item.clone(), equip_action)
            },
            ItemAction::Drop => {
                MenuAction::Item(self.item.clone(), drop_action)
            }
        }
    }

    fn get_text(&self) -> String {
        return self.text.clone();
    }

    fn selectable(&self) -> bool {
        true
    }
}

fn action_noop(_state: &mut State) -> RunState {
    return RunState::AwaitingMenuInput;
}

fn action_quit(_state: &mut State) -> RunState {
    ::std::process::exit(0);
}

fn action_open_item_menu(state: &mut State) -> RunState {
    let maybe_menu = item_menu(&state.world);
    match maybe_menu {
        Some(menu) => {
            state.menu_stack.push(Box::new(menu));
            return RunState::AwaitingMenuInput;
        },
        None => {
            state.log.entries.push("No usable items".to_string());
            return RunState::AwaitingInput;
        }
    }
}

fn action_show_inventory_item_menu(item: Item, state: &mut State) -> RunState {
    let menu = inventory_action_menu(item);
    state.menu_stack.push(Box::new(menu));

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
        no_selectable_rows: false
    }
}

pub fn item_menu(world: &World) -> Option<MenuPanel<ItemRow>> {
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

    if item_rows.len() == 0 {
        return None
    }

    Some(MenuPanel {
        x: 35,
        y: 20,
        rows: item_rows,
        selected_row: 0,
        no_selectable_rows: false
    })
}

pub fn inventory_action_menu(item: Item) -> MenuPanel<ItemActionRow> {
    let mut action_rows = vec!();
    for inventory_action in &item.inventory_actions {
        let description = match inventory_action {
            ItemAction::Throw(_) => "Throw".to_string(),
            ItemAction::Equip => "Equip".to_string(),
            ItemAction::Drop => "Drop".to_string()
        };
        action_rows.push(ItemActionRow {
            action: inventory_action.clone(),
            item: item.clone(),
            text: description
        });
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows: action_rows,
        selected_row: 0,
        no_selectable_rows: false
    }
}

pub fn equipment_menu(world: &World) -> MenuPanel<ItemSlotRow> {
    let mut slot_rows = vec!();
    let mut first_selectable_row = -1;
    let player = world.get_player().unwrap();
    for bodypart in &player.body.parts {
        slot_rows.push(ItemSlotRow {
            item: None,
            text: format!("{}:", bodypart.name)
        });
        for slot_index in &bodypart.slot_index {
            let item_name = match &player.body.item_slots[*slot_index].item {
                Some(item) => {
                    if first_selectable_row == -1 && !item.proxy {
                        first_selectable_row = slot_rows.len() as i32;
                    }
                    item.name.clone()
                },
                None => "---".to_string()
            };
            slot_rows.push(ItemSlotRow {
                item: player.body.item_slots[*slot_index].item.clone(),
                text: format!("    {}", item_name)
            })
        }
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows: slot_rows,
        selected_row: first_selectable_row as usize,
        no_selectable_rows: first_selectable_row == -1
    }
}

impl<RowType> Menu for MenuPanel<RowType> where RowType: MenuRow {
    fn select_next(&mut self) {
        if self.no_selectable_exists() {
            return;
        }
        self.selected_row += 1;
        if self.selected_row > self.rows.len() - 1 {
            self.selected_row = 0;
        }
        if !self.rows[self.selected_row].selectable() {
            self.select_next();
        }
    }

    fn select_previous(&mut self) {
        if self.no_selectable_exists() {
            return;
        }
        if self.selected_row == 0 {
            self.selected_row = self.rows.len() - 1;
        } else {
            self.selected_row -= 1;
        }

        if !self.rows[self.selected_row].selectable() {
            self.select_previous();
        }
    }

    fn no_selectable_exists(&self) -> bool {
        self.no_selectable_rows
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
            let fg = if self.rows[i].selectable() {
                RGB::named(rltk::WHITE)
            } else {
                RGB::named(rltk::DARKGRAY)
            };
            if self.selected_row == i && !self.no_selectable_exists() {
                context.print_color(self.x + 2, self.y + 1 + i as i32, fg, RGB::named(rltk::MAGENTA), row.get_text());
            } else {
                context.print_color(self.x + 2, self.y + 1 + i as i32, fg, RGB::named(rltk::BLACK), row.get_text());
            }
        }
    }
}
