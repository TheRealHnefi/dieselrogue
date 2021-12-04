use rltk::{Rltk, RGB, Point};
use crate::item::*;
use crate::intent::*;
use crate::state::*;
use crate::World;
use crate::Entity;

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
    fn draw(&self, context: &mut Rltk, show_cursor: bool);
}

pub trait MenuRow {
    fn get_action(&self) -> MenuAction;
    fn get_text(&self) -> String;
    fn selectable(&self) -> bool;
}

pub enum MenuAction {
    Simple(fn (&mut State) -> RunState),
    WithItemAction(Item, ItemAction, fn (Item, ItemAction, &mut State) -> RunState),
    WithIntent(Intent, fn (Intent, &mut State) -> RunState),
    WithItem(Item, fn (Item, &mut State) -> RunState),
    WithTargetedBodypartIndex(usize, fn (usize, &mut State) -> RunState)
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

pub struct AbilityRow {
    pub text: String,
    pub item: Item,
    pub action: ItemAction
}

pub struct TargetingRow {
    pub text: String,
    pub entity_id: usize,
    pub bodypart_index: usize
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
        return MenuAction::WithItem(self.item.clone(), action_show_inventory_item_menu);
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
            Some(item) => return MenuAction::WithItem(item.clone(), action_apply_unequip_intent_to_player),
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

impl MenuRow for AbilityRow {
    fn get_action(&self) -> MenuAction {
        match self.action.targeting {
            Targeting::None => {
                let intent = Intent {
                    phase: self.action.phase,
                    data: IntentData::EquippedItem(self.item.equip_slots[0]),
                    action: self.action.effects
                };
                MenuAction::WithIntent(intent, action_apply_intent_to_player)
            },
            Targeting::Positional => {
                MenuAction::WithItemAction(self.item.clone(), self.action.clone(), action_target_equipment_action)
            },
            Targeting::Detailed => {
                MenuAction::WithItemAction(self.item.clone(), self.action.clone(), action_target_equipment_action)
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

impl MenuRow for ItemActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action.targeting {
            Targeting::None => {
                let intent = Intent {
                    phase: self.action.phase,
                    data: IntentData::InventoryItem(self.item.clone()),
                    action: self.action.effects
                };
                MenuAction::WithIntent(intent, action_apply_intent_to_player)
            },
            Targeting::Positional => {
                MenuAction::WithItemAction(self.item.clone(), self.action.clone(), action_target_item_action)
            },
            Targeting::Detailed => {
                MenuAction::WithItemAction(self.item.clone(), self.action.clone(), action_target_item_action)
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

impl MenuRow for TargetingRow {
    fn get_action(&self) -> MenuAction {
        return MenuAction::WithTargetedBodypartIndex(self.bodypart_index, action_apply_intent_to_target_bodypart);
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
            for item in &player.body.inventory {
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
        action_rows.push(ItemActionRow {
            action: inventory_action.clone(),
            item: item.clone(),
            text: inventory_action.name.clone()
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

pub fn ability_menu(world: &World) -> MenuPanel<AbilityRow> {
    let mut rows = vec!();
    let mut no_selectable_rows = true;
    let player = world.get_player().unwrap();

    for slot in &player.body.item_slots {
        match &slot.item {
            Some(item) => {
                for action in &item.equip_actions {
                    rows.push(AbilityRow {  text: format!("{}: {}", item.name, action.name),
                                            item: item.clone(),
                                            action: action.clone() });
                    no_selectable_rows = false;
                }
            },
            None => ()
        }
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows: rows,
        selected_row: 0,
        no_selectable_rows: no_selectable_rows
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

pub fn targeting_menu(world: &World, position: Point) -> Option<MenuPanel<TargetingRow>> {
    let mut targeting_rows = vec!();
    let pos_index = world.map.pos_idx(position);
    match &world.map.pawns[pos_index] {
        Some(pawn) => {
            for (i, bodypart) in pawn.body.parts.iter().enumerate() {
                targeting_rows.push(TargetingRow {
                    text: format!(
                        "{}: {}/{}",
                        bodypart.name,
                        bodypart.damage,
                        bodypart.max_damage),
                    entity_id: pawn.entity_id,
                    bodypart_index: i
                });
            }
        },
        None => return None
    }

    Some(MenuPanel {
        x: 35,
        y: 20,
        rows: targeting_rows,
        selected_row: 0,
        no_selectable_rows: false
    })
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

    fn draw(&self, context: &mut Rltk, show_cursor: bool) {
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
            if show_cursor && self.selected_row == i && !self.no_selectable_exists() {
                context.print_color(self.x + 2, self.y + 1 + i as i32, fg, RGB::named(rltk::MAGENTA), row.get_text());
            } else {
                context.print_color(self.x + 2, self.y + 1 + i as i32, fg, RGB::named(rltk::BLACK), row.get_text());
            }
        }
    }
}

fn action_apply_unequip_intent_to_player(item: Item, state: &mut State) -> RunState {
    match state.world.get_player_mut() {
        Ok(player) => {
            player.intent = Intent {
                data: IntentData::EquippedItem(item.equip_slots[0]),
                phase: IntentPhase::Inventory,
                action: Entity::resolve_unequip_item                
            };
            return RunState::Resolve(IntentPhase::Instant);
        },
        Err(_) => {
            state.log.entries.push("Can not unequip item".to_string());
            return RunState::AwaitingMenuInput
        }
    }
}

fn action_target_item_action(item: Item, item_action: ItemAction, state: &mut State) -> RunState {
    match state.world.get_player() {
        Ok(player) => {
            state.cursor_pos = player.position;
            state.action_item = Some(item);
            state.action_being_used = Some(item_action);
        },
        Err(_) => ()
    }
    RunState::AwaitingPositionalTargetingInput
}

fn action_target_equipment_action(item: Item, item_action: ItemAction, state: &mut State) -> RunState {
    match state.world.get_player() {
        Ok(player) => {
            state.cursor_pos = player.position;
            state.action_slot = Some(item.equip_slots[0]);
            state.action_being_used = Some(item_action);
        },
        Err(_) => ()
    }
    RunState::AwaitingPositionalTargetingInput
}

fn action_apply_intent_to_player(intent: Intent, state: &mut State) -> RunState {
    match state.world.get_player_mut() {
        Ok(player) => player.intent = intent,
        Err(_) => ()
    }    
    RunState::Resolve(IntentPhase::Instant)
}

fn action_apply_intent_to_target_bodypart(bodypart_index: usize, state: &mut State) -> RunState {
    let mut intent_data = IntentData::Void;
    match &state.action_item {
        Some(item_being_used) => {
            intent_data = IntentData::TargetBodypartWithInventory {
                item: item_being_used.clone(),
                target: state.cursor_pos,
                bodypart_index: bodypart_index
            };
        },
        None => {
            match state.action_slot {
                Some(slot_being_used) => {
                    intent_data = IntentData::TargetBodypartWithEquipment {
                        slot: slot_being_used,
                        target: state.cursor_pos,
                        bodypart_index: bodypart_index
                    }
                },
                None => assert!(false)
            }
        }
    }

    let intent = Intent {
        phase: IntentPhase::Attack, // TODO: Not necessarily true
        data: intent_data,
        action: state.action_being_used.take().unwrap().effects
    };

    state.world.get_player_mut().unwrap().intent = intent;
    
    RunState::Resolve(IntentPhase::Instant)
}
