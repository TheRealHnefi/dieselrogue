use super::*;

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

    if player.body.facing != direction {
        player.intent = Intent {
            phase: IntentPhase::Movement,
            data: IntentData::Direction(direction),
            action: Entity::resolve_turn
        };
    } else {
        let target_pos = Point {x: player.body.position.x + delta_x, y: player.body.position.y + delta_y};
        let index = world.map.xy_idx(target_pos.x, target_pos.y);

        if world.map.pawns[index].is_some() {
            player.intent = Intent {
                phase: IntentPhase::Attack,
                data: IntentData::Target(target_pos),
                action: Entity::resolve_melee
            };
        } else {
            player.intent = Intent {
                phase: IntentPhase::Movement,
                data: IntentData::Target(target_pos),
                action: Entity::resolve_move
            };
        }
    }

    Ok(())
}

pub fn getitem_player_intent(world: &mut World) -> Result<(), GameError> {
    if world.player_id.is_none() {
        return Err(GameError{error: Error::BadPrecondition, message: String::from("Player does not exist")});
    }

    let mut player = &mut world.entities[world.player_id.unwrap()];
    let index = world.map.xy_idx(player.body.position.x, player.body.position.y);

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

pub fn get_item_actions(world: &World) -> Vec<ItemAction>{
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