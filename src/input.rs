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
                return RunState::Resolve(ExecutionPhase::Instant);
            },

            VirtualKeyCode::G => {
                match getitem_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve(ExecutionPhase::Instant),
                    Err(error) => {
                        state.log(error.message);
                        return RunState::AwaitingInput;
                    }
                }
            },

            VirtualKeyCode::D => {
                match disembark_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve(ExecutionPhase::Instant),
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
                // Phase 2 of targeting: cursor position confirmed.
                match state.pending_action.take() {
                    Some(pending) => {
                        if pending.item_action.targeting == Targeting::Positional {
                            // Phase 2a: assemble the intent directly from cursor position.
                            let data = match pending.source {
                                Some(ActionSource::InventoryItem(item)) =>
                                    IntentData::TargetWithInventory { item, target: state.cursor_pos },
                                Some(ActionSource::EquippedSlot(slot)) =>
                                    IntentData::TargetWithEquipment { slot, target: state.cursor_pos },
                                None =>
                                    IntentData::Target(state.cursor_pos),
                            };
                            state.world.get_player_mut().unwrap().intent = Intent {
                                phase: pending.item_action.phase,
                                data,
                                action: pending.item_action.action,
                            };
                            return RunState::Resolve(ExecutionPhase::Instant);
                        } else {
                            // Phase 2b: Detailed targeting — open the bodypart menu.
                            // action_apply_intent_to_target_bodypart will complete the intent.
                            state.menu_stack.clear();
                            state.pending_action = Some(pending);
                            match targeting_menu(&state.world, state.cursor_pos) {
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
                    None => return RunState::AwaitingInput,
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
                    // Phase 1 of targeting: store the pending action and enter cursor mode.
                    // The flow continues in positional_targeting_input.
                    MenuAction::WithPendingAction(pending) => {
                        if let Ok(player) = state.world.get_player() {
                            state.cursor_pos = player.position;
                        }
                        state.pending_action = Some(pending);
                        return RunState::AwaitingPositionalTargetingInput;
                    },
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
        Ok(_) => return RunState::Resolve(ExecutionPhase::Instant),
        Err(error) => {
            log.log(error.message);
            return RunState::AwaitingInput;    
        }
    }    
}
