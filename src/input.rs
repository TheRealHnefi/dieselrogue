use rltk::{VirtualKeyCode, Rltk};
use super::*;

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

            VirtualKeyCode::Escape => {
                state.menu_stack.clear();
                state.menu_stack.push(Menu::new_main());
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

// pub fn targeting_input(state: &mut State, context: &mut Rltk) -> RunState {
//     match context.key {
//         Some(key) => match key {
//             VirtualKeyCode::Left |
//             VirtualKeyCode::Numpad4 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 cursor_pos.x = max(cursor_pos.x - 1, 0);
//             },
//             VirtualKeyCode::Right |
//             VirtualKeyCode::Numpad6 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 let map = state.ecs.fetch::<Map>();
//                 cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
//             },
//             VirtualKeyCode::Up |
//             VirtualKeyCode::Numpad8 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 cursor_pos.y = max(cursor_pos.y - 1, 0);
//             },
//             VirtualKeyCode::Down |
//             VirtualKeyCode::Numpad2 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 let map = state.ecs.fetch::<Map>();
//                 cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
//             },
//             VirtualKeyCode::Numpad9 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 cursor_pos.y = max(cursor_pos.y - 1, 0);
//                 let map = state.ecs.fetch::<Map>();
//                 cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
//             },
//             VirtualKeyCode::Numpad7 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 cursor_pos.x = max(cursor_pos.x - 1, 0);
//                 cursor_pos.y = max(cursor_pos.y - 1, 0);
//             },
//             VirtualKeyCode::Numpad3 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 let map = state.ecs.fetch::<Map>();
//                 cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
//                 cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
//             },
//             VirtualKeyCode::Numpad1 => {
//                 let mut cursor_pos = state.ecs.fetch_mut::<Point>();
//                 let map = state.ecs.fetch::<Map>();
//                 cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
//                 cursor_pos.x = max(cursor_pos.x - 1, 0);
//             },
//             VirtualKeyCode::Escape => {
//                 return RunState::AwaitingInput;
//             },
//             VirtualKeyCode::Space |
//             VirtualKeyCode::Return |
//             VirtualKeyCode::T => {
//                 let cursor_pos = state.ecs.fetch::<Point>();
//                 let map = state.ecs.fetch::<Map>();
//                 let index = map.xy_idx(cursor_pos.x, cursor_pos.y);
//                 let maybe_actor = map.tile_blockers[index];
//                 // TODO: Iterate over all entities with this position and, in case of >1 hit, create menu
//                 // to choose which to focus on
//                 match maybe_actor {
//                     Some(entity) => {
//                         state.menu_stack.clear();
//                         state.menu_stack.push(Menu::new_target_menu(&state.ecs, cursor_pos.x, cursor_pos.y, entity));
//                         return RunState::MenuInput;
//                     }
//                     None => {
//                         return RunState::AwaitingInput;
//                     }
//                 }
//             },
//             _ => {
//             }
//         }
//         None => {
//             return RunState::TargetingInput;
//         }
//     }
//     RunState::TargetingInput
// }

pub fn menu_input(state: &mut State, ctx: &mut Rltk) -> RunState {
    assert!(!state.menu_stack.is_empty(), "Menu stack is empty during menu input");
    match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                state.menu_stack.clear();
                return RunState::AwaitingInput;
            },
            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 => {
                let index = state.menu_stack.len() - 1;
                state.menu_stack[index].selected_row += 1;
                if state.menu_stack[index].selected_row > state.menu_stack[index].rows.len() - 1 {
                    state.menu_stack[index].selected_row = 0;
                }
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                let index = state.menu_stack.len() - 1;
                if state.menu_stack[index].selected_row == 0 {
                    state.menu_stack[index].selected_row = state.menu_stack[index].rows.len() - 1;
                } else {
                    state.menu_stack[index].selected_row -= 1;
                }
                return RunState::AwaitingMenuInput;
            },
            VirtualKeyCode::Space |
            VirtualKeyCode::Return => {
                let menu_index = state.menu_stack.len() - 1;
                let row_index = state.menu_stack[menu_index].selected_row;
                assert!(row_index < state.menu_stack[menu_index].rows.len(), "Row index out of bounds");
                let action = state.menu_stack[menu_index].rows[row_index].action;
                return action(&state.menu_stack[menu_index], &mut state.world);
            },
            _ => {
                let rows = &state.menu_stack.last().unwrap().rows;
                for row in rows {
                    if row.hotkey == key {
                        return (row.action)(&state.menu_stack.last().unwrap(), &mut state.world);
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