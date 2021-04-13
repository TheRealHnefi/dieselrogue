use rltk::{VirtualKeyCode, Rltk, Point};
use specs::prelude::*;
use super::*;
use std::cmp::{min, max};

pub fn try_move_player(direction: Direction, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut facings = ecs.write_storage::<Facing>();
    let mut players = ecs.write_storage::<Player>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    let (delta_x, delta_y, glyph);
    match direction {
        Direction::UP => {delta_x = 0; delta_y = -1; glyph = rltk::to_cp437('8')},
        Direction::UPRIGHT => {delta_x = 1; delta_y = -1; glyph = rltk::to_cp437('9')},
        Direction::RIGHT => {delta_x = 1; delta_y = 0; glyph = rltk::to_cp437('6')},
        Direction::DOWNRIGHT => {delta_x = 1; delta_y = 1; glyph = rltk::to_cp437('3')},
        Direction::DOWN => {delta_x = 0; delta_y = 1; glyph = rltk::to_cp437('2')},
        Direction::DOWNLEFT => {delta_x = -1; delta_y = 1; glyph = rltk::to_cp437('1')},
        Direction::LEFT => {delta_x = -1; delta_y = 0; glyph = rltk::to_cp437('4')},
        Direction::UPLEFT => {delta_x = -1; delta_y = -1; glyph = rltk::to_cp437('7')},
    }

    for (_player, pos, facing, renderable, viewshed) in
        (&mut players, &mut positions, &mut facings, &mut renderables, &mut viewsheds).join() {
        if facing.direction == direction {
            let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            if !map.blocked_tiles[dest_idx] {
                pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                viewshed.dirty = true;
            }
        } else {
            facing.direction = direction;
            renderable.glyph = glyph;
            viewshed.dirty = true;
        }
    }
}

pub fn player_input(game_state: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(Direction::LEFT, &mut game_state.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(Direction::RIGHT, &mut game_state.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(Direction::UP, &mut game_state.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(Direction::DOWN, &mut game_state.ecs),

            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => try_move_player(Direction::UPRIGHT, &mut game_state.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => try_move_player(Direction::UPLEFT, &mut game_state.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => try_move_player(Direction::DOWNRIGHT, &mut game_state.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => try_move_player(Direction::DOWNLEFT, &mut game_state.ecs),

            VirtualKeyCode::Numpad5 => {},

            VirtualKeyCode::G => get_item(&mut game_state.ecs),

            VirtualKeyCode::T => {
                return RunState::TargetingInput;
            },

            VirtualKeyCode::Escape => {
                game_state.menu_stack.clear();

                fn quit_function(_ecs: &mut World) -> RunState {
                    ::std::process::exit(0);
                }

                fn close_function(_ecs: &mut World) -> RunState {
                    return RunState::AwaitingInput;
                }

                let quit_row = MenuRow {
                    hotkey: VirtualKeyCode::Q,
                    text: "(Q) Quit".to_string(),
                    functor: quit_function
                };
                let close_row = MenuRow {
                    hotkey: VirtualKeyCode::C,
                    text: "(C) Close Menu".to_string(),
                    functor: close_function
                };

                let main_menu = Menu {
                    x: 35,
                    y: 20,
                    rows: vec![close_row, quit_row]
                };
                game_state.menu_stack.push(main_menu);

                return RunState::MenuInput;
            }

            _ => {
                return RunState::AwaitingInput;
            }
        }
        None => {
            return RunState::AwaitingInput;
        }
    }
    RunState::PlayerTurn
}

pub fn targeting_input(game_state: &mut State, context: &mut Rltk) -> RunState {
    match context.key {
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.x = max(cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.y = max(cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
            },
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.y = max(cursor_pos.y - 1, 0);
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
            },
            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.x = max(cursor_pos.x - 1, 0);
                cursor_pos.y = max(cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
                cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
            },
            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
                cursor_pos.x = max(cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::T => {
                return RunState::AwaitingInput;
            }
            _ => {
            }
        }
        None => {
        }
    }
    RunState::TargetingInput
}

pub fn menu_input(game_state: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                game_state.menu_stack.clear();
                return RunState::AwaitingInput;
            },
            _ => {
                let rows = &game_state.menu_stack.last().unwrap().rows;
                for row in rows {
                    if row.hotkey == key {
                        return (row.functor)(&mut game_state.ecs);
                    }
                }
                return RunState::MenuInput;
            }
        }
        None => {
            return RunState::MenuInput;
        }
    }
}


fn get_item(ecs: &mut World) {
    let player = ecs.fetch::<Entity>();

    let mut get_actions = ecs.write_storage::<GettingItem>();
    get_actions.insert(*player, GettingItem {}).expect("Unable to perform Get action");
}