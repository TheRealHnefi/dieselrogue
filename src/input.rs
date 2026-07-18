use rltk::{VirtualKeyCode, Rltk, Point};
use super::*;
use std::cmp::*;

pub fn welcome_screen_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let key = match state.last_input.take() {
        Some(k) => k,
        None => return RunState::WelcomeScreen,
    };

    match key {
        VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
            if state.welcome_selected > 0 {
                state.welcome_selected -= 1;
            }
            RunState::WelcomeScreen
        },
        VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
            if state.welcome_selected < 2 {
                state.welcome_selected += 1;
            }
            RunState::WelcomeScreen
        },
        VirtualKeyCode::Return | VirtualKeyCode::Space => {
            match state.welcome_selected {
                0 => RunState::WelcomeSplash,
                1 => {
                    state.menu_return_state = RunState::WelcomeScreen;
                    state.menu_stack.clear();
                    state.menu_stack.push(Box::new(settings_menu(state.pending_font_size, state.pending_fullscreen)));
                    RunState::AwaitingMenuInput
                },
                _ => std::process::exit(0),
            }
        },
        VirtualKeyCode::Escape => std::process::exit(0),
        _ => RunState::WelcomeScreen,
    }
}

pub fn game_over_input(state: &mut State, _context: &mut Rltk) -> RunState {
    match state.last_input.take() {
        None => RunState::GameOver,
        Some(_) => {
            let bindings = state.bindings;
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1);
            *state = State::new_welcome_state(seed, bindings);
            RunState::WelcomeScreen
        }
    }
}

pub fn welcome_splash_input(state: &mut State, _context: &mut Rltk) -> RunState {
    match state.last_input.take() {
        Some(_) => {
            let seed = state.seed;
            let bindings = state.bindings;
            *state = State::new_game_state(25, seed, false, bindings);
            RunState::AwaitingInput
        },
        None => RunState::WelcomeSplash,
    }
}

fn move_in_direction(state: &mut State, dir: Direction) -> RunState {
    if state.freelook {
        freelook_move(state, dir)
    } else if state.strafe_held {
        handle_strafe_input(&mut state.world, dir, &mut state.log)
    } else {
        handle_move_input(&mut state.world, dir, &mut state.log)
    }
}

