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
                return RunState::Resolve;
            },

            VirtualKeyCode::G => {
                match getitem_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve,
                    Err(error) => {
                        state.log(error.message);
                        return RunState::AwaitingInput;
                    }
                }
            },

            VirtualKeyCode::I => {
                state.menu_stack.clear();
                let maybe_menu = item_menu(&state.world);
                match maybe_menu {
                    Some(menu) => {
                        state.menu_stack.push(Box::new(menu));
                        return RunState::AwaitingMenuInput;
                    }
                    None => {
                        state.log("No usable items".to_string());
                        return RunState::AwaitingInput;
                    }
                }
            },

            VirtualKeyCode::E => {
                state.menu_stack.clear();
                state.menu_stack.push(Box::new(equipment_menu(&state.world)));
                return RunState::AwaitingMenuInput;
            },

            VirtualKeyCode::A => {
                state.menu_stack.clear();
                state.menu_stack.push(Box::new(ability_menu(&state.world)));
                return RunState::AwaitingMenuInput;
            },

            VirtualKeyCode::Escape => {
                state.menu_stack.clear();
                state.menu_stack.push(Box::new(main_menu()));
                return RunState::AwaitingMenuInput;
            }

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
            VirtualKeyCode::Return => {
                let player = state.world.get_player_mut().unwrap();

                match state.action_being_used.take() {
                    Some(action_in_use) => {

                        if action_in_use.targeting == Targeting::Positional {

                            match state.action_item.take() {
                                Some(item_in_use) => {
                                    let intent = Intent {
                                        phase: action_in_use.phase,
                                        data: IntentData::TargetWithInventory{item: item_in_use, target: state.cursor_pos},
                                        action: action_in_use.effects
                                    };
                                    player.intent = intent;
                                },
                                None => {
                                    match state.action_slot.take() {
                                        Some(slot_in_use) => {
                                            let intent = Intent {
                                                phase: action_in_use.phase,
                                                data: IntentData::TargetWithEquipment{slot: slot_in_use, target: state.cursor_pos},
                                                action: action_in_use.effects
                                            };
                                            player.intent = intent;
                                        },
                                        None => {
                                            let intent = Intent {
                                                phase: action_in_use.phase,
                                                data: IntentData::Target(state.cursor_pos),
                                                action: action_in_use.effects
                                            };
                                            player.intent = intent;
                                        }
                                    }
                                }
                            }
                            return RunState::Resolve;
                        }
                        else if action_in_use.targeting == Targeting::Detailed {
                            state.menu_stack.clear();
                            state.action_being_used = Some(action_in_use);
                            let maybe_menu = targeting_menu(&state.world, state.cursor_pos);
                            match maybe_menu {
                                Some(menu) => {
                                    state.menu_stack.push(Box::new(menu));
                                    return RunState::AwaitingMenuInput;
                                }
                                None => {
                                    state.log("No target".to_string());
                                    return RunState::AwaitingPositionalTargetingInput;
                                }
                            }
                        }
                    },
                    None => return RunState::AwaitingInput
                }
            },
            _ => {
            }
        }
        None => return RunState::AwaitingPositionalTargetingInput
    }
    RunState::AwaitingPositionalTargetingInput
}

pub fn menu_input(state: &mut State, context: &mut Rltk) -> RunState {
    assert!(!state.menu_stack.is_empty());
    let index = state.menu_stack.len() - 1;
    let menu = &mut state.menu_stack[index];

    match context.key {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                state.menu_stack.pop();
                if state.menu_stack.is_empty() {
                    return RunState::AwaitingInput;
                }
                else {
                    return RunState::AwaitingMenuInput;
                }
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                menu.select_next();
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                menu.select_previous();
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Space |
            VirtualKeyCode::Return => {
                match menu.get_action() {
                    MenuAction::Simple(action) => return action(state),
                    MenuAction::WithItemAction(item, itemaction, action) => return action(item, itemaction, state),
                    MenuAction::WithIntent(intent, action) => return action(intent, state),
                    MenuAction::WithItem(item, action) => return action(item, state),
                    MenuAction::WithTargetedBodypartIndex(index, action) => return action(index, state)
                }
            },
            _ => return RunState::AwaitingMenuInput
        }
        None => {
            return RunState::AwaitingMenuInput;
        }
    }
}

fn handle_move_input(world: &mut World, direction: Direction, log: &mut GameLog) -> RunState {
    match move_player_intent(direction, world) {
        Ok(_) => return RunState::Resolve,
        Err(error) => {
            log.log(error.message);
            return RunState::AwaitingInput;    
        }
    }    
}
