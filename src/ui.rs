use rltk::{RGB, Rltk, Point};
use std::cmp::max;
use crate::state::*;
use crate::map::*;
use crate::components::*;

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
    let camera_pos = match state.world.get_player() {
        Ok(player) => player.position,
        Err(_) => Point{x: (SCREEN_WIDTH / 2) as i32, y: (SCREEN_HEIGHT / 2) as i32}
    };

    let mut top = max(camera_pos.y - SCREEN_HEIGHT as i32 / 2, 0);
    let mut left = max(camera_pos.x - SCREEN_WIDTH as i32 / 2, 0);
    let mut bottom = top + VIEWPORT_HEIGHT as i32;
    let mut right = left + SCREEN_WIDTH as i32;

    if right > state.world.map.width as i32{
        right = state.world.map.width as i32;
        left = right - SCREEN_WIDTH as i32;
    }
    if bottom > state.world.map.height as i32 {
        bottom = state.world.map.height as i32;
        top = bottom - VIEWPORT_HEIGHT as i32;
    }

    let blink = (monotime / 250) % 2 == 0;

    draw_map(&state.world.map, left, right, top, bottom, context, blink);
    draw_main_ui(state, left, right, top, bottom, context, blink);
}

fn draw_main_ui(state: &mut State, left: i32, _right: i32, top: i32, _bottom: i32, context: &mut Rltk, blink: bool) {
    context.draw_box(0, VIEWPORT_HEIGHT, SCREEN_WIDTH - 1, BOTTOM_BAR_HEIGHT - 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    if blink && state.run_state == RunState::AwaitingPositionalTargetingInput {
        context.set_bg(state.cursor_pos.x - left, state.cursor_pos.y - top, RGB::named(rltk::PINK));
    }

    let mut y = SCREEN_HEIGHT - BOTTOM_BAR_HEIGHT + 1;
    let length = max(state.log.entries.len() as i32 - 5, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }
}

fn draw_map(map: &Map, left: i32, right: i32, top: i32, bottom: i32, ctx: &mut Rltk, blink: bool) {
    let mut x = 0;
    let mut y = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        let tile_pos = map.idx_pos(idx);
        if tile_pos.x < left
            || tile_pos.x >= right 
            || tile_pos.y < top
            || tile_pos.y >= bottom {
            continue;
        }
        if map.revealed_tiles[idx] {
            let mut renderable = match tile {
                TileType::Floor => render_floor_tile(map, idx, blink),
                TileType::OpenDoor => render_open_door_tile(map, idx, blink),
                TileType::Wall => Renderable {
                    glyph: rltk::to_cp437('█'),
                    color: RGB::from_f32(0.0, 1.0, 0.0),
                    background: RGB::from_f32(0.0, 0.0, 0.0)
                },
                TileType::ClosedDoor => Renderable {
                    glyph: rltk::to_cp437('■'),
                    color: RGB::from_f32(0.0, 1.0, 0.0),
                    background: RGB::from_f32(0.0, 0.0, 0.0)
                }
            };
            if !map.visible_tiles[idx] {
                renderable.color = renderable.color.to_greyscale();
            }
            ctx.set(x, y, renderable.color, renderable.background, renderable.glyph);
        }
        x += 1;
        if x >= right - left {
            x = 0;
            y += 1;
        }
    }
}

fn render_floor_tile(map: &Map, tile_index: usize, blink: bool) -> Renderable {
    match &map.pawns[tile_index] {
        Some(pawn) => Renderable {
            glyph: pawn.sprite.glyph(pawn.body.facing, pawn.sprite_index, blink),
            color: rltk::RGB::named(rltk::YELLOW),
            background: RGB::from_f32(0.0, 0.0, 0.0)
        },
        None => {
            match &map.items[tile_index] {
                Some(item) => item.renderable,
                None => Renderable {
                    glyph: rltk::to_cp437('.'),
                    color: RGB::from_f32(0.0, 0.5, 0.0),
                    background: RGB::from_f32(0.0, 0.0, 0.0),
                }
            }
        }
    }
}

fn render_open_door_tile(map: &Map, tile_index: usize, blink: bool) -> Renderable {
    match &map.pawns[tile_index] {
        Some(pawn) => Renderable {
            glyph: pawn.sprite.glyph(pawn.body.facing, pawn.sprite_index, blink),
            color: rltk::RGB::named(rltk::YELLOW),
            background: RGB::from_f32(0.0, 0.0, 0.0)
        },
        None => {
            match &map.items[tile_index] {
                Some(item) => item.renderable,
                None => Renderable {
                    glyph: rltk::to_cp437(' '),
                    color: RGB::from_f32(0.0, 0.0, 0.0),
                    background: RGB::from_f32(0.0, 0.0, 0.0),
                }
            }
        }
    }
}