#[tracing::instrument(skip_all)]
pub fn main_screen_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let b = state.bindings;
    // Clear any lingering menu-return target (e.g. from inventory browsing) so a
    // later menu's Esc returns here rather than to a stale state.
    state.menu_return_state = RunState::AwaitingInput;
    match state.last_input.take() {
        Some(key) => match key {
            // Hardcoded numpad movement — always active regardless of bindings
            VirtualKeyCode::Numpad4 => return move_in_direction(state, Direction::Left),
            VirtualKeyCode::Numpad6 => return move_in_direction(state, Direction::Right),
            VirtualKeyCode::Numpad8 => return move_in_direction(state, Direction::Up),
            VirtualKeyCode::Numpad2 => return move_in_direction(state, Direction::Down),
            VirtualKeyCode::Numpad7 => return move_in_direction(state, Direction::UpLeft),
            VirtualKeyCode::Numpad9 => return move_in_direction(state, Direction::UpRight),
            VirtualKeyCode::Numpad3 => return move_in_direction(state, Direction::DownRight),
            VirtualKeyCode::Numpad1 => return move_in_direction(state, Direction::DownLeft),
            // Rebindable movement keys
            key if key == b.move_left        => return move_in_direction(state, Direction::Left),
            key if key == b.move_right       => return move_in_direction(state, Direction::Right),
            key if key == b.move_up          => return move_in_direction(state, Direction::Up),
            key if key == b.move_down        => return move_in_direction(state, Direction::Down),
            key if key == b.move_up_left     => return move_in_direction(state, Direction::UpLeft),
            key if key == b.move_up_right    => return move_in_direction(state, Direction::UpRight),
            key if key == b.move_down_right  => return move_in_direction(state, Direction::DownRight),
            key if key == b.move_down_left   => return move_in_direction(state, Direction::DownLeft),
            key if key == b.wait => {
                return RunState::Resolve(ExecutionPhase::Instant);
            },

            key if key == b.get_item => {
                match getitem_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve(ExecutionPhase::Instant),
                    Err(error) => {
                        state.log(error.message);
                        return RunState::AwaitingInput;
                    }
                }
            },

            key if key == b.disembark => {
                match disembark_player_intent(&mut state.world) {
                    Ok(_) => return RunState::Resolve(ExecutionPhase::Instant),
                    Err(error) => {
                        state.log(error.message);
                        return RunState::AwaitingInput;
                    }
                }
            },

            key if key == b.inventory => {
                state.menu_stack.clear();
                let has_items = state.world.get_player()
                    .map(|p| !p.body.inventory.is_empty())
                    .unwrap_or(false);
                if has_items {
                    state.inventory_selected = 0;
                    return RunState::BrowsingInventory;
                } else {
                    state.log("No usable items".to_string());
                    return RunState::AwaitingInput;
                }
            },

            key if key == b.equipment => {
                state.menu_stack.clear();
                match first_selectable_slot(&state.world) {
                    Some(idx) => {
                        state.equipment_selected = idx;
                        return RunState::BrowsingEquipment;
                    }
                    None => {
                        state.log("Nothing to unequip".to_string());
                        return RunState::AwaitingInput;
                    }
                }
            },

            key if key == b.ability => {
                state.menu_stack.clear();
                state.menu_stack.push(Box::new(ability_menu(&state.world)));
                return RunState::AwaitingMenuInput;
            },

            key if key == b.juke => return trigger_action_by_id(state, ActionId::Juke),

            key if key == b.look => {
                if let Ok(player) = state.world.get_player() {
                    state.cursor_pos = player.position;
                }
                return RunState::Looking;
            },

            key if key == b.open_menu => {
                state.menu_stack.clear();
                state.menu_stack.push(Box::new(main_menu()));
                return RunState::AwaitingMenuInput;
            },

            key if key == b.freelook => {
                state.freelook = !state.freelook;
                if state.freelook {
                    state.freelook_pos = state.world.get_player()
                        .map(|p| p.center())
                        .unwrap_or(Point {x: 0, y: 0});
                    state.log("Freelook ON.".to_string());
                } else {
                    state.log("Freelook OFF.".to_string());
                }
                return RunState::AwaitingInput;
            },

            VirtualKeyCode::Key1 => {
                let options = state.world.compute_levelup_options();
                if !options.is_empty() {
                    state.level_up_options = options;
                    state.level_up_selected = 0;
                    return RunState::AwaitingLevelUpInput;
                }
                return RunState::AwaitingInput;
            },

            VirtualKeyCode::F10 => {
                state.world.parallel_ai = !state.world.parallel_ai;
                let msg = if state.world.parallel_ai { "Parallel AI ON." } else { "Parallel AI OFF." };
                state.log(msg.to_string());
                return RunState::AwaitingInput;
            }

            VirtualKeyCode::F12 => {
                state.world.debug_mode = !state.world.debug_mode;
                let msg = if state.world.debug_mode { "Debug mode ON." } else { "Debug mode OFF." };
                state.log(msg.to_string());
                return RunState::AwaitingInput;
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

pub fn positional_targeting_input(state: &mut State, _context: &mut Rltk) -> RunState {
    match state.last_input.take() {
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
                        let cursor_idx = state.world.map.pos_idx(state.cursor_pos);
                        match pending.entity_action.targeting {
                            Targeting::Positional { max_range } => {
                                // Reject a non-visible tile or one beyond the action's range.
                                let mut ok = state.world.map.visible_tiles[cursor_idx];
                                if let (true, Some(range), Ok(player)) = (ok, max_range, state.world.get_player()) {
                                    let dx = state.cursor_pos.x - player.position.x;
                                    let dy = state.cursor_pos.y - player.position.y;
                                    ok = dx * dx + dy * dy <= (range * range) as i32;
                                }
                                if !ok {
                                    state.pending_action = Some(pending);
                                    return RunState::AwaitingPositionalTargetingInput;
                                }
                                let cursor = state.cursor_pos;
                                let PendingAction { entity_action, source } = pending;
                                state.world.get_player_mut().unwrap().intent =
                                    build_intent(&entity_action, source, Resolution::Position(cursor));
                                return RunState::Resolve(ExecutionPhase::Instant);
                            },
                            Targeting::JumpTile => {
                                // Accept any previously-revealed Ground/Road tile (need not be visible).
                                let revealed = state.world.map.revealed_tiles[cursor_idx];
                                let jumpable = revealed && matches!(state.world.map.tiles[cursor_idx],
                                    TileType::Ground | TileType::Road);
                                if !jumpable {
                                    state.pending_action = Some(pending);
                                    return RunState::AwaitingPositionalTargetingInput;
                                }
                                let cursor = state.cursor_pos;
                                let PendingAction { entity_action, source } = pending;
                                state.world.get_player_mut().unwrap().intent =
                                    build_intent(&entity_action, source, Resolution::Position(cursor));
                                return RunState::Resolve(ExecutionPhase::Instant);
                            },
                            _ => {
                                // Detailed targeting — open the bodypart menu.
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

pub fn menu_input(state: &mut State, _context: &mut Rltk) -> RunState {
    assert!(!state.menu_stack.is_empty());
    let index = state.menu_stack.len() - 1;
    let menu = &mut state.menu_stack[index];

    match state.last_input.take() {
        Some(key) => match key {
            VirtualKeyCode::Escape => {
                state.menu_stack.pop();
                if state.menu_stack.is_empty() {
                    return state.menu_return_state;
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
                let result = match menu.get_action() {
                    MenuAction::Simple(action) => action(state),
                    MenuAction::WithPendingAction(pending) => {
                        if let Targeting::UseExistingAim { ask_bodypart } = pending.entity_action.targeting {
                            // Fire using the current aim status — no cursor step needed.
                            fire_from_aim(pending, ask_bodypart, state)
                        } else if let Targeting::EntityAim { max_range } = pending.entity_action.targeting {
                            start_entity_targeting(pending, max_range, state)
                        } else if let Targeting::Direction = pending.entity_action.targeting {
                            start_directional_action(state, pending.entity_action, pending.source)
                        } else if let Targeting::SelfBodypart = pending.entity_action.targeting {
                            start_self_bodypart_targeting(pending, state)
                        } else {
                            // Phase 1 of positional/detailed targeting: enter cursor mode.
                            // The flow continues in positional_targeting_input.
                            if let Ok(player) = state.world.get_player() {
                                state.cursor_pos = player.position;
                            }
                            state.pending_action = Some(pending);
                            RunState::AwaitingPositionalTargetingInput
                        }
                    },
                    MenuAction::WithIntent(intent, action) => action(intent, state),
                    MenuAction::WithItem(item, action) => action(item, state),
                    MenuAction::WithTargetedBodypartIndex(index, action) => action(index, state),
                };
                // A selected action leaves the menu system unless it opened another menu
                // (AwaitingMenuInput) or a keybind rebind. Otherwise drop the now-defunct
                // menu stack so it can't resurface later (e.g. when Look opens a menu).
                if !matches!(result, RunState::AwaitingMenuInput | RunState::AwaitingRebind(_, _)) {
                    state.menu_stack.clear();
                }
                return result;
            },
            _ => return RunState::AwaitingMenuInput
        }
        None => {
            return RunState::AwaitingMenuInput;
        }
    }
}

pub fn rebind_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let target = match state.run_state {
        RunState::AwaitingRebind(t, _) => t,
        _ => return RunState::AwaitingMenuInput,
    };

    let key = match state.last_input.take() {
        Some(k) => k,
        None => return state.run_state,
    };

    if key == VirtualKeyCode::Escape {
        return RunState::AwaitingMenuInput;
    }

    if let Some(conflict) = find_conflicting_action(&state.bindings, key, target) {
        return RunState::AwaitingRebind(target, Some(conflict));
    }

    match target {
        RebindTarget::Wait =>            state.bindings.wait = key,
        RebindTarget::GetItem =>         state.bindings.get_item = key,
        RebindTarget::Disembark =>       state.bindings.disembark = key,
        RebindTarget::Inventory =>       state.bindings.inventory = key,
        RebindTarget::Equipment =>       state.bindings.equipment = key,
        RebindTarget::Ability =>         state.bindings.ability = key,
        RebindTarget::Juke =>            state.bindings.juke = key,
        RebindTarget::Look =>            state.bindings.look = key,
        RebindTarget::OpenMenu =>        state.bindings.open_menu = key,
        RebindTarget::Freelook =>        state.bindings.freelook = key,
        RebindTarget::MoveLeft =>        state.bindings.move_left = key,
        RebindTarget::MoveRight =>       state.bindings.move_right = key,
        RebindTarget::MoveUp =>          state.bindings.move_up = key,
        RebindTarget::MoveDown =>        state.bindings.move_down = key,
        RebindTarget::MoveUpLeft =>      state.bindings.move_up_left = key,
        RebindTarget::MoveUpRight =>     state.bindings.move_up_right = key,
        RebindTarget::MoveDownRight =>   state.bindings.move_down_right = key,
        RebindTarget::MoveDownLeft =>    state.bindings.move_down_left = key,
        RebindTarget::Strafe =>          state.bindings.strafe = key,
    }

    let mut settings = Settings::load();
    settings.bindings = state.bindings;
    settings.save();

    let fresh = Box::new(keybind_menu(&state.bindings));
    state.menu_stack.pop();
    state.menu_stack.push(fresh);
    RunState::AwaitingMenuInput
}

fn find_conflicting_action(bindings: &Bindings, key: VirtualKeyCode, excluding: RebindTarget) -> Option<&'static str> {
    let checks: [(RebindTarget, VirtualKeyCode, &'static str); 19] = [
        (RebindTarget::Wait,          bindings.wait,           "Wait"),
        (RebindTarget::GetItem,       bindings.get_item,       "Get item"),
        (RebindTarget::Disembark,     bindings.disembark,      "Disembark"),
        (RebindTarget::Inventory,     bindings.inventory,      "Inventory"),
        (RebindTarget::Equipment,     bindings.equipment,      "Equipment"),
        (RebindTarget::Ability,       bindings.ability,        "Ability"),
        (RebindTarget::Juke,          bindings.juke,           "Juke"),
        (RebindTarget::Look,          bindings.look,           "Look"),
        (RebindTarget::OpenMenu,      bindings.open_menu,      "Open menu"),
        (RebindTarget::Freelook,      bindings.freelook,       "Freelook"),
        (RebindTarget::MoveLeft,      bindings.move_left,      "Move left"),
        (RebindTarget::MoveRight,     bindings.move_right,     "Move right"),
        (RebindTarget::MoveUp,        bindings.move_up,        "Move up"),
        (RebindTarget::MoveDown,      bindings.move_down,      "Move down"),
        (RebindTarget::MoveUpLeft,    bindings.move_up_left,   "Move up-left"),
        (RebindTarget::MoveUpRight,   bindings.move_up_right,  "Move up-right"),
        (RebindTarget::MoveDownRight, bindings.move_down_right,"Move down-right"),
        (RebindTarget::MoveDownLeft,  bindings.move_down_left, "Move down-left"),
        (RebindTarget::Strafe,        bindings.strafe,         "Strafe"),
    ];
    for (t, bound_key, name) in &checks {
        if *t != excluding && *bound_key == key {
            return Some(name);
        }
    }
    None
}

/// Resolve a fire action using the player's current `AimingAtGround` status.
/// Called instead of entering cursor mode when the action has `Targeting::UseExistingAim`.
///
/// If `ask_bodypart` is true and an entity occupies the aimed tile, opens the bodypart
/// selection menu (Phase 2b of the targeting flow) before resolving.
/// Otherwise fires directly at the aimed position.
fn fire_from_aim(pending: PendingAction, ask_bodypart: bool, state: &mut State) -> RunState {
    let aim_pos = match state.world.get_player_aim_position() {
        Some(pos) => pos,
        None => {
            state.log("Not aiming at anything.".to_string());
            return RunState::AwaitingMenuInput;
        }
    };

    state.cursor_pos = aim_pos;

    if ask_bodypart {
        if let Some(menu) = targeting_menu(&state.world, aim_pos) {
            state.menu_stack.clear();
            state.pending_action = Some(pending);
            state.menu_stack.push(Box::new(menu));
            return RunState::AwaitingMenuInput;
        }
    }

    // No bodypart menu needed (area weapon or empty tile): fire directly at aim position.
    let PendingAction { entity_action, source } = pending;
    match state.world.get_player_mut() {
        Ok(player) => player.intent = build_intent(&entity_action, source, Resolution::Position(aim_pos)),
        Err(_) => return RunState::AwaitingInput,
    }
    RunState::Resolve(ExecutionPhase::Instant)
}

/// Open the bodypart menu on the acting entity's own body (SelfBodypart targeting,
/// e.g. a medkit). Selecting a part completes the intent via
/// `action_apply_intent_to_target_bodypart`, using the player's own position.
fn start_self_bodypart_targeting(pending: PendingAction, state: &mut State) -> RunState {
    let pos = match state.world.get_player() {
        Ok(player) => player.position,
        Err(_) => return RunState::AwaitingInput,
    };
    state.cursor_pos = pos;
    match targeting_menu(&state.world, pos) {
        Some(menu) => {
            state.menu_stack.clear();
            state.pending_action = Some(pending);
            state.menu_stack.push(Box::new(menu));
            RunState::AwaitingMenuInput
        }
        None => RunState::AwaitingInput,
    }
}

/// Navigate the side-panel inventory list directly: up/down move the highlight,
/// Enter opens the item's action menu (returning here on cancel), Esc exits.
pub fn browse_inventory_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let inventory_len = state.world.get_player()
        .map(|p| p.body.inventory.len())
        .unwrap_or(0);
    if inventory_len == 0 {
        return RunState::AwaitingInput;
    }
    // Clamp defensively in case the inventory shrank since selection.
    if state.inventory_selected >= inventory_len {
        state.inventory_selected = inventory_len - 1;
    }

    let Some(key) = state.last_input.take() else {
        return RunState::BrowsingInventory;
    };

    match key {
        VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
            if state.inventory_selected > 0 {
                state.inventory_selected -= 1;
            }
            RunState::BrowsingInventory
        }
        VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
            if state.inventory_selected + 1 < inventory_len {
                state.inventory_selected += 1;
            }
            RunState::BrowsingInventory
        }
        VirtualKeyCode::Escape => RunState::AwaitingInput,
        VirtualKeyCode::Return | VirtualKeyCode::Space => {
            let item = state.world.get_player().unwrap()
                .body.inventory[state.inventory_selected].clone();
            let menu = inventory_action_menu(item, state);
            if menu.rows.is_empty() {
                state.log("No actions for that item".to_string());
                return RunState::BrowsingInventory;
            }
            state.menu_return_state = RunState::BrowsingInventory;
            state.menu_stack.push(Box::new(menu));
            RunState::AwaitingMenuInput
        }
        _ => RunState::BrowsingInventory,
    }
}

/// True when equipment slot `i` holds an item that can be unequipped (not proxy/locked).
fn slot_selectable(world: &World, i: usize) -> bool {
    world.get_player().ok()
        .and_then(|p| p.body.item_slots.get(i))
        .and_then(|s| s.item.as_ref())
        .map(|it| !it.proxy && !it.locked)
        .unwrap_or(false)
}

fn first_selectable_slot(world: &World) -> Option<usize> {
    let len = world.get_player().map(|p| p.body.item_slots.len()).unwrap_or(0);
    (0..len).find(|&i| slot_selectable(world, i))
}

/// Navigate the side-panel equipment list directly: up/down move the highlight over
/// unequippable slots, Enter opens the slot's action menu (returning here on cancel),
/// Esc exits.
pub fn browse_equipment_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let slots_len = state.world.get_player()
        .map(|p| p.body.item_slots.len())
        .unwrap_or(0);
    if slots_len == 0 {
        return RunState::AwaitingInput;
    }
    if state.equipment_selected >= slots_len {
        state.equipment_selected = slots_len - 1;
    }

    let Some(key) = state.last_input.take() else {
        return RunState::BrowsingEquipment;
    };

    match key {
        VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
            if let Some(prev) = (0..state.equipment_selected).rev().find(|&i| slot_selectable(&state.world, i)) {
                state.equipment_selected = prev;
            }
            RunState::BrowsingEquipment
        }
        VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
            if let Some(next) = ((state.equipment_selected + 1)..slots_len).find(|&i| slot_selectable(&state.world, i)) {
                state.equipment_selected = next;
            }
            RunState::BrowsingEquipment
        }
        VirtualKeyCode::Escape => RunState::AwaitingInput,
        VirtualKeyCode::Return | VirtualKeyCode::Space => {
            // Scope the borrow so the menu build + stack push below can reborrow state.
            let slot_type = {
                let player = state.world.get_player().unwrap();
                let slot = &player.body.item_slots[state.equipment_selected];
                match &slot.item {
                    Some(item) if !item.proxy && !item.locked => Some(slot.slot_type),
                    _ => None,
                }
            };
            let Some(slot_type) = slot_type else {
                return RunState::BrowsingEquipment;
            };
            let menu = equipment_action_menu(slot_type, state);
            state.menu_return_state = RunState::BrowsingEquipment;
            state.menu_stack.push(Box::new(menu));
            RunState::AwaitingMenuInput
        }
        _ => RunState::BrowsingEquipment,
    }
}

