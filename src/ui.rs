use rltk::prelude::*;
use std::cmp::max;
use crate::components::*;
use crate::state::*;
use crate::map::*;
use crate::tile::*;
use crate::Ability;
use crate::Rect;
use crate::entity::Entity;
use crate::RebindTarget;

const LEVELUP_SELECT_COLOR: rltk::RGB = RGB { r: 0.9, g: 0.7, b: 0.0 };
const LEVELUP_LIST_X: i32 = 4;
const LEVELUP_DESC_X: i32 = 28;
const LEVELUP_DESC_INNER_WIDTH: usize = 54;
const LEVELUP_TOP_Y: i32 = 9;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 90;

pub const VIEWPORT_HEIGHT: usize = SCREEN_HEIGHT;
pub const VIEWPORT_WIDTH: usize = VIEWPORT_HEIGHT;

const UI_HEIGHT: usize = SCREEN_HEIGHT;
const UI_WIDTH: usize = SCREEN_WIDTH - VIEWPORT_WIDTH - 1;
const UI_X_OFFSET: usize = VIEWPORT_WIDTH;
const UI_Y_OFFSET: usize = 0;

pub const MAIN_CONSOLE_INDEX: usize = 0;
pub const UI_CONSOLE_INDEX: usize = 1;

const LOCATION_PANEL_HEIGHT: usize = 5;
const HEALTH_AND_STATUS_PANEL_HEIGHT: usize = 9;
const HEALTH_PANEL_WIDTH: usize = 45;
const STATUS_PANEL_WIDTH: usize = UI_WIDTH - HEALTH_PANEL_WIDTH;
const GROUND_ITEM_PANEL_HEIGHT: usize = 4;
const INVENTORY_PANEL_HEIGHT: usize = 23;
const EQUIPMENT_PANEL_HEIGHT: usize = 11;
const ABILITIES_PANEL_HEIGHT: usize = 13;
const LOG_NOISE_PANEL_HEIGHT: usize = UI_HEIGHT
    - ABILITIES_PANEL_HEIGHT
    - EQUIPMENT_PANEL_HEIGHT
    - INVENTORY_PANEL_HEIGHT
    - GROUND_ITEM_PANEL_HEIGHT
    - HEALTH_AND_STATUS_PANEL_HEIGHT
    - LOCATION_PANEL_HEIGHT
    - 1;
const NOISE_PANEL_WIDTH: usize = 22;
const ABILITIES_PANEL_WIDTH: usize = UI_WIDTH - NOISE_PANEL_WIDTH;
const LABEL_OFFSET: usize = 2;

const LINE_COLOR: rltk::RGB = RGB {r: 1.0, g: 1.0, b: 1.0};
const BG_COLOR: rltk::RGB = RGB {r: 0.0, g: 0.0, b: 0.0};
const LABEL_COLOR: rltk::RGB = RGB {r: 1.0, g: 1.0, b: 1.0};
const INACTIVE_COLOR: rltk::RGB = RGB {r: 0.4, g: 0.4, b: 0.4};
const PHYS_COLOR: rltk::RGB = RGB {r: 0.8, g: 0.8, b: 0.8};
const FIRE_COLOR: rltk::RGB = RGB {r: 0.8, g: 0.1, b: 0.1};
const ELEC_COLOR: rltk::RGB = RGB {r: 0.1, g: 0.1, b: 0.8};
const ENERGY_COLOR: rltk::RGB = RGB {r: 0.0, g: 0.8, b: 0.8};

const WALL_COLOR: rltk::RGB = RGB {r: 0.7, g: 0.6, b: 0.4};
const GROUND_COLOR: rltk::RGB = RGB {r: 0.3, g: 0.6, b: 0.1};
const ROAD_COLOR: rltk::RGB = RGB {r: 0.3, g: 0.3, b: 0.1};
const FLOOR_COLOR: rltk::RGB = RGB {r: 0.5, g: 0.3, b: 0.1};

pub fn draw_menu(state: &State, context: &mut Rltk, monotime: u128) {
    context.set_active_console(UI_CONSOLE_INDEX);

    let show_cursor = (monotime / 250) % 2 == 0;
    for menu in &state.menu_stack {
        if let Some(doll_kind) = menu.paper_doll_kind() {
            if let Some((mx, my, mw, mh)) = menu.bounds() {
                draw_equipment_panel(context, state.rex_assets.get_doll(doll_kind), mx, my, mw, mh);
            }
        }
        menu.draw(context, show_cursor);
    }
}

/// Draws the outer panel (background fill + border) and the paper doll.
/// Must be called before menu.draw() so the inner menu box renders on top.
fn draw_equipment_panel(ctx: &mut Rltk, xp: &rltk::rex::XpFile, menu_x: i32, menu_y: i32, menu_w: i32, menu_h: i32) {
    let (doll_w, doll_h) = if xp.layers.is_empty() {
        (0, 0)
    } else {
        (xp.layers[0].width as i32, xp.layers[0].height as i32)
    };

    let doll_x = (menu_x - doll_w - 2).max(1);
    let doll_y = menu_y;

    // Outer panel bounds: 1-cell padding around the combined doll + menu area.
    let panel_x = doll_x - 1;
    let panel_y = menu_y - 1;
    let panel_w = (menu_x + menu_w + 1) - panel_x;
    let panel_h = (doll_y + doll_h).max(menu_y + menu_h) + 1 - panel_y;

    ctx.draw_box(panel_x, panel_y, panel_w, panel_h, LINE_COLOR, BG_COLOR);

    // Draw the paper doll inside the panel, to the left of the menu box.
    if !xp.layers.is_empty() {
        let layer = &xp.layers[0];
        for cy in 0..doll_h {
            for cx in 0..doll_w {
                if let Some(cell) = layer.get(cx as usize, cy as usize) {
                    ctx.set(
                        doll_x + cx,
                        doll_y + cy,
                        RGB::from_u8(cell.fg.r, cell.fg.g, cell.fg.b),
                        RGB::from_u8(cell.bg.r, cell.bg.g, cell.bg.b),
                        cell.ch as u8,
                    );
                }
            }
        }
    }
}

