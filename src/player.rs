use super::*;
//use std::cmp::{min, max};
//use rltk::DistanceAlg::*;
//use rltk::Point;

pub fn move_player_intent(direction: Direction, world: &mut World) -> Result<(), GameError>{
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

// pub fn get_item(ecs: &World) -> Result<(), GameError> {
//     let player = *ecs.fetch::<Entity>();
//     let map = ecs.fetch::<Map>();
//     let positions = ecs.read_storage::<Position>();
//     let player_pos = positions.get(player).ok_or(())?;
//     let index = map.xy_idx(player_pos.x, player_pos.y);

//     if map.tile_items[index].is_some() {
//         ecs.write_storage::<GettingItem>().insert(player, GettingItem {})?;
//         return Ok(())
//     }
//     Err(GameError {})
// }

// pub fn drop_item(ecs: &World, item: Entity) -> Result<(), GameError> {
//     let player = *ecs.fetch::<Entity>();
//     let map = ecs.fetch::<Map>();
//     let positions = ecs.read_storage::<Position>();
//     let player_pos = positions.get(player).ok_or(())?;
//     let index = map.xy_idx(player_pos.x, player_pos.y);

//     if map.tile_items[index].is_some() {
//         return Err(GameError {});
//     }
//     ecs.write_storage::<DroppingItem>().insert(player, DroppingItem {item: item})?;
//     Ok(())
// }

// pub fn equip_item(ecs: &World, item: Entity) -> Result<(), GameError> {
//     let player = *ecs.fetch::<Entity>();
//     ecs.write_storage::<EquippingItem>().insert(player, EquippingItem {item: item})?;
//     Ok(())
// }

// pub fn valid_actions(ecs: &World, target: Entity) -> Result<Vec<Action>, GameError> {
//     let mut ret_val: Vec<Action> = Vec::new();

//     let player = *ecs.fetch::<Entity>();
//     let positions = ecs.read_storage::<Position>();
//     let player_pos = positions.get(player).ok_or(())?;
//     let target_pos = positions.get(target).ok_or(())?;
//     let bodies = ecs.read_storage::<HumanoidBody>();
//     let player_body = bodies.get(player).ok_or(())?;

//     for action in Action::into_enum_iter() {
//         match action {
//             Action::Examine => ret_val.push(action),
//             Action::Shoot => {
//                 let item_slot = *player_body.right_arm.equipped_item;
//                 match item_slot {
//                     Some(item) => {
//                         let firearms = ecs.read_storage::<Firearm>();
//                         let firearm = firearms.get(item);
//                         match firearm {
//                             Some(_) => {
//                                 let distance = Pythagoras.distance2d(Point::new(target_pos.x, target_pos.y),
//                                                                      Point::new(player_pos.x, player_pos.y));
//                                 if distance <= firearm.unwrap().range as f32 {
//                                     ret_val.push(action);
//                                     break;
//                                 }
//                             },
//                             None => ()
//                         }
//                     },
//                     None => ()
//                 }
//             }
//         }
//     }

//     Ok(ret_val)
// }