pub fn looking_input(state: &mut State, _context: &mut Rltk) -> RunState {
    if let Some(key) = state.last_input.take() {
        match key {
            VirtualKeyCode::Left  | VirtualKeyCode::Numpad4 => {
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
            },
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 => {
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width as i32 - 1);
            },
            VirtualKeyCode::Up    | VirtualKeyCode::Numpad8 => {
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
            },
            VirtualKeyCode::Down  | VirtualKeyCode::Numpad2 => {
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height as i32 - 1);
            },
            VirtualKeyCode::Numpad9 => {
                state.cursor_pos.x = min(state.cursor_pos.x + 1, state.world.map.width as i32 - 1);
                state.cursor_pos.y = max(state.cursor_pos.y - 1, 0);
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
                state.cursor_pos.x = max(state.cursor_pos.x - 1, 0);
                state.cursor_pos.y = min(state.cursor_pos.y + 1, state.world.map.height as i32 - 1);
            },
            VirtualKeyCode::Escape | VirtualKeyCode::L => {
                return RunState::AwaitingInput;
            },
            VirtualKeyCode::Return => {
                let idx = state.world.map.pos_idx(state.cursor_pos);
                let entity_id = state.world.map.pawns[idx].as_ref().map(|p| p.entity_id);
                if let Some(id) = entity_id {
                    if let Some(entity) = state.world.entities.get(id) {
                        let menu = entity_equipment_view(entity);
                        state.menu_stack.push(Box::new(menu));
                        return RunState::AwaitingMenuInput;
                    }
                }
                return RunState::AwaitingInput;
            },
            _ => {}
        }
    }
    RunState::Looking
}