pub fn draw_welcome_screen(state: &State, context: &mut Rltk) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    context.print_color_centered(30, LINE_COLOR, BG_COLOR, "DIESELROGUE");
    context.print_color_centered(31, INACTIVE_COLOR, BG_COLOR, "a dieselpunk roguelike");

    let welcome_image = &state.rex_assets.title_screen.layers[0];
    for x in 0..SCREEN_WIDTH {
      for y in 0..SCREEN_HEIGHT {
        if let Some(cell) = welcome_image.get(x as usize, y as usize) {
          context.set(
            x,
            y,
            RGB::from_u8(cell.fg.r, cell.fg.g, cell.fg.b),
            RGB::from_u8(cell.bg.r, cell.bg.g, cell.bg.b),
            cell.ch as u8,
        );
        }
      }
    }

    let box_w = 18i32;
    let box_x = SCREEN_WIDTH as i32 / 2 - box_w / 2;
    let box_y = 40i32;

    context.draw_box(box_x, box_y, box_w, 4, LINE_COLOR, BG_COLOR);

    let items = ["New Game", "Settings", "Quit"];
    for (i, &item) in items.iter().enumerate() {
        let row_y = box_y + 1 + i as i32;
        let (fg, bg) = if i == state.welcome_selected {
            (BG_COLOR, LINE_COLOR)
        } else {
            (LINE_COLOR, BG_COLOR)
        };
        let fill = " ".repeat((box_w - 1) as usize);
        context.print_color(box_x + 1, row_y, fg, bg, fill);
        context.print_color(box_x + 2, row_y, fg, bg, item);
    }

    context.print_color_centered(
        SCREEN_HEIGHT as i32 - 3,
        INACTIVE_COLOR,
        BG_COLOR,
        "Arrow keys to navigate, Enter to select",
    );
}

pub fn draw_rebind_prompt(target: RebindTarget, conflict: Option<&'static str>, context: &mut Rltk) {
    context.set_active_console(UI_CONSOLE_INDEX);
    let name = match target {
        RebindTarget::Wait =>            "Wait",
        RebindTarget::GetItem =>         "Get item",
        RebindTarget::Disembark =>       "Disembark",
        RebindTarget::Inventory =>       "Inventory",
        RebindTarget::Equipment =>       "Equipment",
        RebindTarget::Ability =>         "Ability",
        RebindTarget::Juke =>            "Juke",
        RebindTarget::Look =>            "Look",
        RebindTarget::OpenMenu =>        "Open menu",
        RebindTarget::Freelook =>        "Freelook",
        RebindTarget::MoveLeft =>        "Move left",
        RebindTarget::MoveRight =>       "Move right",
        RebindTarget::MoveUp =>          "Move up",
        RebindTarget::MoveDown =>        "Move down",
        RebindTarget::MoveUpLeft =>      "Move up-left",
        RebindTarget::MoveUpRight =>     "Move up-right",
        RebindTarget::MoveDownRight =>   "Move down-right",
        RebindTarget::MoveDownLeft =>    "Move down-left",
    };
    let (text, color) = match conflict {
        Some(other) => (
            format!("Already bound to '{}'. Press another key or Esc to cancel", other),
            FIRE_COLOR,
        ),
        None => (
            format!("Press a key for '{}', or Esc to cancel", name),
            INACTIVE_COLOR,
        ),
    };
    // Menu is at y=7 with 18 rows → bottom border at y=26; prompt sits two lines below
    context.print_color_centered(28, color, BG_COLOR, &text);
}

pub fn draw_game_over_screen(context: &mut Rltk) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    let mid = SCREEN_HEIGHT as i32 / 2;
    context.print_color_centered(mid - 2, FIRE_COLOR,     BG_COLOR, "GAME OVER");
    context.print_color_centered(mid,     LINE_COLOR,     BG_COLOR, "The diesels fall silent.");
    context.print_color_centered(mid + 4, INACTIVE_COLOR, BG_COLOR, "Press any key to return to the main menu...");
}

pub fn draw_victory_screen(context: &mut Rltk) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    let mid = SCREEN_HEIGHT as i32 / 2;
    context.print_color_centered(mid - 2, LEVELUP_SELECT_COLOR, BG_COLOR, "YOU ESCAPED");
    context.print_color_centered(mid,     LINE_COLOR,            BG_COLOR, "The diesels fade behind you.");
    context.print_color_centered(mid + 4, INACTIVE_COLOR,        BG_COLOR, "Press any key to return to the main menu...");
}

pub fn draw_welcome_splash(context: &mut Rltk) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    let lines = [
        "You are a freelance operative working the fringe.",
        "The job: retrieve a prototype from the Armek compound.",
        "The pay: enough to disappear.",
        "",
        "Good luck. You'll need it.",
    ];

    let start_y = SCREEN_HEIGHT as i32 / 2 - lines.len() as i32 / 2;
    for (i, &line) in lines.iter().enumerate() {
        context.print_color_centered(start_y + i as i32, LINE_COLOR, BG_COLOR, line);
    }

    context.print_color_centered(
        SCREEN_HEIGHT as i32 - 5,
        INACTIVE_COLOR,
        BG_COLOR,
        "Press any key to continue...",
    );
}

