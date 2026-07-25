use rltk::{Rltk, RGB, Point};
use crate::entity::Entity;
use crate::item::*;
use crate::intent::*;
use crate::state::*;
use crate::World;
use crate::SlotType;
use crate::{FontSize, Settings, Bindings, RebindTarget, key_to_str};
use crate::PaperDoll;

/**
 * Menu overview:
 * Main menu - system commands (save, quit, load)
 * Action menu - in-world commands (use item, use ability, inspect, shoot)
 *   - "Use item" enters BrowsingInventory: the player picks an item directly in the
 *     side-panel inventory list, which then opens that item's action menu.
 *     - Item action menu - pick an action for that item
 *   - Ability menu - pick an ability to use
 */

pub trait Menu {
    fn select_next(&mut self);
    fn select_previous(&mut self);
    fn no_selectable_exists(&self) -> bool;
    fn get_action(&self) -> MenuAction;
    fn draw(&self, context: &mut Rltk, show_cursor: bool);
    fn paper_doll_kind(&self) -> Option<PaperDoll> { None }
    /// Returns (x, y, width, height) of the menu box, matching what draw() renders.
    fn bounds(&self) -> Option<(i32, i32, i32, i32)> { None }
}

pub trait MenuRow {
    fn get_action(&self) -> MenuAction;
    fn get_text(&self) -> String;
    fn selectable(&self) -> bool;
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
    pub entity_action: EntityAction,
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
    no_selectable_rows: bool,
    pub paper_doll: Option<PaperDoll>,
}

pub struct SystemRow {
    pub text: String,
    pub action: fn (state: &mut State) -> RunState
}

pub struct ItemActionRow {
    pub text: String,
    pub item: Item,
    pub action: EntityAction
}

/// A row for an action from `entity.innate_actions` — no equipment slot needed.
pub struct InnateActionRow {
    pub text: String,
    pub action: EntityAction,
}