/// Trigger a catalog action by id from a hotkey: look it up in the player's
/// available actions and drive the same resolution the menu uses. A missing
/// action (precondition failed / not available) is a silent no-op.
fn trigger_action_by_id(state: &mut State, id: ActionId) -> RunState {
    let found = {
        let Ok(player) = state.world.get_player() else { return RunState::AwaitingInput };
        player.get_available_actions(&state.world.map).into_iter()
            .find(|(a, _)| a.id == id)
            .map(|(a, slot)| (a.clone(), slot))
    };
    let (action, slot) = match found {
        Some(x) => x,
        None => return RunState::AwaitingInput,
    };
    let source = slot.map(ActionSource::EquippedSlot);
    match action.targeting {
        Targeting::None => {
            let intent = build_intent(&action, source, Resolution::None);
            if let Ok(player) = state.world.get_player_mut() { player.intent = intent; }
            RunState::Resolve(ExecutionPhase::Instant)
        },
        Targeting::Direction => start_directional_action(state, action, source),
        Targeting::UseExistingAim { ask_bodypart } =>
            fire_from_aim(PendingAction { entity_action: action, source }, ask_bodypart, state),
        Targeting::EntityAim { max_range } =>
            start_entity_targeting(PendingAction { entity_action: action, source }, max_range, state),
        Targeting::Positional { .. } | Targeting::Detailed | Targeting::JumpTile => {
            if let Ok(player) = state.world.get_player() {
                state.cursor_pos = player.position;
            }
            state.pending_action = Some(PendingAction { entity_action: action, source });
            RunState::AwaitingPositionalTargetingInput
        },
        Targeting::SelfBodypart =>
            start_self_bodypart_targeting(PendingAction { entity_action: action, source }, state),
    }
}

