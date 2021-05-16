use legion::*;
use super::*;
use std::cmp::{min, max};
use rltk::DistanceAlg::*;
use rltk::Point;
use enum_iterator::IntoEnumIterator;

#[derive(Clone, IntoEnumIterator, PartialEq)]
pub enum Action {
    Examine,
    Shoot
}

// TODO: Move this to a system?
pub fn try_move_player(direction: Direction, state: &mut State) {

    let map = state.resources.get::<Map>().unwrap();

    let (delta_x, delta_y, glyph);
    match direction {
        Direction::Up => {delta_x = 0; delta_y = -1; glyph = rltk::to_cp437('▲')},
        Direction::UpRight => {delta_x = 1; delta_y = -1; glyph = rltk::to_cp437('┐')},
        Direction::Right => {delta_x = 1; delta_y = 0; glyph = rltk::to_cp437('►')},
        Direction::DownRight => {delta_x = 1; delta_y = 1; glyph = rltk::to_cp437('┘')},
        Direction::Down => {delta_x = 0; delta_y = 1; glyph = rltk::to_cp437('▼')},
        Direction::DownLeft => {delta_x = -1; delta_y = 1; glyph = rltk::to_cp437('└')},
        Direction::Left => {delta_x = -1; delta_y = 0; glyph = rltk::to_cp437('◄')},
        Direction::UpLeft => {delta_x = -1; delta_y = -1; glyph = rltk::to_cp437('┌')},
    }

    let mut query = <(&mut Position, &mut Facing, &mut Viewshed, &mut Renderable, &Player)>::query();

    for (pos, facing, viewshed, renderable, _player) in query.iter_mut(&mut state.ecs) {
        if facing.direction == direction {
            let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            if !map.blocked_tiles[dest_idx] {
                pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                pos.y = min(map.height - 1, max(0, pos.y + delta_y));
                viewshed.dirty = true;
            }
        } else {
            facing.direction = direction;
            renderable.glyph = glyph;
            viewshed.dirty = true;
        }
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