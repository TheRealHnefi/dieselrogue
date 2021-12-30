use rltk::{RGB, Rltk};
use std::cmp::max;
use crate::components::*;
use crate::state::*;
use crate::map::*;
use crate::Rect;

pub const SCREEN_WIDTH: usize = 80;
pub const SCREEN_HEIGHT: usize = 50;
pub const BOTTOM_BAR_HEIGHT: usize = 7;
pub const VIEWPORT_HEIGHT: usize = SCREEN_HEIGHT - BOTTOM_BAR_HEIGHT;

pub fn draw_menu(state: &State, context: &mut Rltk, monotime: u128) {
    let show_cursor = (monotime / 250) % 2 == 0;
    for menu in &state.menu_stack {
        menu.draw(context, show_cursor);
    }
}

pub fn draw_main_screen(state: &mut State, context: &mut Rltk, monotime: u128) {
    let blink = (monotime / 250) % 2 == 0;

    let viewport = state.get_viewport();

    draw_map(&state.world.map, viewport, context, blink);
    draw_main_ui(state, viewport, context, blink);
}

fn draw_main_ui(state: &mut State, viewport: Rect, context: &mut Rltk, blink: bool) {
    context.draw_box(0, VIEWPORT_HEIGHT, SCREEN_WIDTH - 1, BOTTOM_BAR_HEIGHT - 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    if blink && state.run_state == RunState::AwaitingPositionalTargetingInput {
        context.set_bg(state.cursor_pos.x - viewport.x1, state.cursor_pos.y - viewport.y1, RGB::named(rltk::PINK));
    }

    let mut y = SCREEN_HEIGHT - BOTTOM_BAR_HEIGHT + 1;
    let length = max(state.log.entries.len() as i32 - 5, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }
}

fn draw_map(map: &Map, viewport: Rect, ctx: &mut Rltk, blink: bool) {
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
                ctx.set(x - viewport.x1, y - viewport.y1, renderable.color, renderable.background, renderable.glyph);
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
