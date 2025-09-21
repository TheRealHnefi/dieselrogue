use rltk::{VirtualKeyCode, Rltk};
use super::*;
use std::cmp::*;

pub fn main_screen_input(state: &mut State, context: &mut Rltk) -> RunState {
    match context.key {
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 => {
                return handle_move_input(&mut state.world, Direction::Left, &mut state.log);
            },
            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 => {
                return handle_move_input(&mut state.world, Direction::Right, &mut state.log);
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                return handle_move_input(&mut state.world, Direction::Up, &mut state.log);
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                return handle_move_input(&mut state.world, Direction::Down, &mut state.log);
            },
            VirtualKeyCode::Numpad7 => {
                return handle_move_input(&mut state.world, Direction::UpLeft, &mut state.log);
            },
            VirtualKeyCode::Numpad9 => {
                return handle_move_input(&mut state.world, Direction::UpRight, &mut state.log);
            },
            VirtualKeyCode::Numpad3 => {
                return handle_move_input(&mut state.world, Direction::DownRight, &mut state.log);
            },
            VirtualKeyCode::Numpad1 => {
                return handle_move_input(&mut state.world, Direction::DownLeft, &mut state.log);
            },
            VirtualKeyCode::Numpad5 => {
                return RunState::Resolve(IntentPhase::Instant);
            },

            VirtualKeyCode::G => {
                match getitem_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve(IntentPhase::Instant),
                    Err(error) => {
                        state.log(error.message);
                        return RunState::AwaitingInput;
                    }
                }
            },

            VirtualKeyCode::D => {
                match disembark_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve(IntentPhase::Instant),
                    Err(error) => {
                        state.log(error.message);
                        return RunState::AwaitingInput;
                    }
                }
            },

            _ => {
                return RunState::AwaitingInput;
            }
        }
        None => {
            return RunState::AwaitingInput;
        }
    }
}

pub fn positional_targeting_input(state: &mut State, context: &mut Rltk) -> RunState {
    match context.key {
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 => {
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 => {
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width as i32 - 1);
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height as i32 - 1);
            },
            VirtualKeyCode::Numpad9 => {
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width as i32 - 1);
            },
            VirtualKeyCode::Numpad7 => {
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Numpad3 => {
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width as i32 - 1);
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height as i32 - 1);
            },
            VirtualKeyCode::Numpad1 => {
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height as i32 - 1);
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::Escape => {
                return RunState::AwaitingInput;
            },
            _ => {
              return RunState::AwaitingPositionalTargetingInput;
            }
        }
        None => return RunState::AwaitingPositionalTargetingInput
    }
    RunState::AwaitingPositionalTargetingInput
}

fn handle_move_input(world: &mut World, direction: Direction, log: &mut GameLog) -> RunState {
    match move_player_intent(direction, world) {
        Ok(_) => return RunState::Resolve(IntentPhase::Instant),
        Err(error) => {
            log.log(error.message);
            return RunState::AwaitingInput;    
        }
    }    
}