#[tracing::instrument(skip_all)]
pub fn draw_main_screen(state: &mut State, context: &mut Rltk, monotime: u128) {
    let blink = (monotime / 250) % 2 == 0;
    let viewport = state.get_viewport(VIEWPORT_WIDTH as i32, VIEWPORT_HEIGHT as i32);

    draw_map(&state.world.map, &state.world.entities, viewport, context, blink, monotime, state.world.debug_mode);
    draw_main_ui(state, viewport, context, blink);
}

fn draw_main_ui(state: &mut State, viewport: Rect, context: &mut Rltk, blink: bool) {
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    let in_cursor_mode = state.run_state == RunState::AwaitingPositionalTargetingInput
        || state.run_state == RunState::AwaitingEntityTargetingInput
        || state.run_state == RunState::Looking;
    if in_cursor_mode && blink {
        let invalid_target = state.pending_action.as_ref().map(|pa| {
            if let crate::Targeting::Positional { max_range } = pa.item_action.targeting {
                let cursor_idx = state.world.map.pos_idx(state.cursor_pos);
                let not_visible = !state.world.map.visible_tiles[cursor_idx];
                let out_of_range = if let Some(range) = max_range {
                    state.world.get_player().ok().map(|p| {
                        let dx = state.cursor_pos.x - p.position.x;
                        let dy = state.cursor_pos.y - p.position.y;
                        dx * dx + dy * dy > (range * range) as i32
                    }).unwrap_or(false)
                } else { false };
                not_visible || out_of_range
            } else { false }
        }).unwrap_or(false);
        let cursor_color = if invalid_target { RGB::named(rltk::RED) } else { RGB::named(rltk::PINK) };
        context.set(state.cursor_pos.x - viewport.x1, state.cursor_pos.y - viewport.y1, cursor_color, RGB::named(rltk::BLACK), rltk::to_cp437('█'));
    }
    if state.run_state == RunState::Looking {
        draw_look_tooltip(state, viewport, context);
    }

    draw_panel_geometry(context);
    draw_panel_contents(state, context);
}

