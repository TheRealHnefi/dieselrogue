use rltk::{Rltk, RGB, Point};
use crate::ability::Ability;
use crate::entity::Entity;
use crate::item::*;
use crate::intent::*;
use crate::player::{disembark_player_intent, iron_body_player_intent};
use crate::state::*;
use crate::World;
use crate::actions;
use crate::SlotType;

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

/// The source context attached to a pending player action.
/// Determines which IntentData variant is built when the target is confirmed.
pub enum ActionSource {
    InventoryItem(Item),
    EquippedSlot(SlotType),
}

/// An action selected from a menu that still needs a map target before it can execute.
/// Stored in State while the player moves the targeting cursor.
///
/// Targeting flow (two phases):
///   Phase 1 — player selects an action from the ability/item menu.
///              `MenuAction::WithPendingAction` is returned; menu_input stores a
///              PendingAction in State and enters AwaitingPositionalTargetingInput.
///   Phase 2a (Positional) — player confirms cursor position.
///              positional_targeting_input assembles an Intent from PendingAction + cursor_pos
///              and immediately resolves.
///   Phase 2b (Detailed) — player confirms cursor position, then selects a bodypart.
///              positional_targeting_input opens the targeting_menu; the player picks a
///              bodypart; action_apply_intent_to_target_bodypart assembles the final Intent.
pub struct PendingAction {
    pub item_action: ItemAction,
    pub source: Option<ActionSource>,
}

pub enum MenuAction {
    Simple(fn (&mut State) -> RunState),
    /// Carries a targeting action that needs cursor input before it can execute.
    /// Handled by menu_input, which stores it in State and enters targeting mode.
    /// See PendingAction for the full two-phase targeting flow.
    WithPendingAction(PendingAction),
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
    pub activation: fn(&mut State) -> RunState,
}

pub struct AbilityEntityTargetRow {
    pub text: String,
    pub action: ItemAction,
}

pub struct EquippedActionRow {
    pub text: String,
    pub slot: SlotType,
    pub action: ItemAction,
}

pub struct TargetingRow {
    pub text: String,
    pub entity_id: usize,
    pub bodypart_index: usize
}

pub struct EntityViewRow {
    pub text: String,
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
        MenuAction::Simple(self.activation)
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn selectable(&self) -> bool {
        true
    }
}

impl MenuRow for AbilityEntityTargetRow {
    fn get_action(&self) -> MenuAction {
        MenuAction::WithPendingAction(PendingAction {
            item_action: self.action.clone(),
            source: None,
        })
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn selectable(&self) -> bool {
        true
    }
}

impl MenuRow for EquippedActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action.targeting {
            Targeting::None => {
                MenuAction::WithIntent(Intent {
                    phase: self.action.phase,
                    data: IntentData::EquippedItem(self.slot),
                    action: self.action.action,
                }, action_apply_intent_to_player)
            },
            Targeting::Positional { .. } | Targeting::Detailed | Targeting::UseExistingAim { .. } | Targeting::EntityAim { .. } => {
                MenuAction::WithPendingAction(PendingAction {
                    item_action: self.action.clone(),
                    source: Some(ActionSource::EquippedSlot(self.slot)),
                })
            }
        }
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn selectable(&self) -> bool {
        true
    }
}

impl MenuRow for Box<dyn MenuRow> {
    fn get_action(&self) -> MenuAction {
        (**self).get_action()
    }
    fn get_text(&self) -> String {
        (**self).get_text()
    }
    fn selectable(&self) -> bool {
        (**self).selectable()
    }
}