pub struct EquippedActionRow {
    pub text: String,
    pub slot: SlotType,
    pub action: EntityAction,
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

impl MenuRow for InnateActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action.targeting {
            Targeting::None => MenuAction::WithIntent(
                build_intent(&self.action, None, Resolution::None),
                action_apply_intent_to_player,
            ),
            Targeting::Positional { .. } | Targeting::Detailed | Targeting::SelfBodypart | Targeting::JumpTile
            | Targeting::UseExistingAim { .. } | Targeting::EntityAim { .. } | Targeting::Direction => {
                MenuAction::WithPendingAction(PendingAction {
                    entity_action: self.action.clone(),
                    source: None,
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

impl MenuRow for EquippedActionRow {
    fn get_action(&self) -> MenuAction {
        match self.action.targeting {
            Targeting::None => {
                MenuAction::WithIntent(
                    build_intent(&self.action, Some(ActionSource::EquippedSlot(self.slot)), Resolution::None),
                    action_apply_intent_to_player,
                )
            },
            Targeting::Positional { .. } | Targeting::Detailed | Targeting::SelfBodypart | Targeting::JumpTile | Targeting::UseExistingAim { .. } | Targeting::EntityAim { .. } | Targeting::Direction => {
                MenuAction::WithPendingAction(PendingAction {
                    entity_action: self.action.clone(),
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
                MenuAction::WithIntent(
                    build_intent(&self.action, Some(ActionSource::InventoryItem(self.item.clone())), Resolution::None),
                    action_apply_intent_to_player,
                )
            },
            Targeting::Positional { .. } | Targeting::Detailed | Targeting::SelfBodypart | Targeting::JumpTile | Targeting::UseExistingAim { .. } | Targeting::EntityAim { .. } | Targeting::Direction => {
                MenuAction::WithPendingAction(PendingAction {
                    entity_action: self.action.clone(),
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

fn action_toggle_fullscreen(state: &mut State) -> RunState {
    let current = state.pending_fullscreen
        .unwrap_or_else(|| Settings::load().fullscreen);
    let next = !current;
    state.pending_fullscreen = Some(next);
    let mut settings = Settings::load();
    settings.fullscreen = next;
    settings.save();
    state.menu_stack.pop();
    state.menu_stack.push(Box::new(settings_menu(state.pending_font_size, state.pending_fullscreen)));
    RunState::AwaitingMenuInput
}

fn action_cycle_font_size(state: &mut State) -> RunState {
    let current = state.pending_font_size
        .unwrap_or_else(|| Settings::load().font_size);
    let next = current.next();
    state.pending_font_size = Some(next);
    let mut settings = Settings::load();
    settings.font_size = next;
    settings.save();
    state.menu_stack.pop();
    state.menu_stack.push(Box::new(settings_menu(state.pending_font_size, state.pending_fullscreen)));
    RunState::AwaitingMenuInput
}

fn action_open_settings_menu(state: &mut State) -> RunState {
    state.menu_stack.push(Box::new(settings_menu(state.pending_font_size, state.pending_fullscreen)));
    RunState::AwaitingMenuInput
}

fn action_quit(_state: &mut State) -> RunState {
    ::std::process::exit(0);
}

fn action_open_item_menu(state: &mut State) -> RunState {
    let has_items = state.world.get_player()
        .map(|p| !p.body.inventory.is_empty())
        .unwrap_or(false);
    if has_items {
        state.menu_stack.clear();
        state.inventory_selected = 0;
        RunState::BrowsingInventory
    } else {
        state.log.entries.push("No usable items".to_string());
        RunState::AwaitingInput
    }
}

pub fn main_menu() -> MenuPanel<SystemRow> {
    MenuPanel {
        x: 35,
        y: 20,
        rows: vec![
            SystemRow { text: "Use item".to_string(),  action: action_open_item_menu    },
            SystemRow { text: "Settings".to_string(),  action: action_open_settings_menu },
            SystemRow { text: "Quit".to_string(),      action: action_quit              },
        ],
        selected_row: 0,
        no_selectable_rows: false,
        paper_doll: None,
    }
}

fn action_open_keybind_menu(state: &mut State) -> RunState {
    let menu = Box::new(keybind_menu(&state.bindings));
    state.menu_stack.push(menu);
    RunState::AwaitingMenuInput
}

fn rebind_wait(_: &mut State)             -> RunState { RunState::AwaitingRebind(RebindTarget::Wait,          None) }
fn rebind_get_item(_: &mut State)         -> RunState { RunState::AwaitingRebind(RebindTarget::GetItem,       None) }
fn rebind_disembark(_: &mut State)        -> RunState { RunState::AwaitingRebind(RebindTarget::Disembark,     None) }
fn rebind_inventory(_: &mut State)        -> RunState { RunState::AwaitingRebind(RebindTarget::Inventory,     None) }
fn rebind_equipment(_: &mut State)        -> RunState { RunState::AwaitingRebind(RebindTarget::Equipment,     None) }
fn rebind_ability(_: &mut State)          -> RunState { RunState::AwaitingRebind(RebindTarget::Ability,       None) }
fn rebind_juke(_: &mut State)             -> RunState { RunState::AwaitingRebind(RebindTarget::Juke,          None) }
fn rebind_look(_: &mut State)             -> RunState { RunState::AwaitingRebind(RebindTarget::Look,          None) }
fn rebind_open_menu(_: &mut State)        -> RunState { RunState::AwaitingRebind(RebindTarget::OpenMenu,      None) }
fn rebind_freelook(_: &mut State)         -> RunState { RunState::AwaitingRebind(RebindTarget::Freelook,      None) }
fn rebind_move_left(_: &mut State)        -> RunState { RunState::AwaitingRebind(RebindTarget::MoveLeft,      None) }
fn rebind_move_right(_: &mut State)       -> RunState { RunState::AwaitingRebind(RebindTarget::MoveRight,     None) }
fn rebind_move_up(_: &mut State)          -> RunState { RunState::AwaitingRebind(RebindTarget::MoveUp,        None) }
fn rebind_move_down(_: &mut State)        -> RunState { RunState::AwaitingRebind(RebindTarget::MoveDown,      None) }
fn rebind_move_up_left(_: &mut State)     -> RunState { RunState::AwaitingRebind(RebindTarget::MoveUpLeft,    None) }
fn rebind_move_up_right(_: &mut State)    -> RunState { RunState::AwaitingRebind(RebindTarget::MoveUpRight,   None) }
fn rebind_move_down_right(_: &mut State)  -> RunState { RunState::AwaitingRebind(RebindTarget::MoveDownRight, None) }
fn rebind_move_down_left(_: &mut State)   -> RunState { RunState::AwaitingRebind(RebindTarget::MoveDownLeft,  None) }
fn rebind_strafe(_: &mut State)           -> RunState { RunState::AwaitingRebind(RebindTarget::Strafe,        None) }

pub fn keybind_menu(bindings: &Bindings) -> MenuPanel<SystemRow> {
    let row = |name: &str, key, action: fn(&mut State) -> RunState| SystemRow {
        text: format!("{:<18}{}", name, key_to_str(key)),
        action,
    };
    MenuPanel {
        x: 35,
        y: 7,
        rows: vec![
            row("Wait",           bindings.wait,           rebind_wait),
            row("Get item",       bindings.get_item,       rebind_get_item),
            row("Disembark",      bindings.disembark,      rebind_disembark),
            row("Inventory",      bindings.inventory,      rebind_inventory),
            row("Equipment",      bindings.equipment,      rebind_equipment),
            row("Ability",        bindings.ability,        rebind_ability),
            row("Juke",           bindings.juke,           rebind_juke),
            row("Look",           bindings.look,           rebind_look),
            row("Open menu",      bindings.open_menu,      rebind_open_menu),
            row("Freelook",       bindings.freelook,       rebind_freelook),
            row("Move left",      bindings.move_left,      rebind_move_left),
            row("Move right",     bindings.move_right,     rebind_move_right),
            row("Move up",        bindings.move_up,        rebind_move_up),
            row("Move down",      bindings.move_down,      rebind_move_down),
            row("Move up-left",   bindings.move_up_left,   rebind_move_up_left),
            row("Move up-right",  bindings.move_up_right,  rebind_move_up_right),
            row("Move down-right",bindings.move_down_right,rebind_move_down_right),
            row("Move down-left", bindings.move_down_left, rebind_move_down_left),
            row("Strafe",         bindings.strafe,         rebind_strafe),
        ],
        selected_row: 0,
        no_selectable_rows: false,
        paper_doll: None,
    }
}

pub fn settings_menu(pending_font_size: Option<FontSize>, pending_fullscreen: Option<bool>) -> MenuPanel<SystemRow> {
    let settings = Settings::load();

    let current_size = pending_font_size.unwrap_or(settings.font_size);
    let font_label = if pending_font_size.is_some() {
        format!("Font size: {} (restart to apply)", current_size.label())
    } else {
        format!("Font size: {}", current_size.label())
    };

    let current_fs = pending_fullscreen.unwrap_or(settings.fullscreen);
    let fs_label = if pending_fullscreen.is_some() {
        format!("Fullscreen: {} (restart to apply)", if current_fs { "On" } else { "Off" })
    } else {
        format!("Fullscreen: {}", if current_fs { "On" } else { "Off" })
    };

    MenuPanel {
        x: 35,
        y: 20,
        rows: vec![
            SystemRow { text: "Key bindings".to_string(), action: action_open_keybind_menu },
            SystemRow { text: font_label,                 action: action_cycle_font_size   },
            SystemRow { text: fs_label,                   action: action_toggle_fullscreen },
        ],
        selected_row: 0,
        no_selectable_rows: false,
        paper_doll: None,
    }
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

    // Anchor the box just left of the side-panel inventory, first row aligned with the
    // highlighted item (box width = longest action + border/padding, per MenuPanel::draw).
    let (row_x, row_y) = crate::inventory_row_pos(state.inventory_selected);
    let box_w = action_rows.iter().map(|r| r.text.len()).max().unwrap_or(0) as i32 + 3;
    MenuPanel {
        x: (row_x - 3 - box_w).max(0),
        y: row_y - 1,
        rows: action_rows,
        selected_row: 0,
        no_selectable_rows: false,
        paper_doll: None,
    }
}

pub fn equipment_action_menu(slot: SlotType, state: &State) -> MenuPanel<EquippedActionRow> {
    let mut action_rows = vec![];

    // Expose every action the equipped item allows (aim/fire, reload, recon, …) whose
    // precondition currently holds — the same filter the ability menu uses.
    if let Ok(player) = state.world.get_player() {
        if let Some(item) = player.get_equipped_item_ref(slot) {
            for action in &item.equip_actions {
                if (action.precondition)(player, &state.world.map, Some(item)) {
                    action_rows.push(EquippedActionRow {
                        text: action.name.clone(),
                        slot,
                        action: action.clone(),
                    });
                }
            }
        }
    }

    // Unequip is always available.
    action_rows.push(EquippedActionRow {
        text: unequip_action_def().name.clone(),
        slot,
        action: unequip_action_def(),
    });

    // Anchor just left of the side-panel equipment list, aligned with the highlighted row.
    let (row_x, row_y) = crate::equipment_row_pos(state.equipment_selected);
    let box_w = action_rows.iter().map(|r| r.text.len()).max().unwrap_or(0) as i32 + 3;
    MenuPanel {
        x: (row_x - 3 - box_w).max(0),
        y: row_y - 1,
        rows: action_rows,
        selected_row: 0,
        no_selectable_rows: false,
        paper_doll: None,
    }
}

pub fn ability_menu(world: &World) -> MenuPanel<Box<dyn MenuRow>> {
    let mut rows: Vec<Box<dyn MenuRow>> = vec![];
    let mut no_selectable_rows = true;

    if let Ok(player) = world.get_player() {
        // Equipped item actions (aim, fire, etc.).
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

        // Innate actions (Shout, IronBody, Rush, Twist, Distract, …) — same
        // precondition filter used by the AI, so what the player sees matches
        // what the AI is allowed to do.
        for action in &player.innate_actions {
            if (action.precondition)(player, &world.map, None) {
                rows.push(Box::new(InnateActionRow {
                    text: action.name.clone(),
                    action: action.clone(),
                }));
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
        paper_doll: None,
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
        no_selectable_rows: false,
        paper_doll: None,
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
        paper_doll: entity.paper_doll,
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

    fn paper_doll_kind(&self) -> Option<PaperDoll> {
        self.paper_doll
    }

    fn bounds(&self) -> Option<(i32, i32, i32, i32)> {
        let w = self.rows.iter().map(|r| r.get_text().len()).max().unwrap_or(0) as i32 + 3;
        let h = self.rows.len() as i32 + 1;
        Some((self.x, self.y, w, h))
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

    let intent = build_intent(
        &pending.entity_action,
        pending.source,
        Resolution::Bodypart { target: state.cursor_pos, bodypart_index },
    );
    state.world.get_player_mut().unwrap().intent = intent;

    RunState::Resolve(ExecutionPhase::Instant)
}