fn draw_panel_geometry(context: &mut Rltk) {
    let mut offset = UI_Y_OFFSET;
    // Draw scaffolding
    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, LOCATION_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += LOCATION_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, HEALTH_PANEL_WIDTH, HEALTH_AND_STATUS_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    context.draw_hollow_box_double(UI_X_OFFSET + HEALTH_PANEL_WIDTH, offset, STATUS_PANEL_WIDTH, HEALTH_AND_STATUS_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += HEALTH_AND_STATUS_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, GROUND_ITEM_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += GROUND_ITEM_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, INVENTORY_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += INVENTORY_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, EQUIPMENT_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += EQUIPMENT_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, ABILITIES_PANEL_WIDTH, ABILITIES_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    context.draw_hollow_box_double(UI_X_OFFSET + ABILITIES_PANEL_WIDTH, offset, NOISE_PANEL_WIDTH, ABILITIES_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += ABILITIES_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, LOG_NOISE_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);

    // Draw crossing points
    offset = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT;
    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + HEALTH_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╦'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += HEALTH_AND_STATUS_PANEL_HEIGHT;

    // Health/status split ends here; ground item panel is full-width below.
    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + HEALTH_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╩'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += GROUND_ITEM_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += INVENTORY_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += EQUIPMENT_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + ABILITIES_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╦'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += ABILITIES_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + ABILITIES_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╩'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));

    // Draw titles
    offset = UI_Y_OFFSET;
    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Location ╠");
    offset += LOCATION_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Damage and armor ╠");
    context.print_color(UI_X_OFFSET + LABEL_OFFSET + HEALTH_PANEL_WIDTH, offset, LABEL_COLOR, BG_COLOR, "╣ Status effects ╠");
    offset += HEALTH_AND_STATUS_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Ground ╠");
    offset += GROUND_ITEM_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Inventory ╠");
    offset += INVENTORY_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Equipment ╠");
    offset += EQUIPMENT_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Abilities ╠");
    context.print_color(UI_X_OFFSET + ABILITIES_PANEL_WIDTH + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Noise ╠");
    offset += ABILITIES_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Log ╠");
}

fn draw_panel_contents(state: &State, context: &mut Rltk) {
    let mut offset_y = UI_Y_OFFSET + 2;
    let player = match state.world.get_player() {
        Ok(player) => player,
        Err(_) => return
    };

    // Location panel — right-align indicators against the inner panel edge (UI_WIDTH - 1)
    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, "Location: Unknown");
    context.print_color(UI_X_OFFSET + LABEL_OFFSET + 35, offset_y, LABEL_COLOR, BG_COLOR, format!("Turn: {}", state.turn));
    if state.world.parallel_ai {
        context.print_color(UI_X_OFFSET + UI_WIDTH - 1 - 5 - 1 - 8, offset_y, RGB::named(rltk::GREEN), BG_COLOR, "PARALLEL");
    }
    if state.world.debug_mode {
        context.print_color(UI_X_OFFSET + UI_WIDTH - 1 - 5, offset_y, RGB::named(rltk::RED), BG_COLOR, "DEBUG");
    }
    offset_y += 1;
    let pos = player.center();
    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, format!("Position: {},{}", pos.x, pos.y));
    if state.run_state == RunState::Looking {
        context.print_color(UI_X_OFFSET + LABEL_OFFSET + 35, offset_y, LABEL_COLOR, BG_COLOR,
            format!("Cursor: {},{}", state.cursor_pos.x, state.cursor_pos.y));
    }
    if state.freelook {
        context.print_color(UI_X_OFFSET + UI_WIDTH - 1 - 8, offset_y, RGB::named(rltk::CYAN), BG_COLOR, "FREELOOK");
    }

    // Health panel
    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + 1;
    const HEALTH_X_OFFSET: usize = UI_X_OFFSET + LABEL_OFFSET + 9;
    const PHYS_X_OFFSET: usize = HEALTH_X_OFFSET + 9;
    const ELEC_X_OFFSET: usize = PHYS_X_OFFSET + 9;
    const FIRE_X_OFFSET: usize = ELEC_X_OFFSET + 9;

    context.print_color(HEALTH_X_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, "Damage");
    context.print_color(PHYS_X_OFFSET, offset_y, PHYS_COLOR, BG_COLOR, "Physical");
    context.print_color(ELEC_X_OFFSET, offset_y, ELEC_COLOR, BG_COLOR, "Electric");
    context.print_color(FIRE_X_OFFSET, offset_y, FIRE_COLOR, BG_COLOR, "Fire");

    // Status effect panel
    offset_y += 1;
    for bodypart in &player.body.parts {
        context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, &bodypart.name);
        context.print_color(HEALTH_X_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, format!("{}\\{}", &bodypart.damage, &bodypart.max_damage));
        context.print_color(PHYS_X_OFFSET, offset_y, PHYS_COLOR, BG_COLOR, format!("{}\\{}", &bodypart.armor.phys_absorption, &bodypart.armor.phys_resistance * 100.0));
        context.print_color(ELEC_X_OFFSET, offset_y, ELEC_COLOR, BG_COLOR, format!("{}\\{}", &bodypart.armor.elec_absorption, &bodypart.armor.elec_resistance * 100.0));
        context.print_color(FIRE_X_OFFSET, offset_y, FIRE_COLOR, BG_COLOR, format!("{}\\{}", &bodypart.armor.fire_absorption, &bodypart.armor.fire_resistance * 100.0));
        offset_y += 1;
    }

    // Energy bar
    const ENERGY_BAR_WIDTH: usize = 20;
    let filled = if player.body.max_energy > 0 {
        (player.body.energy as f32 / player.body.max_energy as f32 * ENERGY_BAR_WIDTH as f32) as usize
    } else {
        0
    };
    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, "Energy ");
    let bar_x = UI_X_OFFSET + LABEL_OFFSET + 7;
    for i in 0..ENERGY_BAR_WIDTH {
        let (ch, color) = if i < filled { ('█', ENERGY_COLOR) } else { ('░', INACTIVE_COLOR) };
        context.set(bar_x + i, offset_y, color, BG_COLOR, rltk::to_cp437(ch));
    }
    context.print_color(bar_x + ENERGY_BAR_WIDTH + 1, offset_y, LABEL_COLOR, BG_COLOR,
        format!("{}/{}", player.body.energy, player.body.max_energy));

    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + 2;
    const STATUS_COLUMN_WIDTH: usize = 12;
    const STATUS_COLUMN_HEIGHT: usize = HEALTH_AND_STATUS_PANEL_HEIGHT - 2;
    let status_x = UI_X_OFFSET + LABEL_OFFSET + HEALTH_PANEL_WIDTH;
    for (i, status) in player.body.status_effects.iter().enumerate() {
        let label = match status.duration() {
            Some(n) => format!("{} ({})", status.to_string(), n),
            None    => status.to_string(),
        };
        if i < STATUS_COLUMN_HEIGHT {
            context.print_color(status_x, offset_y + i, LABEL_COLOR, BG_COLOR, label);
        } else {
            context.print_color(status_x + STATUS_COLUMN_WIDTH, offset_y + i - STATUS_COLUMN_HEIGHT, LABEL_COLOR, BG_COLOR, label);
        }
    }

    // Ground item panel
    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT + 2;
    let player_tile = state.world.map.pos_idx(state.world.get_player().map(|p| p.position).unwrap_or(rltk::Point::zero()));
    if let Some(ground_item) = &state.world.map.items[player_tile] {
        context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, &ground_item.name);
    } else {
        context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, INACTIVE_COLOR, BG_COLOR, "Nothing");
    }

    // Inventory panel
    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT + GROUND_ITEM_PANEL_HEIGHT + 2;
    const INVENTORY_NAME_COLUMN_WIDTH: usize = 20;
    for (i, item) in player.body.inventory.iter().enumerate() {
        debug_assert!(i < crate::components::INVENTORY_MAX);
        context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y + i, LABEL_COLOR, BG_COLOR, format!("{}: {}", i, &item.name));
        match &item.kind {
            ItemKind::Firearm{ammo, max_ammo, damage, range} => {
                context.print_color(UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH, offset_y + i, LABEL_COLOR, BG_COLOR, format!("Ammo: {}\\{}", ammo, max_ammo));

                let label = String::from("Dmg: ");
                let phys = format!("{}", damage.physical);
                let elec = format!("{}", damage.electrical);
                let fire = format!("{}", damage.fire);
                let pierce = format!("{}", damage.piercing);
                let mut offset_x = UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH + 15;
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, &label);
                offset_x += label.len();

                context.print_color(offset_x, offset_y + i, PHYS_COLOR, BG_COLOR, &phys);
                offset_x += phys.len();

                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, "\\");
                offset_x += 1;

                context.print_color(offset_x, offset_y + i, ELEC_COLOR, BG_COLOR, &elec);
                offset_x += elec.len();

                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, "\\");
                offset_x += 1;

                context.print_color(offset_x, offset_y + i, FIRE_COLOR, BG_COLOR, &fire);
                offset_x += fire.len();

                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, "\\");
                offset_x += 1;

                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, &pierce);
                offset_x += pierce.len();

                context.print_color(offset_x + 3, offset_y + i, LABEL_COLOR, BG_COLOR, format!("Range: {}", range));
            },
            ItemKind::MeleeWeapon{damage} => {
                let label = String::from("Dmg: ");
                let phys = format!("{}", damage.physical);
                let elec = format!("{}", damage.electrical);
                let fire = format!("{}", damage.fire);
                let pierce = format!("{}", damage.piercing);
                let mut offset_x = UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH + 15;
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, &label);
                offset_x += label.len();
                context.print_color(offset_x, offset_y + i, PHYS_COLOR, BG_COLOR, &phys);
                offset_x += phys.len();
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, "\\");
                offset_x += 1;
                context.print_color(offset_x, offset_y + i, ELEC_COLOR, BG_COLOR, &elec);
                offset_x += elec.len();
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, "\\");
                offset_x += 1;
                context.print_color(offset_x, offset_y + i, FIRE_COLOR, BG_COLOR, &fire);
                offset_x += fire.len();
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, "\\");
                offset_x += 1;
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, &pierce);
            },
            ItemKind::Wearable{armor} => {
                let label = String::from("Armor: ");
                let phys = format!("{}\\{} ", armor.phys_absorption, armor.phys_resistance * 100.0);
                let elec = format!("{}\\{} ", armor.elec_absorption, armor.elec_resistance * 100.0);
                let fire = format!("{}\\{} ", armor.fire_absorption, armor.fire_resistance * 100.0);
                let mut offset_x = UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH;
                context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, &label);
                offset_x += label.len();

                context.print_color(offset_x, offset_y + i, PHYS_COLOR, BG_COLOR, &phys);
                offset_x += phys.len();

                context.print_color(offset_x, offset_y + i, ELEC_COLOR, BG_COLOR, &elec);
                offset_x += elec.len();

                context.print_color(offset_x, offset_y + i, FIRE_COLOR, BG_COLOR, &fire);
            },
            ItemKind::FusedExplosive{timeout, ..} => {
                let label = if item.active { format!("Fuse: {}", timeout) } else { String::from("Inert") };
                context.print_color(UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH, offset_y + i, LABEL_COLOR, BG_COLOR, label);
            },
            ItemKind::Key { color } => {
                let (r, g, b) = crate::components::KEY_COLORS[*color];
                let key_color = rltk::RGB::from_u8(r, g, b);
                context.print_color(UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH, offset_y + i, key_color, BG_COLOR, crate::components::KEY_COLOR_NAMES[*color]);
            },
            ItemKind::Misc => {
                context.print_color(UI_X_OFFSET + LABEL_OFFSET + INVENTORY_NAME_COLUMN_WIDTH, offset_y + i, LABEL_COLOR, BG_COLOR, format!("?"));
            }
        }
    }

    // Equipment panel
    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT + GROUND_ITEM_PANEL_HEIGHT + INVENTORY_PANEL_HEIGHT + 2;
    for (i, slot) in player.body.item_slots.iter().enumerate() {
        let mut slot_label = slot.slot_type.to_string();
        let mut offset_x = UI_X_OFFSET + LABEL_OFFSET;
        slot_label.push(':');
        context.print_color(offset_x, offset_y + i, LABEL_COLOR, BG_COLOR, slot_label);
        offset_x += 15;

        match &slot.item {
            Some(item) => {
                let color = if item.proxy { INACTIVE_COLOR } else { LABEL_COLOR };
                context.print_color(offset_x, offset_y + i, color, BG_COLOR, &item.name);
                offset_x += item.name.len();

                match item.kind {
                    ItemKind::Firearm{ammo, max_ammo, damage: _, range} => {
                        context.print_color(offset_x, offset_y + i, color, BG_COLOR, format!("  Ammo: {}\\{}  Range: {}", ammo, max_ammo, range));
                    },
                    _ => ()
                }
            },
            None => context.print_color(offset_x, offset_y + i, INACTIVE_COLOR, BG_COLOR, "---".to_string())
        }
    }

    // Abilities panel
    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT + GROUND_ITEM_PANEL_HEIGHT + INVENTORY_PANEL_HEIGHT + EQUIPMENT_PANEL_HEIGHT + 2;
    const ABILITY_TYPE_X: usize = UI_X_OFFSET + LABEL_OFFSET + 20;
    let mut abilities: Vec<&Ability> = player.body.abilities.iter().filter(|a| !a.is_innate()).collect();
    abilities.sort_by_key(|a| !a.is_passive()); // passives first
    for ability in abilities {
        let (name_color, type_label) = if ability.is_passive() {
            (LABEL_COLOR, "Passive")
        } else {
            (ENERGY_COLOR, "Active")
        };
        context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, name_color, BG_COLOR, ability.to_string());
        context.print_color(ABILITY_TYPE_X, offset_y, INACTIVE_COLOR, BG_COLOR, type_label);
        offset_y += 1;
    }

    // Log
    offset_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT + GROUND_ITEM_PANEL_HEIGHT + INVENTORY_PANEL_HEIGHT + EQUIPMENT_PANEL_HEIGHT + ABILITIES_PANEL_HEIGHT + 2;
    let max_logs = LOG_NOISE_PANEL_HEIGHT - 2;
    let length = max(state.log.entries.len() as i32 - max_logs as i32, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset_y, LABEL_COLOR, BG_COLOR, message);
        offset_y += 1;
    }

    // Noise panel
    draw_noise_panel(state, context);
}

