use rltk::Point;
use crate::components::*;
use crate::error::*;
use crate::entity::*;
use crate::ability::*;
use crate::intent::*;
use crate::World;
use crate::TileType;

pub fn move_player_intent(direction: Direction, world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    
    let player_id = world.player_id.unwrap();
    let mut player = &mut world.entities[player_id];

    let driving = match player.driving {
        DrivingState::DrivenBy(_) => true,
        _ => false
    };

    if !player.has_ability(Ability::Move) {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player can not move")});
    }

    let (delta_x, delta_y);
    match direction {
        Direction::Up => {delta_x = 0; delta_y = -1},
        Direction::UpRight => {delta_x = 1; delta_y = -1},
        Direction::Right => {delta_x = 1; delta_y = 0},
        Direction::DownRight => {delta_x = 1; delta_y = 1},
        Direction::Down => {delta_x = 0; delta_y = 1},
        Direction::DownLeft => {delta_x = -1; delta_y = 1},
        Direction::Left => {delta_x = -1; delta_y = 0},
        Direction::UpLeft => {delta_x = -1; delta_y = -1},
    }

    if player.body.facing != direction {
        player.intent = Intent {
            phase: IntentPhase::Movement,
            data: IntentData::Direction(direction),
            action: Entity::resolve_turn
        };
    } else if !driving {
        let target_pos = Point {x: player.position.x + delta_x, y: player.position.y + delta_y};
        let index = world.map.xy_idx(target_pos.x, target_pos.y);
        match &world.map.pawns[index] {
            Some(pawn) => {
                if pawn.driving == DrivingState::Drivable {
                    player.intent = Intent {
                        phase: IntentPhase::Movement,
                        data: IntentData::Target(target_pos),
                        action: Entity::resolve_embark
                    };
                } else {
                    player.intent = Intent {
                        phase: IntentPhase::Attack,
                        data: IntentData::Target(target_pos),
                        action: Entity::resolve_melee
                    };
                }
            },
            None => {
                if world.map.tiles[index] == TileType::ClosedDoor {
                    player.intent = Intent {
                        phase: IntentPhase::Movement,
                        data: IntentData::Target(target_pos),
                        action: Entity::resolve_open_door
                    };
                } else {
                    player.intent = Intent {
                        phase: IntentPhase::Movement,
                        data: IntentData::Target(target_pos),
                        action: Entity::resolve_move
                    };
                }
            }
        }
    } else {
        let target_pos = Point {x: player.position.x + delta_x, y: player.position.y + delta_y};
        if player.check_fit(target_pos, &world.map) {
            player.intent = Intent {
                phase: IntentPhase::Movement,
                data: IntentData::Target(target_pos),
                action: Entity::resolve_move
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
    let mut player = &mut world.entities[player_id];

    let driving = match player.driving {
        DrivingState::DrivenBy(_) => true,
        _ => false
    };

    if !driving {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player is not driving a vehicle")});
    }

    player.intent = Intent {
        phase: IntentPhase::Movement,
        data: IntentData::Void,
        action: Entity::resolve_disembark
    };

    Ok(())
}

pub fn getitem_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    let mut player = &mut world.entities[world.player_id.unwrap()];
    if !player.has_ability(Ability::PickUp) {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player can not pick up items")});
    }

    let index = world.map.xy_idx(player.position.x, player.position.y);

    if world.map.items[index].is_some() {
        player.intent = Intent {
            phase: IntentPhase::Inventory,
            data: IntentData::Void,
            action: Entity::resolve_get_item
        };
        return Ok(());
    }

    Err(GameError{error: Error::BadPrecondition, message: String::from("There is no item here")})
}

pub fn get_item_actions(world: &World) -> Vec<IntentAction>{
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