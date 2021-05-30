use legion::*;
use rltk::{VirtualKeyCode, Rltk};
use super::*;
use std::cmp::{min, max};

pub fn main_screen_input(state: &mut State, ctx: &mut Rltk) -> RunState {
    return match ctx.key {
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => player_move(state, Direction::Left),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => player_move(state, Direction::Right),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => player_move(state, Direction::Up),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => player_move(state, Direction::Down),

            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => player_move(state, Direction::UpRight),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => player_move(state, Direction::UpLeft),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => player_move(state, Direction::DownRight),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => player_move(state, Direction::DownLeft),

            VirtualKeyCode::Numpad5 => player_wait(state),

            // VirtualKeyCode::G => {
            //     let mut game_log = state.ecs.fetch_mut::<GameLog>();
            //     match get_item(&state.ecs) {
            //         Ok(_) => (),
            //         Err(_) => {
            //             game_log.entries.push("Can't pick up item".to_string());
            //             return RunState::AwaitingInput;
            //         }
            //     }
            // },

            VirtualKeyCode::I => RunState::InventoryInput,

            // VirtualKeyCode::T => {
            //     let player = *state.ecs.fetch::<Entity>();
            //     let positions = state.ecs.read_storage::<Position>();
            //     let player_pos = positions.get(player).expect("Could not get player position");
            //     let mut cursor_pos = state.ecs.fetch_mut::<Point>();
            //     cursor_pos.x = player_pos.x;
            //     cursor_pos.y = player_pos.y;
            //     return RunState::TargetingInput;
            // },

            VirtualKeyCode::Escape => {
                state.menu_stack.clear();
                state.menu_stack.push(Menu::new_main());
                RunState::MenuInput
            }

            _ => RunState::AwaitingInput
        }
        None => RunState::AwaitingInput
    }
}