/// Stash a Direction-targeted action and prompt the player for a direction.
fn start_directional_action(state: &mut State, action: EntityAction, source: Option<ActionSource>) -> RunState {
    state.log(format!("{}: choose direction.", action.name));
    state.pending_action = Some(PendingAction { entity_action: action, source });
    RunState::AwaitingDirectionalTargetingInput
}

pub fn directional_targeting_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let key = match state.last_input.take() {
        Some(k) => k,
        None => return RunState::AwaitingDirectionalTargetingInput,
    };

    let dir = match key {
        VirtualKeyCode::Left  | VirtualKeyCode::Numpad4 => Direction::Left,
        VirtualKeyCode::Right | VirtualKeyCode::Numpad6 => Direction::Right,
        VirtualKeyCode::Up    | VirtualKeyCode::Numpad8 => Direction::Up,
        VirtualKeyCode::Down  | VirtualKeyCode::Numpad2 => Direction::Down,
        VirtualKeyCode::Numpad7 => Direction::UpLeft,
        VirtualKeyCode::Numpad9 => Direction::UpRight,
        VirtualKeyCode::Numpad3 => Direction::DownRight,
        VirtualKeyCode::Numpad1 => Direction::DownLeft,
        VirtualKeyCode::Escape  => { state.pending_action = None; return RunState::AwaitingInput; },
        _ => return RunState::AwaitingDirectionalTargetingInput,
    };

    let pending = match state.pending_action.take() {
        Some(p) => p,
        None => return RunState::AwaitingInput,
    };

    let (dx, dy) = dir.delta_pos();
    let player_pos = match state.world.get_player() {
        Ok(p) => p.position,
        Err(_) => return RunState::AwaitingInput,
    };
    let target = Point { x: player_pos.x + dx, y: player_pos.y + dy };

    let PendingAction { entity_action, source } = pending;
    let intent = build_intent(&entity_action, source, Resolution::Position(target));
    if let Ok(player) = state.world.get_player_mut() {
        player.intent = intent;
    }
    // Start from Idle so an Instant-phase action (juke) resolves before later phases.
    RunState::Resolve(ExecutionPhase::Idle)
}