fn sound_direction_glyph(player_pos: rltk::Point, sound_pos: rltk::Point) -> rltk::FontCharType {
    let dx = sound_pos.x - player_pos.x;
    let dy = sound_pos.y - player_pos.y;
    if dx == 0 && dy == 0 {
        return rltk::to_cp437('•');
    }
    let angle = (dy as f32).atan2(dx as f32).to_degrees();
    match angle as i32 {
        -180..=-158 | 158..=180 => rltk::to_cp437('◄'),
        -157..=-113              => rltk::to_cp437('┌'),
        -112..=-68               => rltk::to_cp437('▲'),
        -67..=-23                => rltk::to_cp437('┐'),
        -22..=22                 => rltk::to_cp437('►'),
        23..=67                  => rltk::to_cp437('┘'),
        68..=112                 => rltk::to_cp437('▼'),
        _                        => rltk::to_cp437('└'),
    }
}

fn loudness_label(dist: f32) -> &'static str {
    if dist <= 3.0       { "Loud"    }
    else if dist <= 8.0  { "Close"   }
    else if dist <= 15.0 { "Distant" }
    else                 { "Faint"   }
}

fn sound_color(kind: &SoundKind, dist: f32) -> rltk::RGB {
    let base = match kind {
        SoundKind::Gunshot   => RGB { r: 0.8, g: 0.8, b: 0.8 },
        SoundKind::Burst     => RGB { r: 1.0, g: 1.0, b: 0.6 },
        SoundKind::Explosion => RGB { r: 1.0, g: 0.4, b: 0.1 },
        SoundKind::Footstep  => RGB { r: 0.5, g: 0.5, b: 0.5 },
        SoundKind::Engine    => RGB { r: 0.4, g: 0.8, b: 0.4 },
        SoundKind::Shout     => RGB { r: 0.9, g: 0.7, b: 0.3 },
    };
    let fade = if dist <= 3.0 { 1.0 } else if dist <= 8.0 { 0.75 } else if dist <= 15.0 { 0.5 } else { 0.3 };
    RGB { r: base.r * fade, g: base.g * fade, b: base.b * fade }
}

