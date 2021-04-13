use rltk::{VirtualKeyCode, Rltk, Point};
use super::*;
use std::cmp::{min, max};

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
                game_state.menu_stack.push(Menu::new_main());

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
            VirtualKeyCode::Numpad4 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.x = max(cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.y = max(cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
            },
            VirtualKeyCode::Numpad9 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.y = max(cursor_pos.y - 1, 0);
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
            },
            VirtualKeyCode::Numpad7 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                cursor_pos.x = max(cursor_pos.x - 1, 0);
                cursor_pos.y = max(cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Numpad3 => {
                let mut cursor_pos = game_state.ecs.fetch_mut::<Point>();
                let map = game_state.ecs.fetch::<Map>();
                cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
                cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
            },
            VirtualKeyCode::Numpad1 => {
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
    assert!(!game_state.menu_stack.is_empty(), "Menu stack is empty during menu input");
    match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                game_state.menu_stack.clear();
                return RunState::AwaitingInput;
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                let index = game_state.menu_stack.len() - 1;
                game_state.menu_stack[index].selected_row += 1;
                if game_state.menu_stack[index].selected_row > game_state.menu_stack[index].rows.len() - 1 {
                    game_state.menu_stack[index].selected_row = 0;
                }
                return RunState::MenuInput;
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                let index = game_state.menu_stack.len() - 1;
                if game_state.menu_stack[index].selected_row == 0 {
                    game_state.menu_stack[index].selected_row = game_state.menu_stack[index].rows.len() - 1;
                } else {
                    game_state.menu_stack[index].selected_row -= 1;
                }
                return RunState::MenuInput;
            },
            VirtualKeyCode::Space |
            VirtualKeyCode::Return => {
                let menu_index = game_state.menu_stack.len() - 1;
                let row_index = game_state.menu_stack[menu_index].selected_row;
                assert!(row_index < game_state.menu_stack[menu_index].rows.len(), "Row index out of bounds");
                return (game_state.menu_stack[menu_index].rows[row_index].action)(&mut game_state.ecs);
            },
            _ => {
                let rows = &game_state.menu_stack.last().unwrap().rows;
                for row in rows {
                    if row.hotkey == key {
                        return (row.action)(&mut game_state.ecs);
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