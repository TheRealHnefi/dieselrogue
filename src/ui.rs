use rltk::prelude::*;
use std::cmp::max;
use crate::components::*;
use crate::state::*;
use crate::map::*;
use crate::Rect;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 90;

pub const VIEWPORT_HEIGHT: usize = SCREEN_HEIGHT;
pub const VIEWPORT_WIDTH: usize = VIEWPORT_HEIGHT;

const UI_WIDTH: usize = SCREEN_WIDTH - VIEWPORT_WIDTH - 1;
const UI_X_OFFSET: usize = VIEWPORT_WIDTH;
const UI_Y_OFFSET: usize = 0;

const MAIN_CONSOLE_INDEX: usize = 0;
const UI_CONSOLE_INDEX: usize = 1;

pub fn draw_menu(state: &State, context: &mut Rltk, monotime: u128) {
    context.set_active_console(UI_CONSOLE_INDEX);

    let show_cursor = (monotime / 250) % 2 == 0;
    for menu in &state.menu_stack {
        menu.draw(context, show_cursor);
    }
}

pub fn draw_main_screen(state: &mut State, context: &mut Rltk, monotime: u128) {
    let blink = (monotime / 250) % 2 == 0;
    let viewport = state.get_viewport(VIEWPORT_WIDTH as i32, VIEWPORT_HEIGHT as i32);

    draw_map(&state.world.map, viewport, context, blink);
    draw_main_ui(state, viewport, context, blink);
}

fn draw_main_ui(state: &mut State, viewport: Rect, context: &mut Rltk, blink: bool) {
    context.set_active_console(UI_CONSOLE_INDEX);
    context.cls();

    if blink && state.run_state == RunState::AwaitingPositionalTargetingInput {
        context.set(state.cursor_pos.x - viewport.x1, state.cursor_pos.y - viewport.y1, RGB::named(rltk::PINK), RGB::named(rltk::BLACK), rltk::to_cp437('█'));
    }

    draw_panel_geometry(context);

    let mut y = SCREEN_HEIGHT - 10;
    let length = max(state.log.entries.len() as i32 - 5, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }
}