fn draw_noise_panel(state: &State, context: &mut Rltk) {
    let player = match state.world.get_player() {
        Ok(p) => p,
        Err(_) => return,
    };

    if player.body.get_status_effect(&StatusEffect::Deaf(0)).is_some() {
        let panel_x = UI_X_OFFSET + ABILITIES_PANEL_WIDTH + LABEL_OFFSET;
        let panel_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT
            + GROUND_ITEM_PANEL_HEIGHT + INVENTORY_PANEL_HEIGHT + EQUIPMENT_PANEL_HEIGHT + 2;
        let warn_color = RGB { r: 1.0, g: 0.2, b: 0.2 };
        context.print_color(panel_x, panel_y, warn_color, BG_COLOR, "Deaf!");
        return;
    }

    let player_pos = player.center();
    let tolerance = player.body.noise_tolerance;

    let panel_x = UI_X_OFFSET + ABILITIES_PANEL_WIDTH + LABEL_OFFSET;
    let panel_y = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT + HEALTH_AND_STATUS_PANEL_HEIGHT
        + GROUND_ITEM_PANEL_HEIGHT + INVENTORY_PANEL_HEIGHT + EQUIPMENT_PANEL_HEIGHT + 2;
    let max_rows = ABILITIES_PANEL_HEIGHT - 2;
    let inner_width = NOISE_PANEL_WIDTH - LABEL_OFFSET - 1;

    // Collect audible sounds with distance and noise contribution.
    let mut audible: Vec<(&SoundEvent, f32, u32)> = state.world.sounds_last_turn.iter()
        .filter_map(|s| {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_pos, s.pos);
            if dist <= s.volume as f32 {
                let noise = (s.volume as f32 - dist).max(0.0) as u32;
                Some((s, dist, noise))
            } else { None }
        })
        .collect();

    // Loudest first.
    audible.sort_by(|a, b| b.2.cmp(&a.2));

    let mut noise_sum = 0u32;
    for (row, (sound, dist, noise)) in audible.iter().enumerate() {
        if row >= max_rows {
            break;
        }

        let glyph    = sound_direction_glyph(player_pos, sound.pos);
        let loudness = loudness_label(*dist);
        let kind_str = match sound.kind {
            SoundKind::Gunshot   => "gunshot",
            SoundKind::Burst     => "burst",
            SoundKind::Explosion => "explosion",
            SoundKind::Footstep  => "footstep",
            SoundKind::Engine    => "engine",
            SoundKind::Shout     => "shout",
        };
        let color = sound_color(&sound.kind, *dist);
        let text = format!("{} {}", loudness, kind_str);
        let text = if text.len() > inner_width - 1 { text[..inner_width - 1].to_string() } else { text };
        context.print_color(panel_x, panel_y + row, color, BG_COLOR, &text);
        context.set(panel_x + inner_width - 1, panel_y + row, color, BG_COLOR, glyph);

        noise_sum += noise;
        if noise_sum >= tolerance && row + 1 < audible.len() {
            if row + 1 < max_rows {
                let warn_color = RGB { r: 1.0, g: 0.2, b: 0.2 };
                context.print_color(panel_x, panel_y + row + 1, warn_color, BG_COLOR, "Too much noise!");
            }
            break;
        }
    }
}

