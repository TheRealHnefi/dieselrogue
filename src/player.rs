use super::*;
//use std::cmp::{min, max};
//use rltk::DistanceAlg::*;
//use rltk::Point;

pub fn move_player_intent(direction: Direction, world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }

    let mut player = &mut world.entities[world.player_id.unwrap()];

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

    if player.facing.direction != direction {
        player.intent = Intent {action: Action::Turn(direction)};
    } else {
        let target_pos = Point {x: player.position.x + delta_x, y: player.position.y + delta_y};
        let index = world.map.xy_idx(target_pos.x, target_pos.y);

        if world.map.pawns[index].is_some() {
            player.intent = Intent {action: Action::Melee(target_pos)};
        } else {
            player.intent = Intent {action: Action::Move(target_pos)};
        }
    }

    Ok(())
}

pub fn getitem_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }

    let mut player = &mut world.entities[world.player_id.unwrap()];
    let index = world.map.xy_idx(player.position.x, player.position.y);

    if world.map.items[index].is_some() {
        player.intent = Intent {action: Action::GetItem};
        return Ok(());
    }

    Err(GameError{error: Error::BadPrecondition, message: String::from("There is no item here")})
}

pub fn get_item_actions(world: &World) -> Vec<ItemAction>{
    if world.player_id.is_none() {
        return vec!();
    }

    let player = &world.entities[world.player_id.unwrap()];

    // TODO: Filter duplicates?
    let mut valid_actions = vec!();
    for item in &player.inventory {
        valid_actions.append(&mut item.inventory_actions.clone());
    }

    return valid_actions;
}