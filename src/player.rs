use legion::*;
use super::*;
use std::cmp::{min, max};
use rltk::DistanceAlg::*;
use rltk::Point;

pub fn player_wait(state: &mut State) -> RunState {
    if state.player.is_none() {
        return RunState::AwaitingInput;
    }
    let intent = Intent{ action: Action::Idle };

    match set_intent(&mut state.ecs, state.player.unwrap(), intent) {
        Ok(_) => return RunState::ExecuteTurn,
        Err(_) => return RunState::AwaitingInput,
    }
}

pub fn player_move(state: &mut State, direction: Direction) -> RunState {
    if state.player.is_none() {
        return RunState::AwaitingInput;
    }

    let player_entry = state.ecs.entry(state.player.unwrap()).unwrap();
    let player_facing = player_entry.into_component::<Facing>().unwrap();

    let intent;
    if player_facing.direction == direction {
        intent = Intent{ action: Action::Walk(direction) };
    } else {
        intent = Intent{ action: Action::Turn(direction) };
    }

    match set_intent(&mut state.ecs, state.player.unwrap(), intent) {
        Ok(_) => return RunState::ExecuteTurn,
        Err(_) => return RunState::AwaitingInput,
    }
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