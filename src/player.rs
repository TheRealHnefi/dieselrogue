use specs::prelude::*;
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

pub fn try_move_player(direction: Direction, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut facings = ecs.write_storage::<Facing>();
    let mut players = ecs.write_storage::<Player>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    let (delta_x, delta_y, glyph);
    match direction {
        Direction::Up => {delta_x = 0; delta_y = -1; glyph = rltk::to_cp437('8')},
        Direction::UpRight => {delta_x = 1; delta_y = -1; glyph = rltk::to_cp437('9')},
        Direction::Right => {delta_x = 1; delta_y = 0; glyph = rltk::to_cp437('6')},
        Direction::DownRight => {delta_x = 1; delta_y = 1; glyph = rltk::to_cp437('3')},
        Direction::Down => {delta_x = 0; delta_y = 1; glyph = rltk::to_cp437('2')},
        Direction::DownLeft => {delta_x = -1; delta_y = 1; glyph = rltk::to_cp437('1')},
        Direction::Left => {delta_x = -1; delta_y = 0; glyph = rltk::to_cp437('4')},
        Direction::UpLeft => {delta_x = -1; delta_y = -1; glyph = rltk::to_cp437('7')},
    }

    for (_player, pos, facing, renderable, viewshed) in
        (&mut players, &mut positions, &mut facings, &mut renderables, &mut viewsheds).join() {
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

pub fn get_item(ecs: &World) -> Result<(), GameError> {
    let player = *ecs.fetch::<Entity>();
    let map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let player_pos = positions.get(player).ok_or(())?;
    let index = map.xy_idx(player_pos.x, player_pos.y);

    if map.tile_items[index].is_some() {
        ecs.write_storage::<GettingItem>().insert(player, GettingItem {})?;
        return Ok(())
    }
    Err(GameError {})
}

pub fn drop_item(ecs: &World, item: Entity) -> Result<(), GameError> {
    let player = *ecs.fetch::<Entity>();
    let map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let player_pos = positions.get(player).ok_or(())?;
    let index = map.xy_idx(player_pos.x, player_pos.y);
    
    if map.tile_items[index].is_some() {
        return Err(GameError {});
    }
    ecs.write_storage::<DroppingItem>().insert(player, DroppingItem {item: item})?;
    Ok(())
}

pub fn instant_equip_item(ecs: &World, item: Entity) {
    let player = ecs.fetch::<Entity>();
    let mut bodies = ecs.write_storage::<HumanoidBody>();
    let body_maybe = bodies.get_mut(*player);
    match body_maybe {
        Some(body) => {
            let mut equippables = ecs.write_storage::<Equippable>();
            let equippable = equippables.get_mut(item).unwrap();
            let unequipped_item;
            match equippable.slot {
                ItemSlot::MainWeapon => {
                    unequipped_item = body.right_arm.equipped_item;
                    if body.right_arm.equipped_item.is_some() && body.right_arm.equipped_item.unwrap() == item {
                        body.right_arm.equipped_item = EntityOption::from(None);
                    } else {
                        equippable.equipped = true;
                        body.right_arm.equipped_item = EntityOption::<Entity>::from(Some(item));
                    }
                },
                ItemSlot::OffhandWeapon => {
                    unequipped_item = body.left_arm.equipped_item;
                    if body.left_arm.equipped_item.is_some() && body.left_arm.equipped_item.unwrap() == item {
                        body.left_arm.equipped_item = EntityOption::from(None);
                    } else {
                        equippable.equipped = true;
                        body.left_arm.equipped_item = EntityOption::<Entity>::from(Some(item));
                    }
                },
                ItemSlot::Head => {
                    unequipped_item = body.head.equipped_item;
                    if body.head.equipped_item.is_some() && body.head.equipped_item.unwrap() == item {
                        body.head.equipped_item = EntityOption::from(None);
                    } else {
                        equippable.equipped = true;
                        body.head.equipped_item = EntityOption::<Entity>::from(Some(item));
                    }
                },
                ItemSlot::Torso => {
                    unequipped_item = body.torso.equipped_item;
                    if body.torso.equipped_item.is_some() && body.torso.equipped_item.unwrap() == item {
                        body.torso.equipped_item = EntityOption::from(None);
                    } else {
                        equippable.equipped = true;
                        body.torso.equipped_item = EntityOption::<Entity>::from(Some(item));
                    }
                },
                ItemSlot::Legs => {
                    unequipped_item = body.legs.equipped_item;
                    if body.legs.equipped_item.is_some() && body.legs.equipped_item.unwrap() == item {
                        body.legs.equipped_item = EntityOption::from(None);
                    } else {
                        equippable.equipped = true;
                        body.legs.equipped_item = EntityOption::<Entity>::from(Some(item));
                    }
                }
            }

            match *unequipped_item {
                Some(unequip_entity) => {
                    let unequippable = equippables.get_mut(unequip_entity).unwrap();
                    unequippable.equipped = false;
                },
                None => ()
            }
        }
        None => {
            panic!("Player lacks body");
        }
    }
}

pub fn valid_actions(ecs: &World, target: Entity) -> Result<Vec<Action>, GameError> {
    let mut ret_val: Vec<Action> = Vec::new();

    let player = *ecs.fetch::<Entity>();
    let positions = ecs.read_storage::<Position>();
    let player_pos = positions.get(player).ok_or(())?;
    let target_pos = positions.get(target).ok_or(())?;
    let bodies = ecs.read_storage::<HumanoidBody>();
    let player_body = bodies.get(player).ok_or(())?;

    for action in Action::into_enum_iter() {
        match action {
            Action::Examine => ret_val.push(action),
            Action::Shoot => {
                let item_slot = *player_body.right_arm.equipped_item;
                match item_slot {
                    Some(item) => {
                        let firearms = ecs.read_storage::<Firearm>();
                        let firearm = firearms.get(item);
                        match firearm {
                            Some(_) => {
                                let distance = Pythagoras.distance2d(Point::new(target_pos.x, target_pos.y),
                                                                     Point::new(player_pos.x, player_pos.y));
                                if distance <= firearm.unwrap().range as f32 {
                                    ret_val.push(action);
                                    break;
                                }
                            },
                            None => ()
                        }
                    },
                    None => ()
                }
            }
        }
    }

    Ok(ret_val)
}