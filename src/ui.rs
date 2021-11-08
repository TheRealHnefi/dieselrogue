use rltk::{RGB, Rltk};
use super::*;
use std::cmp::max;

pub const SCREEN_WIDTH: usize = 80;
pub const SCREEN_HEIGHT: usize = 50;

pub fn draw_main_screen(state: &mut State, context: &mut Rltk) {
    draw_map(&state.world.map, context);
    draw_main_ui(state, context);
}

fn draw_main_ui(state: &mut State, context: &mut Rltk) {
    context.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    if state.run_state == RunState::AwaitingPositionalTargetingInput {
        context.set_bg(state.cursor_pos.x, state.cursor_pos.y, RGB::named(rltk::PINK));
    }

    let mut y = 44;
    let length = max(state.log.entries.len() as i32 - 5, 0) as usize;
    for message in &state.log.entries[length..] {
        context.print(2, y, message);
        y += 1;
    }
}

fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // TODO - remove true
        if true || map.revealed_tiles[idx] {
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
                                    foreground = RGB::from_f32(0.25, 0.25, 0.25);
                                }
                            }
                        }
                    };
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('█');
                    foreground = RGB::from_f32(0.0, 1.0, 0.0);
                }
            }
            if !map.visible_tiles[idx] {
                foreground = foreground.to_greyscale();
            }
            ctx.set(x, y, foreground, background, glyph);
        }
        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

pub fn draw_menu(state: &State, context: &mut Rltk) {
    for menu in &state.menu_stack {
        menu.draw(context);
    }
}