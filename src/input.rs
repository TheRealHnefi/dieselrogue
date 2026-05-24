use rltk::{VirtualKeyCode, Rltk, Point};
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

            VirtualKeyCode::J => {
                let can_juke = state.world.get_player()
                    .map(|p| p.has_ability(Ability::Juke))
                    .unwrap_or(false);
                if can_juke {
                    state.log("Juke: choose direction.".to_string());
                    return RunState::AwaitingJukeInput;
                }
                return RunState::AwaitingInput;
            },

            VirtualKeyCode::L => {
                if let Ok(player) = state.world.get_player() {
                    state.cursor_pos = player.position;
                }
                return RunState::Looking;
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
                        if matches!(pending.item_action.targeting, Targeting::Positional { .. }) {
                            // Reject if cursor is on a non-visible tile.
                            let cursor_idx = state.world.map.pos_idx(state.cursor_pos);
                            if !state.world.map.visible_tiles[cursor_idx] {
                                state.pending_action = Some(pending);
                                return RunState::AwaitingPositionalTargetingInput;
                            }
                            // Reject if cursor is beyond the action's max range.
                            if let Targeting::Positional { max_range: Some(range) } = pending.item_action.targeting {
                                if let Ok(player) = state.world.get_player() {
                                    let dx = state.cursor_pos.x - player.position.x;
                                    let dy = state.cursor_pos.y - player.position.y;
                                    if dx * dx + dy * dy > (range * range) as i32 {
                                        state.pending_action = Some(pending);
                                        return RunState::AwaitingPositionalTargetingInput;
                                    }
                                }
                            }
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
                    MenuAction::WithPendingAction(pending) => {
                        if let Targeting::UseExistingAim { ask_bodypart } = pending.item_action.targeting {
                            // Fire using the current aim status — no cursor step needed.
                            return fire_from_aim(pending, ask_bodypart, state);
                        }
                        if let Targeting::EntityAim { max_range } = pending.item_action.targeting {
                            return start_entity_targeting(pending, max_range, state);
                        }
                        // Phase 1 of positional/detailed targeting: enter cursor mode.
                        // The flow continues in positional_targeting_input.
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
    let slot = match pending.source {
        Some(ActionSource::EquippedSlot(s)) => s,
        _ => unreachable!("fire_from_aim requires an equipped slot source"),
    };
    match state.world.get_player_mut() {
        Ok(player) => player.intent = Intent {
            phase: pending.item_action.phase,
            data: IntentData::TargetWithEquipment { slot, target: aim_pos },
            action: pending.item_action.action,
        },
        Err(_) => return RunState::AwaitingInput,
    }
    RunState::Resolve(ExecutionPhase::Instant)
}

pub fn looking_input(state: &mut State, context: &mut Rltk) -> RunState {
    if let Some(key) = context.key {
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

pub fn juke_direction_input(state: &mut State, context: &mut Rltk) -> RunState {
    let key = match context.key {
        Some(k) => k,
        None => return RunState::AwaitingJukeInput,
    };

    let dir = match key {
        VirtualKeyCode::Left  | VirtualKeyCode::Numpad4 => Some(Direction::Left),
        VirtualKeyCode::Right | VirtualKeyCode::Numpad6 => Some(Direction::Right),
        VirtualKeyCode::Up    | VirtualKeyCode::Numpad8 => Some(Direction::Up),
        VirtualKeyCode::Down  | VirtualKeyCode::Numpad2 => Some(Direction::Down),
        VirtualKeyCode::Numpad7 => Some(Direction::UpLeft),
        VirtualKeyCode::Numpad9 => Some(Direction::UpRight),
        VirtualKeyCode::Numpad3 => Some(Direction::DownRight),
        VirtualKeyCode::Numpad1 => Some(Direction::DownLeft),
        VirtualKeyCode::Escape  => return RunState::AwaitingInput,
        _ => return RunState::AwaitingJukeInput,
    };

    if let Some(dir) = dir {
        let (dx, dy) = match dir {
            Direction::Up        => ( 0, -1),
            Direction::UpRight   => ( 1, -1),
            Direction::Right     => ( 1,  0),
            Direction::DownRight => ( 1,  1),
            Direction::Down      => ( 0,  1),
            Direction::DownLeft  => (-1,  1),
            Direction::Left      => (-1,  0),
            Direction::UpLeft    => (-1, -1),
        };
        if let Ok(player) = state.world.get_player_mut() {
            let target = Point { x: player.position.x + dx, y: player.position.y + dy };
            player.intent = Intent {
                phase: ExecutionPhase::Instant,
                data: IntentData::Target(target),
                action: actions::juke_action,
            };
        }
        // Start the round from Idle so the Instant phase runs before Inventory/Attack/Movement.
        return RunState::Resolve(ExecutionPhase::Idle);
    }

    RunState::AwaitingJukeInput
}

pub fn level_up_input(state: &mut State, context: &mut Rltk) -> RunState {
    let key = match context.key {
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

pub fn entity_targeting_input(state: &mut State, context: &mut Rltk) -> RunState {
    if let Some(key) = context.key {
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
    let player_id = player.id;
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

    let slot = match pending.source {
        Some(ActionSource::EquippedSlot(s)) => s,
        _ => return RunState::AwaitingInput,
    };

    let entity_center = match state.world.entities.get(entity_id) {
        Some(e) => e.center(),
        None => {
            state.log("Target no longer exists.".to_string());
            return RunState::AwaitingInput;
        }
    };

    match state.world.get_player_mut() {
        Ok(player) => {
            player.intent = Intent {
                phase: pending.item_action.phase,
                data: IntentData::TargetWithEquipment { slot, target: entity_center },
                action: pending.item_action.action,
            };
        },
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
        Ok(_) => return RunState::Resolve(ExecutionPhase::Instant),
        Err(error) => {
            log.log(error.message);
            return RunState::AwaitingInput;    
        }
    }    
}
