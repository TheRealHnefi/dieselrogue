use rltk::{VirtualKeyCode, Rltk};
use super::*;
use std::cmp::*;

pub fn main_screen_input(state: &mut State, context: &mut Rltk) -> RunState {
    match context.key {
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 => {
                if move_player_intent(Direction::Left, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 => {
                if move_player_intent(Direction::Right, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                if move_player_intent(Direction::Up, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                if move_player_intent(Direction::Down, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Numpad7 => {
                if move_player_intent(Direction::UpLeft, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Numpad9 => {
                if move_player_intent(Direction::UpRight, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Numpad3 => {
                if move_player_intent(Direction::DownRight, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },
            VirtualKeyCode::Numpad1 => {
                if move_player_intent(Direction::DownLeft, &mut state.world).is_ok() {
                    return RunState::Resolve;
                }
                else {
                    return RunState::AwaitingInput;
                }
            },

            VirtualKeyCode::G => {
                let result = getitem_player_intent(&mut state.world); 
                if result.is_ok() {
                    return RunState::Resolve;
                }
                else {
                    state.log.entries.push(result.err().unwrap().message);
                    return RunState::AwaitingInput;
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
                        state.log.entries.push("No usable items".to_string());
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
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width - 1);
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height - 1);
            },
            VirtualKeyCode::Numpad9 => {
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width - 1);
            },
            VirtualKeyCode::Numpad7 => {
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Numpad3 => {
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width - 1);
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height - 1);
            },
            VirtualKeyCode::Numpad1 => {
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height - 1);
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::Escape => {
                return RunState::AwaitingInput;
            },
            VirtualKeyCode::Return => {
                let player = state.world.get_player_mut().unwrap();

                match state.action_being_used.take() {
                    Some(action_in_use) => {
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
                    MenuAction::WithItem(item, action) => return action(item, state)
                }
            },
            _ => return RunState::AwaitingMenuInput
        }
        None => {
            return RunState::AwaitingMenuInput;
        }
    }
}