pub fn level_up_input(state: &mut State, _context: &mut Rltk) -> RunState {
    let key = match state.last_input.take() {
        Some(k) => k,
        None => return RunState::AwaitingLevelUpInput,
    };

    let count = state.level_up_options.len();
    if count == 0 {
        return RunState::AwaitingInput;
    }

    match key {
        VirtualKeyCode::Up | VirtualKeyCode::Numpad8 => {
            if state.level_up_selected == 0 {
                state.level_up_selected = count - 1;
            } else {
                state.level_up_selected -= 1;
            }
            RunState::AwaitingLevelUpInput
        },
        VirtualKeyCode::Down | VirtualKeyCode::Numpad2 => {
            state.level_up_selected = (state.level_up_selected + 1) % count;
            RunState::AwaitingLevelUpInput
        },
        VirtualKeyCode::Return | VirtualKeyCode::Space => {
            let ability = state.level_up_options[state.level_up_selected].clone();
            add_levelup_ability(&mut state.world, ability);
            RunState::DeclareIntent
        },
        VirtualKeyCode::Escape => RunState::AwaitingInput,
        _ => RunState::AwaitingLevelUpInput,
    }
}

pub fn entity_targeting_input(state: &mut State, _context: &mut Rltk) -> RunState {
    if let Some(key) = state.last_input.take() {
        match key {
            VirtualKeyCode::Left  | VirtualKeyCode::Numpad4 |
            VirtualKeyCode::Up    | VirtualKeyCode::Numpad8 |
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Numpad9 => {
                if !state.entity_targets.is_empty() {
                    if state.entity_target_index == 0 {
                        state.entity_target_index = state.entity_targets.len() - 1;
                    } else {
                        state.entity_target_index -= 1;
                    }
                    sync_entity_cursor(state);
                }
            },
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 |
            VirtualKeyCode::Down  | VirtualKeyCode::Numpad2 |
            VirtualKeyCode::Numpad1 | VirtualKeyCode::Numpad3 => {
                if !state.entity_targets.is_empty() {
                    state.entity_target_index = (state.entity_target_index + 1) % state.entity_targets.len();
                    sync_entity_cursor(state);
                }
            },
            VirtualKeyCode::Return | VirtualKeyCode::Space => {
                return confirm_entity_target(state);
            },
            VirtualKeyCode::Escape => {
                state.pending_action = None;
                return RunState::AwaitingInput;
            },
            _ => {}
        }
    }
    RunState::AwaitingEntityTargetingInput
}