fn draw_look_tooltip(state: &State, viewport: Rect, context: &mut Rltk) {
    let pos = state.cursor_pos;
    if pos.x < viewport.x1 || pos.x >= viewport.x2 || pos.y < viewport.y1 || pos.y >= viewport.y2 {
        return;
    }

    let precognition = state.world.get_player()
        .map(|p| p.has_ability(Ability::Precognition))
        .unwrap_or(false);

    let idx = state.world.map.pos_idx(pos);
    let (name, status_line, intent_desc) = if let Some(pawn) = &state.world.map.pawns[idx] {
        let entity = &state.world.entities[pawn.entity_id];
        let intent = if precognition { Some(entity.intent.description()) } else { None };
        let statuses: Vec<String> = entity.body.status_effects.iter().map(|e| {
            match e.duration() {
                Some(n) => format!("{} ({})", e.to_string(), n),
                None    => e.to_string(),
            }
        }).collect();
        let status = if statuses.is_empty() { None } else { Some(statuses.join(", ")) };
        (entity.name.clone(), status, intent)
    } else if let Some(item) = &state.world.map.items[idx] {
        let status = match &item.kind {
            ItemKind::FusedExplosive { timeout, .. } => {
                if item.active { Some(format!("Fuse: {}", timeout)) } else { Some(String::from("Inert")) }
            },
            _ => None,
        };
        (item.name.clone(), status, None)
    } else {
        return;
    };

    let cx = pos.x - viewport.x1;
    let cy = pos.y - viewport.y1;
    let max_len = name.len()
        .max(status_line.as_ref().map_or(0, |s| s.len()))
        .max(intent_desc.as_ref().map_or(0, |s| s.len())) as i32;

    // Prefer drawing to the right; fall back to the left near the viewport edge.
    let label_x = if cx + 4 + max_len < VIEWPORT_WIDTH as i32 {
        cx + 4
    } else {
        cx - 3 - max_len
    };

    let extra_rows = status_line.is_some() as i32 + intent_desc.is_some() as i32;
    let box_height = 2 + extra_rows;
    context.draw_box(label_x - 2, cy - 1, max_len + 3, box_height,
        RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    context.print_color(label_x, cy, LABEL_COLOR, BG_COLOR, &name);
    let mut row = cy + 1;
    if let Some(status) = status_line {
        context.print_color(label_x, row, INACTIVE_COLOR, BG_COLOR, &status);
        row += 1;
    }
    if let Some(desc) = intent_desc {
        context.print_color(label_x, row, INACTIVE_COLOR, BG_COLOR, &desc);
    }
}

fn flame_background(monotime: u128) -> RGB {
    match (monotime / 80) % 5 {
        0 => RGB::from_f32(0.55, 0.05, 0.0),
        1 => RGB::from_f32(0.70, 0.18, 0.0),
        2 => RGB::from_f32(0.80, 0.35, 0.0),
        3 => RGB::from_f32(0.65, 0.22, 0.0),
        _ => RGB::from_f32(0.45, 0.08, 0.0),
    }
}

fn draw_map(map: &Map, entities: &[Entity], viewport: Rect, context: &mut Rltk, blink: bool, monotime: u128, debug_mode: bool) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();

    for x in viewport.x1..viewport.x2 {
        for y in viewport.y1..viewport.y2 {
            let index = map.xy_idx(x, y);
            if debug_mode || map.revealed_tiles[index] {
                let mut renderable = match map.tiles[index] {
                    TileType::Floor => render_open_tile(map, entities, index, blink, monotime, '-', FLOOR_COLOR, debug_mode),
                    TileType::Ground => render_open_tile(map, entities, index, blink, monotime, '.', GROUND_COLOR, debug_mode),
                    TileType::Road => render_open_tile(map, entities, index, blink, monotime, '_', ROAD_COLOR, debug_mode),
                    TileType::Doorway => render_open_tile(map, entities, index, blink, monotime, ' ', FLOOR_COLOR, debug_mode),
                    TileType::Wall => Renderable {
                        glyph: rltk::to_cp437('█'),
                        color: WALL_COLOR,
                        background: rltk::RGB::named(rltk::BLACK)
                    },
                    TileType::Fence => Renderable {
                        glyph: rltk::to_cp437('#'),
                        color: WALL_COLOR,
                        background: rltk::RGB::named(rltk::BLACK)
                    },
                    TileType::Window => Renderable {
                        glyph: 8,
                        color: WALL_COLOR,
                        background: rltk::RGB::named(rltk::BLACK)
                    }
                };
                if !debug_mode && !map.visible_tiles[index] {
                    renderable.color = rltk::RGB::from_u8(30, 30, 30);
                }
                context.set(x - viewport.x1, y - viewport.y1, renderable.color, renderable.background, renderable.glyph);
            }
        }
    }
}