impl MenuRow for ItemActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action.targeting {
            Targeting::None => {
                MenuAction::WithIntent(Intent {
                    phase: self.action.phase,
                    data: IntentData::InventoryItem(self.item.clone()),
                    action: self.action.action
                }, action_apply_intent_to_player)
            },
            Targeting::Positional { .. } | Targeting::Detailed | Targeting::UseExistingAim { .. } | Targeting::EntityAim { .. } => {
                MenuAction::WithPendingAction(PendingAction {
                    item_action: self.action.clone(),
                    source: Some(ActionSource::InventoryItem(self.item.clone())),
                })
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

impl MenuRow for EntityViewRow {
    fn get_action(&self) -> MenuAction {
        MenuAction::Simple(action_noop)
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn selectable(&self) -> bool {
        false
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
    let menu = inventory_action_menu(item, state);
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

pub fn inventory_action_menu(item: Item, state: &State) -> MenuPanel<ItemActionRow> {
    let mut action_rows = vec!();
    let player = state.world.get_player().unwrap();
    for inventory_action in &item.inventory_actions {
        if (inventory_action.precondition)(player, &state.world.map, Some(&item)) {
            action_rows.push(ItemActionRow {
                action: inventory_action.clone(),
                item: item.clone(),
                text: inventory_action.name.clone()
            });
        }
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows: action_rows,
        selected_row: 0,
        no_selectable_rows: false
    }
}

fn action_use_juke(state: &mut State) -> RunState {
    state.log("Juke: choose direction.".to_string());
    RunState::AwaitingJukeInput
}

fn action_use_disembark(state: &mut State) -> RunState {
    match disembark_player_intent(&mut state.world) {
        Ok(_) => RunState::Resolve(ExecutionPhase::Instant),
        Err(e) => {
            state.log(e.message);
            RunState::AwaitingInput
        }
    }
}

fn action_use_iron_body(state: &mut State) -> RunState {
    match iron_body_player_intent(&mut state.world) {
        Ok(_) => RunState::Resolve(ExecutionPhase::Instant),
        Err(e) => {
            state.log(e.message);
            RunState::AwaitingInput
        }
    }
}

pub fn ability_menu(world: &World) -> MenuPanel<Box<dyn MenuRow>> {
    let mut rows: Vec<Box<dyn MenuRow>> = vec![];
    let mut no_selectable_rows = true;

    if let Ok(player) = world.get_player() {
        for slot in &player.body.item_slots {
            if let Some(item) = &slot.item {
                if item.proxy { continue; }
                for equip_action in &item.equip_actions {
                    if (equip_action.precondition)(player, &world.map, Some(item)) {
                        rows.push(Box::new(EquippedActionRow {
                            text: equip_action.name.clone(),
                            slot: slot.slot_type,
                            action: equip_action.clone(),
                        }));
                        no_selectable_rows = false;
                    }
                }
            }
        }

        for ability in &player.body.abilities {
            let maybe_row: Option<Box<dyn MenuRow>> = match ability {
                Ability::Juke => Some(Box::new(AbilityRow {
                    text: ability.to_string(),
                    activation: action_use_juke,
                })),
                Ability::Disembark => Some(Box::new(AbilityRow {
                    text: ability.to_string(),
                    activation: action_use_disembark,
                })),
                Ability::IronBody => Some(Box::new(AbilityRow {
                    text: ability.to_string(),
                    activation: action_use_iron_body,
                })),
                Ability::Rush => Some(Box::new(AbilityEntityTargetRow {
                    text: ability.to_string(),
                    action: ItemAction {
                        name: ability.to_string(),
                        targeting: Targeting::EntityAim { max_range: Some(3) },
                        phase: ExecutionPhase::Inventory,
                        precondition: precondition_ok,
                        action: actions::rush_action,
                    },
                })),
                _ => None,
            };
            if let Some(row) = maybe_row {
                rows.push(row);
                no_selectable_rows = false;
            }
        }
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows,
        selected_row: 0,
        no_selectable_rows,
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
            let entity = &world.entities[pawn.entity_id];
            for (i, bodypart) in entity.body.parts.iter().enumerate() {
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

pub fn entity_equipment_view(entity: &Entity) -> MenuPanel<EntityViewRow> {
    let mut rows = vec![];

    rows.push(EntityViewRow { text: entity.name.clone() });

    for bodypart in &entity.body.parts {
        rows.push(EntityViewRow {
            text: format!("{}: {}/{}", bodypart.name, bodypart.damage, bodypart.max_damage),
        });
        for slot_index in &bodypart.slot_index {
            let item_name = match &entity.body.item_slots[*slot_index].item {
                Some(item) => item.name.clone(),
                None => "---".to_string(),
            };
            rows.push(EntityViewRow {
                text: format!("    {}", item_name),
            });
        }
    }

    MenuPanel {
        x: 35,
        y: 20,
        rows,
        selected_row: 0,
        no_selectable_rows: true,
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
                phase: ExecutionPhase::Inventory,
                action: actions::unequip_item_action                
            };
            return RunState::Resolve(ExecutionPhase::Instant);
        },
        Err(_) => {
            state.log.entries.push("Can not unequip item".to_string());
            return RunState::AwaitingMenuInput
        }
    }
}

fn action_apply_intent_to_player(intent: Intent, state: &mut State) -> RunState {
    match state.world.get_player_mut() {
        Ok(player) => player.intent = intent,
        Err(_) => ()
    }
    RunState::Resolve(ExecutionPhase::Instant)
}

fn action_apply_intent_to_target_bodypart(bodypart_index: usize, state: &mut State) -> RunState {
    // Phase 2b of detailed targeting: bodypart selected; assemble and commit the final intent.
    let pending = state.pending_action.take()
        .expect("bodypart targeting reached without a pending action");

    let data = match pending.source {
        Some(ActionSource::InventoryItem(item)) => IntentData::TargetBodypartWithInventory {
            item, target: state.cursor_pos, bodypart_index
        },
        Some(ActionSource::EquippedSlot(slot)) => IntentData::TargetBodypartWithEquipment {
            slot, target: state.cursor_pos, bodypart_index
        },
        None => unreachable!("detailed targeting requires an item or slot source"),
    };

    state.world.get_player_mut().unwrap().intent = Intent {
        phase: pending.item_action.phase,
        data,
        action: pending.item_action.action,
    };

    RunState::Resolve(ExecutionPhase::Instant)
}