pub fn start_entity_targeting(pending: PendingAction, max_range: Option<u32>, state: &mut State) -> RunState {
    let targets = collect_entity_targets(&state.world, max_range);
    if targets.is_empty() {
        state.log("No targets in range.".to_string());
        return RunState::AwaitingInput;
    }
    state.entity_targets = targets;
    state.entity_target_index = 0;
    state.pending_action = Some(pending);
    sync_entity_cursor(state);
    RunState::AwaitingEntityTargetingInput
}

fn sync_entity_cursor(state: &mut State) {
    if let Some(&entity_id) = state.entity_targets.get(state.entity_target_index) {
        if let Some(entity) = state.world.entities.get(entity_id) {
            let center = entity.center();
            state.cursor_pos = center;
        }
    }
}

fn collect_entity_targets(world: &World, max_range: Option<u32>) -> Vec<usize> {
    let player = match world.get_player() {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let player_id = player.index;
    let player_center = player.center();

    let mut seen = std::collections::HashSet::new();
    let mut targets = vec![];

    for (idx, slot) in world.map.pawns.iter().enumerate() {
        let pawn = match slot { Some(p) => p, None => continue };
        if pawn.entity_id == player_id { continue; }
        if !world.map.visible_tiles[idx] { continue; }
        if !seen.insert(pawn.entity_id) { continue; }

        let entity = match world.entities.get(pawn.entity_id) {
            Some(e) => e,
            None => continue,
        };

        if let Some(range) = max_range {
            let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_center, entity.center());
            if dist > range as f32 { continue; }
        }

        targets.push(pawn.entity_id);
    }

    targets
}