fn render_open_tile(map: &Map, entities: &[Entity], tile_index: usize, blink: bool, monotime: u128, empty_character: char, empty_color: rltk::RGB, debug_mode: bool) -> Renderable {
    let empty = Renderable {
        glyph: rltk::to_cp437(empty_character),
        color: empty_color,
        background: rltk::RGB::named(rltk::BLACK),
    };
    if !debug_mode && !map.visible_tiles[tile_index] {
        return empty;
    }
    match &map.pawns[tile_index] {
        Some(pawn) => {
            let entity = &entities[pawn.entity_id];
            let burning = entity.body.get_status_effect(&StatusEffect::Burning(0)).is_some();
            let color = if let Some(c) = entity.key_color {
                let (r, g, b) = crate::components::KEY_COLORS[c];
                rltk::RGB::from_u8(r, g, b)
            } else {
                rltk::RGB::named(rltk::YELLOW)
            };
            Renderable {
                glyph: entity.sprite.glyph(entity.body.facing, pawn.sprite_index, blink),
                color,
                background: if burning { flame_background(monotime) } else { rltk::RGB::named(rltk::BLACK) },
            }
        },
        None => match &map.items[tile_index] {
            Some(item) => item.renderable,
            None => empty,
        }
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub fn draw_level_up_screen(state: &State, context: &mut Rltk) {
    context.set_active_console(MAIN_CONSOLE_INDEX);

    let options = &state.level_up_options;
    if options.is_empty() {
        return;
    }

    let selected = state.level_up_selected;
    let desc_lines = wrap_text(options[selected].description(), LEVELUP_DESC_INNER_WIDTH);

    // Left panel needs one row per option plus padding.
    // Right panel needs: name row + blank row + desc lines + padding.
    let panel_height = max(options.len() + 2, desc_lines.len() + 4) as i32;

    // Title
    context.print_color(
        LEVELUP_LIST_X,
        LEVELUP_TOP_Y - 2,
        LEVELUP_SELECT_COLOR,
        BG_COLOR,
        "CHOOSE AN ABILITY",
    );

    // Left panel — ability list
    context.draw_box(LEVELUP_LIST_X, LEVELUP_TOP_Y, 22, panel_height, LINE_COLOR, BG_COLOR);
    for (i, ability) in options.iter().enumerate() {
        let (fg, prefix) = if i == selected {
            (LEVELUP_SELECT_COLOR, ">")
        } else {
            (LABEL_COLOR, " ")
        };
        context.print_color(
            LEVELUP_LIST_X + 2,
            LEVELUP_TOP_Y + 1 + i as i32,
            fg,
            BG_COLOR,
            format!("{} {}", prefix, ability.to_string()),
        );
    }

    // Right panel — description
    let desc_box_width = LEVELUP_DESC_INNER_WIDTH as i32 + 3;
    context.draw_box(LEVELUP_DESC_X, LEVELUP_TOP_Y, desc_box_width, panel_height, LINE_COLOR, BG_COLOR);
    context.print_color(
        LEVELUP_DESC_X + 2,
        LEVELUP_TOP_Y + 1,
        LEVELUP_SELECT_COLOR,
        BG_COLOR,
        options[selected].to_string(),
    );
    for (i, line) in desc_lines.iter().enumerate() {
        context.print_color(
            LEVELUP_DESC_X + 2,
            LEVELUP_TOP_Y + 3 + i as i32,
            LABEL_COLOR,
            BG_COLOR,
            line,
        );
    }
}

fn draw_help_column(context: &mut Rltk, x: i32, start_y: i32, sections: &[(&str, Vec<String>)]) {
    let mut y = start_y;
    for (title, lines) in sections {
        context.print_color(x, y, LABEL_COLOR, BG_COLOR, *title);
        y += 2;
        for line in lines {
            context.print_color(x, y, INACTIVE_COLOR, BG_COLOR, line.as_str());
            y += 1;
        }
        y += 2;
    }
}

pub fn draw_help_screen(state: &State, context: &mut Rltk) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    let b = &state.bindings;
    let w = SCREEN_WIDTH as i32;
    let h = SCREEN_HEIGHT as i32;

    // Outer double-line box
    context.draw_hollow_box_double(0, 0, w - 1, h - 1, LINE_COLOR, BG_COLOR);

    // Title and close hint
    context.print_color_centered(2, LINE_COLOR,     BG_COLOR, "HELP");
    context.print_color_centered(3, INACTIVE_COLOR, BG_COLOR, "Esc to close");

    // Horizontal separator at y=5
    for x in 1..w - 1 {
        context.set(x, 5, LINE_COLOR, BG_COLOR, rltk::to_cp437('═'));
    }
    context.set(0,     5, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(w - 1, 5, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));

    // Vertical mid-column divider — starts at the separator, not the top border.
    let mid = w / 2;
    for y in 6..h - 1 {
        context.set(mid, y, LINE_COLOR, BG_COLOR, rltk::to_cp437('║'));
    }
    context.set(mid, 5,     LINE_COLOR, BG_COLOR, rltk::to_cp437('╦'));
    context.set(mid, h - 1, LINE_COLOR, BG_COLOR, rltk::to_cp437('╩'));

    let lx    = 2i32;
    let rx    = mid + 2;
    let top_y = 7i32;

    let k = |key: rltk::VirtualKeyCode| -> String { crate::key_to_str(key).to_string() };

    let left: Vec<(&str, Vec<String>)> = vec![
        ("OBJECTIVE", vec![
            "Reach the edge of the map to escape.".into(),
            "Enemies will try to stop you.".into(),
        ]),
        ("TURNS", vec![
            "Each action advances time.".into(),
            "After you act, all enemies act simultaneously.".into(),
            format!("Press [{}] to skip a turn.", k(b.wait)),
        ]),
        ("FIRING A WEAPON", vec![
            format!("Press [{}] to open Equipment.", k(b.equipment)),
            "Select a weapon to see its actions.".into(),
            "Choose Aim, move the cursor to a target,".into(),
            "then confirm. Choose Fire to shoot.".into(),
            "You need ammo to shoot.".into(),
        ]),
        ("DAMAGE", vec![
            "Four types: physical, fire, electrical, piercing.".into(),
            "Armor absorbs, then reduces what remains.".into(),
            "Fire causes burning (damage over time).".into(),
            "Electrical can stun — you skip your next turn.".into(),
            "Head and torso are vital. Losing either is lethal.".into(),
        ]),
    ];

    let right: Vec<(&str, Vec<String>)> = vec![
        ("MOVEMENT", vec![
            "8 directions: arrow keys, numpad 1-4/6-9,".into(),
            "or rebindable keys (Settings).".into(),
            "Moving or acting always costs a turn.".into(),
        ]),
        ("LOOKING & FREELOOK", vec![
            format!("Press [{}] to enter look mode.", k(b.look)),
            "Move a cursor to inspect tiles and entities.".into(),
            format!("Press [{}] to toggle freelook.", k(b.freelook)),
            "In freelook, movement keys scroll the camera".into(),
            "without moving your character.".into(),
        ]),
        ("SOUND", vec![
            "Most actions produce sound enemies can hear.".into(),
            "Gunshots are loud; careful movement less so.".into(),
            "Enemies will investigate nearby sounds.".into(),
            "The noise panel shows nearby sound events.".into(),
        ]),
        ("ENERGY", vec![
            "Shown in the status panel.".into(),
            format!("Used by combat abilities (press [{}]).", k(b.ability)),
            "Regenerates gradually each turn.".into(),
            "Save it for when you need it most.".into(),
        ]),
    ];

    draw_help_column(context, lx, top_y, &left);
    draw_help_column(context, rx, top_y, &right);
}
