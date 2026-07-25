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

            VirtualKeyCode::Space => {
                state.system_menu_stack.clear();
                state.action_menu_stack.clear();
                state.action_menu_stack.push(ActionMenu::all_actions(&state.world));
                return RunState::AwaitingMenuInput;
            },

            VirtualKeyCode::Escape => {
                state.system_menu_stack.clear();
                state.action_menu_stack.clear();
                state.system_menu_stack.push(SystemMenu::main_menu());
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
                state.world.entities[state.world.player_id.unwrap()].intent =
                    Intent { action: Action::Throw(0, state.cursor_pos) };
                return RunState::Resolve;
            },
            _ => {
            }
        }
        None => {
            return RunState::AwaitingPositionalTargetingInput
        }
    }
    RunState::AwaitingPositionalTargetingInput
}

pub fn menu_input(state: &mut State, ctx: &mut Rltk) -> RunState {
    assert!(!state.system_menu_stack.is_empty() || !state.action_menu_stack.is_empty(),
        "Menu stack is empty during menu input");
    assert!(state.system_menu_stack.is_empty() || state.action_menu_stack.is_empty(),
        "Both menu systems active simultaneously");

    if !state.system_menu_stack.is_empty() {
        return system_menu_input(state, ctx);
    }
    else {
        return action_menu_input(state, ctx);
    }
}

fn system_menu_input(state: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                state.system_menu_stack.clear();
                state.action_menu_stack.clear();
                return RunState::AwaitingInput;
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                let index = state.system_menu_stack.len() - 1;
                state.system_menu_stack[index].selected_row += 1;
                if state.system_menu_stack[index].selected_row > state.system_menu_stack[index].rows.len() - 1 {
                    state.system_menu_stack[index].selected_row = 0;
                }
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                let index = state.system_menu_stack.len() - 1;
                if state.system_menu_stack[index].selected_row == 0 {
                    state.system_menu_stack[index].selected_row = state.system_menu_stack[index].rows.len() - 1;
                } else {
                    state.system_menu_stack[index].selected_row -= 1;
                }
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Space |
            VirtualKeyCode::Return => {
                let menu_index = state.system_menu_stack.len() - 1;
                let row_index = state.system_menu_stack[menu_index].selected_row;
                assert!(row_index < state.system_menu_stack[menu_index].rows.len(), "Row index out of bounds");
                let action = state.system_menu_stack[menu_index].rows[row_index].action;
                return action(&state.system_menu_stack[menu_index], &mut state.world);
            },
            _ => {
                let rows = &state.system_menu_stack.last().unwrap().rows;
                for row in rows {
                    if row.hotkey == key {
                        return (row.action)(&state.system_menu_stack.last().unwrap(), &mut state.world);
                    }
                }
                return RunState::AwaitingMenuInput;
            }
        }
        None => {
            return RunState::AwaitingMenuInput;
        }
    }
}

fn action_menu_input(state: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                state.system_menu_stack.clear();
                state.action_menu_stack.clear();
                return RunState::AwaitingInput;
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                let index = state.action_menu_stack.len() - 1;
                state.action_menu_stack[index].selected_row += 1;
                if state.action_menu_stack[index].selected_row > state.action_menu_stack[index].item_rows.len() - 1 {
                    state.action_menu_stack[index].selected_row = 0;
                }
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                let index = state.action_menu_stack.len() - 1;
                if state.action_menu_stack[index].selected_row == 0 {
                    state.action_menu_stack[index].selected_row = state.action_menu_stack[index].item_rows.len() - 1;
                } else {
                    state.action_menu_stack[index].selected_row -= 1;
                }
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Space |
            VirtualKeyCode::Return => {
                let menu_index = state.action_menu_stack.len() - 1;
                let row_index = state.action_menu_stack[menu_index].selected_row;
                assert!(row_index < state.action_menu_stack[menu_index].item_rows.len(), "Row index out of bounds");
                let action = &state.action_menu_stack[menu_index].item_rows[row_index].action.clone();
                return use_item(action, state);
            },
            _ => {
                // Keyboard shortcuts
                // let rows = &state.action_menu_stack.last().unwrap().item_rows;
                // for row in rows {
                //     if row.hotkey == key {
                //         return use_item(&row.action, state);
                //     }
                // }
                return RunState::AwaitingMenuInput;
            }
        }
        None => {
            return RunState::AwaitingMenuInput;
        }
    }
}

fn use_item(action: &ItemAction, state: &mut State) -> RunState {
    match action {
        ItemAction::Throw(_) => {
            state.cursor_pos = state.world.entities[state.world.player_id.unwrap()].position;
            return RunState::AwaitingPositionalTargetingInput;
        }
    }
}

// pub fn inventory_screen_input(state: &mut State, ctx: &mut Rltk) -> RunState {
//     match ctx.key {
//         Some(key) => {
//             let player = state.ecs.fetch::<Entity>();
//             let inventories = state.ecs.read_storage::<Inventory>();
//             let inventory = inventories.get(*player).unwrap();
//             let mut game_log = state.ecs.fetch_mut::<GameLog>();
            
//             match key {
//                 VirtualKeyCode::Escape |
//                 VirtualKeyCode::I => {
//                     return RunState::AwaitingInput;
//                 },
//                 VirtualKeyCode::Down |
//                 VirtualKeyCode::Numpad2 => {
//                     state.inventory_screen_selection = min(state.inventory_screen_selection + 1,
//                                                                 inventory.items.len() as i32 - 1);           
//                     return RunState::InventoryScreen;
//                 },
//                 VirtualKeyCode::Up |
//                 VirtualKeyCode::Numpad8 => {
//                     state.inventory_screen_selection = max(state.inventory_screen_selection - 1, 0);
//                     return RunState::InventoryScreen;
//                 },
//                 VirtualKeyCode::D => {
//                     match drop_item(&state.ecs, inventory.items[state.inventory_screen_selection as usize]) {
//                         Ok(_) => return RunState::PlayerTurn,
//                         Err(_) => {
//                             game_log.entries.push("Can't drop item. Is something in the way?".to_string());
//                             return RunState::AwaitingInput;
//                         }
//                     }
//                 },
//                 VirtualKeyCode::Space |
//                 VirtualKeyCode::Return => {
//                     match equip_item(&state.ecs, inventory.items[state.inventory_screen_selection as usize]) {
//                         Ok(_) => return RunState::PlayerTurn,
//                         Err(_) => {
//                             game_log.entries.push("Can't equip item".to_string());
//                             return RunState::AwaitingInput;
//                         }
//                     }
//                 },
//                 _ => {
//                     return RunState::InventoryScreen;
//                 }
//             }
//         }
//         None => {
//             return RunState::InventoryScreen;
//         }
//     }
// }