pub fn targeting_input(_state: &mut State, _context: &mut Rltk) -> RunState {
    // match context.key {
    //     Some(key) => match key {
    //         VirtualKeyCode::Left |
    //         VirtualKeyCode::Numpad4 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             cursor_pos.x = max(cursor_pos.x - 1, 0);
    //         },
    //         VirtualKeyCode::Right |
    //         VirtualKeyCode::Numpad6 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             let map = state.ecs.fetch::<Map>();
    //             cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
    //         },
    //         VirtualKeyCode::Up |
    //         VirtualKeyCode::Numpad8 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             cursor_pos.y = max(cursor_pos.y - 1, 0);
    //         },
    //         VirtualKeyCode::Down |
    //         VirtualKeyCode::Numpad2 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             let map = state.ecs.fetch::<Map>();
    //             cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
    //         },
    //         VirtualKeyCode::Numpad9 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             cursor_pos.y = max(cursor_pos.y - 1, 0);
    //             let map = state.ecs.fetch::<Map>();
    //             cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
    //         },
    //         VirtualKeyCode::Numpad7 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             cursor_pos.x = max(cursor_pos.x - 1, 0);
    //             cursor_pos.y = max(cursor_pos.y - 1, 0);
    //         },
    //         VirtualKeyCode::Numpad3 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             let map = state.ecs.fetch::<Map>();
    //             cursor_pos.x = min(cursor_pos.x + 1, map.width - 1);
    //             cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
    //         },
    //         VirtualKeyCode::Numpad1 => {
    //             let mut cursor_pos = state.ecs.fetch_mut::<Point>();
    //             let map = state.ecs.fetch::<Map>();
    //             cursor_pos.y = min(cursor_pos.y + 1, map.height - 1);
    //             cursor_pos.x = max(cursor_pos.x - 1, 0);
    //         },
    //         VirtualKeyCode::Escape => {
    //             return RunState::AwaitingInput;
    //         },
    //         VirtualKeyCode::Space |
    //         VirtualKeyCode::Return |
    //         VirtualKeyCode::T => {
    //             let cursor_pos = state.ecs.fetch::<Point>();
    //             let map = state.ecs.fetch::<Map>();
    //             let index = map.xy_idx(cursor_pos.x, cursor_pos.y);
    //             let maybe_actor = map.tile_blockers[index];
    //             // TODO: Iterate over all entities with this position and, in case of >1 hit, create menu
    //             // to choose which to focus on
    //             match maybe_actor {
    //                 Some(entity) => {
    //                     state.menu_stack.clear();
    //                     state.menu_stack.push(Menu::new_target_menu(&state.ecs, cursor_pos.x, cursor_pos.y, entity));
    //                     return RunState::MenuInput;
    //                 }
    //                 None => {
    //                     return RunState::AwaitingInput;
    //                 }
    //             }
    //         },
    //         _ => {
    //         }
    //     }
    //     None => {
    //         return RunState::TargetingInput;
    //     }
    // }
    // RunState::TargetingInput
    RunState::AwaitingInput
}

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
                return RunState::MenuInput;
            },
            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 => {
                let index = state.menu_stack.len() - 1;
                if state.menu_stack[index].selected_row == 0 {
                    state.menu_stack[index].selected_row = state.menu_stack[index].rows.len() - 1;
                } else {
                    state.menu_stack[index].selected_row -= 1;
                }
                return RunState::MenuInput;
            },
            VirtualKeyCode::Space |
            VirtualKeyCode::Return => {
                let menu_index = state.menu_stack.len() - 1;
                let row_index = state.menu_stack[menu_index].selected_row;
                assert!(row_index < state.menu_stack[menu_index].rows.len(), "Row index out of bounds");
                return (state.menu_stack[menu_index].rows[row_index].action)(&state.menu_stack[menu_index], &mut state.ecs);
            },
            _ => {
                let rows = &state.menu_stack.last().unwrap().rows;
                for row in rows {
                    if row.hotkey == key {
                        return (row.action)(&state.menu_stack.last().unwrap(), &mut state.ecs);
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

pub fn inventory_screen_input(state: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        Some(key) => {
            
            // MIGRATION_TODO: Treat this as one case, not a loop of cases
            let mut query = <(&Inventory, &Player)>::query();
            for (inventory, _player) in query.iter(&state.ecs) {
                match key {
                    VirtualKeyCode::Escape |
                    VirtualKeyCode::I => {
                        return RunState::AwaitingInput;
                    },
                    VirtualKeyCode::Down |
                    VirtualKeyCode::Numpad2 => {
                        state.inventory_screen_selection = min(state.inventory_screen_selection + 1,
                                                            inventory.items.len() as i32 - 1);           
                        return RunState::InventoryInput;
                    },
                    VirtualKeyCode::Up |
                    VirtualKeyCode::Numpad8 => {
                        state.inventory_screen_selection = max(state.inventory_screen_selection - 1, 0);
                        return RunState::InventoryInput;
                    },
                    // VirtualKeyCode::D => {
                    //     match drop_item(&state.ecs, inventory.items[state.inventory_screen_selection as usize]) {
                    //         Ok(_) => return RunState::PlayerTurn,
                    //         Err(_) => {
                    //             game_log.entries.push("Can't drop item. Is something in the way?".to_string());
                    //             return RunState::AwaitingInput;
                    //         }
                    //     }
                    // },
                    // VirtualKeyCode::Space |
                    // VirtualKeyCode::Return => {
                    //     match equip_item(&state.ecs, inventory.items[state.inventory_screen_selection as usize]) {
                    //         Ok(_) => return RunState::PlayerTurn,
                    //         Err(_) => {
                    //             game_log.entries.push("Can't equip item".to_string());
                    //             return RunState::AwaitingInput;
                    //         }
                    //     }
                    // },
                    _ => {
                        return RunState::InventoryInput;
                    }
                }
            }
        }
        None => {
            return RunState::InventoryInput;
        }
    }
    RunState::InventoryInput
}