fn draw_panel_geometry(context: &mut Rltk) {
    const LOCATION_PANEL_HEIGHT: usize = 3;
    const HEALTH_AND_STATUS_PANEL_HEIGHT: usize = 8;
    const HEALTH_PANEL_WIDTH: usize = 35;
    const STATUS_PANEL_WIDTH: usize = 34;
    const INVENTORY_PANEL_HEIGHT: usize = 21;
    const EQUIPMENT_PANEL_HEIGHT: usize = 13;
    const ABILITIES_PANEL_HEIGHT: usize = 35;
    const LOG_NOISE_PANEL_HEIGHT: usize = 9;
    const LOG_PANEL_WIDTH: usize = 55;
    const NOISE_PANEL_WIDTH: usize = UI_WIDTH - LOG_PANEL_WIDTH;
    const LABEL_OFFSET: usize = 2;

    const LINE_COLOR: rltk::RGB = RGB {r: 1.0, g: 1.0, b: 1.0};
    const BG_COLOR: rltk::RGB = RGB {r: 0.0, g: 0.0, b: 0.0};
    const LABEL_COLOR: rltk::RGB = RGB {r: 1.0, g: 1.0, b: 1.0};

    let mut offset = UI_Y_OFFSET;
    // Draw scaffolding
    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, LOCATION_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += LOCATION_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, HEALTH_PANEL_WIDTH, HEALTH_AND_STATUS_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    context.draw_hollow_box_double(UI_X_OFFSET + HEALTH_PANEL_WIDTH, offset, STATUS_PANEL_WIDTH, HEALTH_AND_STATUS_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += HEALTH_AND_STATUS_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, INVENTORY_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += INVENTORY_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, EQUIPMENT_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += EQUIPMENT_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, UI_WIDTH, ABILITIES_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    offset += ABILITIES_PANEL_HEIGHT;

    context.draw_hollow_box_double(UI_X_OFFSET, offset, LOG_PANEL_WIDTH, LOG_NOISE_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);
    context.draw_hollow_box_double(UI_X_OFFSET + LOG_PANEL_WIDTH, offset, NOISE_PANEL_WIDTH, LOG_NOISE_PANEL_HEIGHT, LINE_COLOR, BG_COLOR);

    // Draw crossing points
    offset = UI_Y_OFFSET + LOCATION_PANEL_HEIGHT;
    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + HEALTH_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╦'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += HEALTH_AND_STATUS_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + HEALTH_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╩'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += INVENTORY_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += EQUIPMENT_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += ABILITIES_PANEL_HEIGHT;

    context.set(UI_X_OFFSET, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╠'));
    context.set(UI_X_OFFSET + LOG_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╦'));
    context.set(UI_X_OFFSET + UI_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╣'));
    offset += LOG_NOISE_PANEL_HEIGHT;

    context.set(UI_X_OFFSET + LOG_PANEL_WIDTH, offset, LINE_COLOR, BG_COLOR, rltk::to_cp437('╩'));

    // Draw titles
    offset = UI_Y_OFFSET;
    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Location ╠");
    offset += LOCATION_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Health and armor ╠");
    context.print_color(UI_X_OFFSET + LABEL_OFFSET + HEALTH_PANEL_WIDTH, offset, LABEL_COLOR, BG_COLOR, "╣ Status effects ╠");
    offset += HEALTH_AND_STATUS_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Inventory ╠");
    offset += INVENTORY_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Equipment ╠");
    offset += EQUIPMENT_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Abilities ╠");
    offset += ABILITIES_PANEL_HEIGHT;

    context.print_color(UI_X_OFFSET + LABEL_OFFSET, offset, LABEL_COLOR, BG_COLOR, "╣ Log ╠");
    context.print_color(UI_X_OFFSET + LABEL_OFFSET + LOG_PANEL_WIDTH, offset, LABEL_COLOR, BG_COLOR, "╣ Noise ╠");
}

fn draw_map(map: &Map, viewport: Rect, context: &mut Rltk, blink: bool) {
    context.set_active_console(MAIN_CONSOLE_INDEX);
    context.cls();

    for x in viewport.x1..viewport.x2 {
        for y in viewport.y1..viewport.y2 {
            let index = map.xy_idx(x, y);
            if map.revealed_tiles[index] {
                let mut renderable = match map.tiles[index] {
                    TileType::Floor => render_open_tile(map, index, blink, '-'),
                    TileType::Ground => render_open_tile(map, index, blink, '.'),
                    TileType::Doorway => render_open_tile(map, index, blink, ' '),
                    TileType::Wall => Renderable {
                        glyph: rltk::to_cp437('█'),
                        color: rltk::RGB::named(rltk::GREEN),
                        background: rltk::RGB::named(rltk::BLACK)
                    },
                };
                if !map.visible_tiles[index] {
                    renderable.color = renderable.color.to_greyscale();
                }
                context.set(x - viewport.x1, y - viewport.y1, renderable.color, renderable.background, renderable.glyph);
            }
        }
    }
}

fn render_open_tile(map: &Map, tile_index: usize, blink: bool, empty_character: char) -> Renderable {
    match &map.pawns[tile_index] {
        Some(pawn) => Renderable {
            glyph: pawn.sprite.glyph(pawn.body.facing, pawn.sprite_index, blink),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        },
        None => {
            match &map.items[tile_index] {
                Some(item) => item.renderable,
                None => Renderable {
                    glyph: rltk::to_cp437(empty_character),
                    color: RGB::from_f32(0.0, 0.5, 0.0),
                    background: rltk::RGB::named(rltk::BLACK),
                }
            }
        }
    }
}
