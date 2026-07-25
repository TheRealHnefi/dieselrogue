use rltk::Point;
use crate::components::*;
use crate::error::{Error, GameError};
use crate::entity::*;
use crate::ability::*;
use crate::intent::*;
use crate::World;

pub fn move_player_intent(direction: Direction, world: &mut World) -> Result<(), GameError> {
    let Some(player_id) = world.player_id else {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    };
    if let Some(intent) = resolve_step(&world.entities[player_id], direction, &world.map, &world.entities)? {
        world.entities[player_id].intent = intent;
    }
    Ok(())
}

// TODO: Strafe doesn't work with numpad keys. Why?
pub fn strafe_player_intent(direction: Direction, world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }
    let player_id = world.player_id.unwrap();

    if !world.entities[player_id].has_ability(Ability::HumanMove) {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player can not move")});
    }
    if matches!(world.entities[player_id].driving, DrivingState::DrivenBy(_)) {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Can not strafe while driving")});
    }

    let (delta_x, delta_y) = direction.delta_pos();
    let player_pos = world.entities[player_id].position;
    let target_pos = Point {x: player_pos.x + delta_x, y: player_pos.y + delta_y};

    if target_pos.x < 0 || target_pos.y < 0
        || target_pos.x >= world.map.width as i32
        || target_pos.y >= world.map.height as i32
    {
        return Err(GameError { error: Error::MapExit, message: String::new() });
    }
    let index = world.map.xy_idx(target_pos.x, target_pos.y);
    let pawn_entity_id = world.map.pawns[index].as_ref().map(|p| p.entity_id);

    if pawn_entity_id.is_some() || !world.entities[player_id].check_fit(target_pos, &world.map) {
        return Err(GameError{error: Error::BadPrecondition, message: "Bump!".to_string()});
    }
    world.entities[player_id].intent = move_intent(target_pos);
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

    player.intent = build_intent(&disembark_action_def(), None, Resolution::None);

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
        player.intent = build_intent(&get_item_action_def(), None, Resolution::None);
        return Ok(());
    }

    Err(GameError{error: Error::BadPrecondition, message: String::from("There is no item here")})
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