fn confirm_entity_target(state: &mut State) -> RunState {
    let pending = match state.pending_action.take() {
        Some(p) => p,
        None => return RunState::AwaitingInput,
    };

    if state.entity_targets.is_empty() {
        state.log("No targets in range.".to_string());
        return RunState::AwaitingInput;
    }

    let entity_id = state.entity_targets[state.entity_target_index];

    let entity_center = match state.world.entities.get(entity_id) {
        Some(e) => e.center(),
        None => {
            state.log("Target no longer exists.".to_string());
            return RunState::AwaitingInput;
        }
    };

    let PendingAction { entity_action, source } = pending;
    let intent = build_intent(&entity_action, source, Resolution::Position(entity_center));
    match state.world.get_player_mut() {
        Ok(player) => player.intent = intent,
        Err(_) => return RunState::AwaitingInput,
    }

    RunState::Resolve(ExecutionPhase::Instant)
}

fn add_levelup_ability(world: &mut World, ability: Ability) {
    if let Ok(player) = world.get_player_mut() {
        let part_idx = ability.default_body_part();
        player.body.parts[part_idx].abilities.push(ability);
        player.body.update_abilities();
    }
}

fn handle_move_input(world: &mut World, direction: Direction, log: &mut GameLog) -> RunState {
    match move_player_intent(direction, world) {
        Ok(_) => RunState::Resolve(ExecutionPhase::Instant),
        Err(error) if matches!(error.error, Error::MapExit) => RunState::Victory,
        Err(error) => {
            log.log(error.message);
            RunState::AwaitingInput
        }
    }
}

fn handle_strafe_input(world: &mut World, direction: Direction, log: &mut GameLog) -> RunState {
    match strafe_player_intent(direction, world) {
        Ok(_) => RunState::Resolve(ExecutionPhase::Instant),
        Err(error) if matches!(error.error, Error::MapExit) => RunState::Victory,
        Err(error) => {
            log.log(error.message);
            RunState::AwaitingInput
        }
    }
}

pub fn victory_input(state: &mut State, _context: &mut Rltk) -> RunState {
    match state.last_input.take() {
        None => RunState::Victory,
        Some(_) => {
            let bindings = state.bindings;
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1);
            *state = State::new_welcome_state(seed, bindings);
            RunState::WelcomeScreen
        }
    }
}

fn freelook_move(state: &mut State, direction: Direction) -> RunState {
    let (dx, dy) = direction.delta_pos();
    let map_w = state.world.map.width as i32;
    let map_h = state.world.map.height as i32;
    state.freelook_pos.x = (state.freelook_pos.x + dx).clamp(0, map_w - 1);
    state.freelook_pos.y = (state.freelook_pos.y + dy).clamp(0, map_h - 1);
    RunState::AwaitingInput
}
