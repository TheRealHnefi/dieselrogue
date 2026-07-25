use rltk::Point;
use crate::components::*;
use crate::error::{Error, GameError};
use crate::entity::*;
use crate::ability::*;
use crate::intent::*;
use crate::actions;
use crate::Map;
use crate::World;

pub fn move_player_intent(direction: Direction, world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }

    let player_id = world.player_id.unwrap();

    let driving = matches!(world.entities[player_id].driving, DrivingState::DrivenBy(_));
    let can_move = world.entities[player_id].has_ability(Ability::HumanMove)
        || world.entities[player_id].has_ability(Ability::VehicleMove);

    if !can_move {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player can not move")});
    }

    let (delta_x, delta_y) = direction.delta_pos();
    let facing = world.entities[player_id].body.facing;
    let player_pos = world.entities[player_id].position;

    if facing != direction {
        world.entities[player_id].intent = Intent {
            phase: ExecutionPhase::Movement,
            data: IntentData::Direction(direction),
            action: actions::turn_action
        };
    } else if !driving {
        let target_pos = Point {x: player_pos.x + delta_x, y: player_pos.y + delta_y};
        if target_pos.x < 0 || target_pos.y < 0
            || target_pos.x >= world.map.width as i32
            || target_pos.y >= world.map.height as i32
        {
            return Err(GameError { error: Error::MapExit, message: String::new() });
        }
        let index = world.map.xy_idx(target_pos.x, target_pos.y);

        // Determine what intent to set based on what's in the target tile.
        // We must not hold a borrow on world.map.pawns when we later mutate world.entities,
        // so extract just the pawn_entity_id before doing entity lookups.
        let pawn_entity_id = world.map.pawns[index].as_ref().map(|p| p.entity_id);

        match pawn_entity_id {
            Some(pawn_entity_id) => {
                if world.entities[pawn_entity_id].kind == EntityKind::Door {
                    world.entities[player_id].intent = Intent {
                        phase: ExecutionPhase::Movement,
                        data: IntentData::Target(target_pos),
                        action: actions::open_door_action
                    };
                } else if world.entities[pawn_entity_id].driving == DrivingState::Drivable {
                    if !world.entities[player_id].has_ability(Ability::Embark) {
                        return Err(GameError{error: Error::BadPrecondition, message: "You don't know how to operate that vehicle.".to_string()});
                    }
                    world.entities[player_id].intent = Intent {
                        phase: ExecutionPhase::Movement,
                        data: IntentData::Target(target_pos),
                        action: actions::embark_action
                    };
                } else {
                    world.entities[player_id].intent = Intent {
                        phase: ExecutionPhase::Attack,
                        data: IntentData::Target(target_pos),
                        action: actions::melee_action
                    };
                }
            },
            None => {
                if !world.entities[player_id].check_fit(target_pos, &world.map) {
                    return Err(GameError{error: Error::BadPrecondition, message: "Bump!".to_string()});
                }
                world.entities[player_id].intent = Intent {
                    phase: ExecutionPhase::Movement,
                    data: IntentData::Target(target_pos),
                    action: actions::move_action
                };
            }
        }
    } else {
        let target_pos = Point {x: player_pos.x + delta_x, y: player_pos.y + delta_y};
        if world.entities[player_id].check_fit(target_pos, &world.map) {
            world.entities[player_id].intent = Intent {
                phase: ExecutionPhase::Movement,
                data: IntentData::Target(target_pos),
                action: actions::move_action
            };
        }
    }

    Ok(())
}

pub fn disembark_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    
    let player_id = world.player_id.unwrap();
    let player = &mut world.entities[player_id];

    let driving = match player.driving {
        DrivingState::DrivenBy(_) => true,
        _ => false
    };

    if !driving {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player is not driving a vehicle")});
    }

    player.intent = Intent {
        phase: ExecutionPhase::Movement,
        data: IntentData::Void,
        action: actions::disembark_action
    };

    Ok(())
}

pub fn getitem_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    let player = &mut world.entities[world.player_id.unwrap()];
    if !player.has_ability(Ability::PickUp) {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player can not pick up items")});
    }

    let index = world.map.xy_idx(player.position.x, player.position.y);

    if world.map.items[index].as_ref().map_or(false, |i| i.is_droppable()) {
        player.intent = Intent {
            phase: ExecutionPhase::Inventory,
            data: IntentData::Void,
            action: actions::get_item_action
        };
        return Ok(());
    }

    Err(GameError{error: Error::BadPrecondition, message: String::from("There is no item here")})
}

pub fn shout_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    let player = &mut world.entities[world.player_id.unwrap()];
    player.intent = Intent {
        phase: ExecutionPhase::Inventory,
        data: IntentData::Void,
        action: actions::shout_action,
    };
    Ok(())
}

pub fn iron_body_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    let player = &mut world.entities[world.player_id.unwrap()];
    player.intent = Intent {
        phase: ExecutionPhase::Inventory,
        data: IntentData::Void,
        action: actions::iron_body_action,
    };
    Ok(())
}

/// Returns all actions currently available to `entity`: equipped-item actions
/// whose preconditions pass, followed by innate actions whose preconditions pass.
/// `Option<SlotType>` is `Some(slot)` for equipped actions, `None` for innate ones.
/// This is the authoritative source used by both the player menu and the AI.
pub fn get_entity_available_actions<'a>(entity: &'a Entity, map: &Map) -> Vec<(&'a EntityAction, Option<SlotType>)> {
    let mut result = Vec::new();
    for slot in &entity.body.item_slots {
        let Some(item) = &slot.item else { continue };
        if item.proxy { continue; }
        for action in &item.equip_actions {
            if (action.precondition)(entity, map, Some(item)) {
                result.push((action, Some(slot.slot_type)));
            }
        }
    }
    for action in &entity.innate_actions {
        if (action.precondition)(entity, map, None) {
            result.push((action, None));
        }
    }
    result
}

pub fn get_item_actions(world: &World) -> Vec<EntityAction>{
    if world.player_id.is_none() {
        return vec!();
    }

    let player = &world.entities[world.player_id.unwrap()];

    // TODO: Filter duplicates?
    let mut valid_actions = vec!();
    for item in &player.body.inventory {
        valid_actions.append(&mut item.inventory_actions.clone());
    }

    return valid_actions;
}