use rltk::{RGB, Rltk, Point};
use std::cmp::max;
use crate::state::*;
use crate::map::*;

pub const SCREEN_WIDTH: usize = 80;
pub const SCREEN_HEIGHT: usize = 50;
pub const BOTTOM_BAR_HEIGHT: usize = 7;
pub const VIEWPORT_HEIGHT: usize = SCREEN_HEIGHT - BOTTOM_BAR_HEIGHT;

pub fn draw_main_screen(state: &mut State, context: &mut Rltk) {
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

    draw_map(&state.world.map, left, right, top, bottom, context);
    draw_main_ui(state, left, right, top, bottom, context);
}

fn draw_main_ui(state: &mut State, left: i32, _right: i32, top: i32, _bottom: i32, context: &mut Rltk) {
    context.draw_box(0, VIEWPORT_HEIGHT, SCREEN_WIDTH - 1, BOTTOM_BAR_HEIGHT - 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
    if state.run_state == RunState::AwaitingPositionalTargetingInput {
        context.set_bg(state.cursor_pos.x - left, state.cursor_pos.y - top, RGB::named(rltk::PINK));
    }

    let mut y = SCREEN_HEIGHT - BOTTOM_BAR_HEIGHT + 1;
    let length = max(state.log.entries.len() as i32 - 5, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }
}

fn draw_map(map: &Map, left: i32, right: i32, top: i32, bottom: i32, ctx: &mut Rltk) {
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
            let glyph;
            let mut foreground;
            let mut background = RGB::from_f32(0.0, 0.0, 0.0);
            match tile {
                TileType::Floor => {
                    match &map.pawns[idx] {
                        Some(pawn) => {
                            glyph = pawn.renderable.glyph;
                            foreground = pawn.renderable.color;
                            background = pawn.renderable.background;
                        },

                        None => {
                            match &map.items[idx] {
                                Some(item) => {
                                    glyph = item.renderable.glyph;
                                    foreground = item.renderable.color;
                                    background = item.renderable.background;
                                }
                                None => {
                                    glyph = rltk::to_cp437('.');
                                    foreground = RGB::from_f32(0.0, 0.5, 0.0);
                                }
                            }
                        }
                    };
                },
                TileType::Wall => {
                    glyph = rltk::to_cp437('█');
                    foreground = RGB::from_f32(0.0, 1.0, 0.0);
                },
                TileType::OpenDoor => {
                    match &map.pawns[idx] {
                        Some(pawn) => {
                            glyph = pawn.renderable.glyph;
                            foreground = pawn.renderable.color;
                            background = pawn.renderable.background;
                        },

                        None => {
                            match &map.items[idx] {
                                Some(item) => {
                                    glyph = item.renderable.glyph;
                                    foreground = item.renderable.color;
                                    background = item.renderable.background;
                                }
                                None => {
                                    glyph = rltk::to_cp437(' ');
                                    foreground = RGB::from_f32(0.25, 0.25, 0.25);
                                }
                            }
                        }
                    };
                },
                TileType::ClosedDoor => {
                    glyph = rltk::to_cp437('■');
                    foreground = RGB::from_f32(0.0, 1.0, 0.0);
                },
            }
            if !map.visible_tiles[idx] {
                foreground = foreground.to_greyscale();
            }
            ctx.set(x, y, foreground, background, glyph);
        }
        x += 1;
        if x >= right - left {
            x = 0;
            y += 1;
        }
    }
}

pub fn draw_menu(state: &State, context: &mut Rltk, monotime: u128) {
    let show_cursor = (monotime / 250) % 2 == 0;
    for menu in &state.menu_stack {
        menu.draw(context, show_cursor);
    }
}