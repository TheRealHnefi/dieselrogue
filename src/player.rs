use specs::prelude::*;
use super::*;
use std::cmp::{min, max};
use enum_iterator::IntoEnumIterator;

#[derive(Clone, IntoEnumIterator, PartialEq)]
pub enum Action {
    Examine,
    Throw,
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
        Direction::UP => {delta_x = 0; delta_y = -1; glyph = rltk::to_cp437('8')},
        Direction::UPRIGHT => {delta_x = 1; delta_y = -1; glyph = rltk::to_cp437('9')},
        Direction::RIGHT => {delta_x = 1; delta_y = 0; glyph = rltk::to_cp437('6')},
        Direction::DOWNRIGHT => {delta_x = 1; delta_y = 1; glyph = rltk::to_cp437('3')},
        Direction::DOWN => {delta_x = 0; delta_y = 1; glyph = rltk::to_cp437('2')},
        Direction::DOWNLEFT => {delta_x = -1; delta_y = 1; glyph = rltk::to_cp437('1')},
        Direction::LEFT => {delta_x = -1; delta_y = 0; glyph = rltk::to_cp437('4')},
        Direction::UPLEFT => {delta_x = -1; delta_y = -1; glyph = rltk::to_cp437('7')},
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

pub fn get_item(ecs: &mut World) {
    let player = *ecs.fetch::<Entity>();

    let mut get_actions = ecs.write_storage::<GettingItem>();
    get_actions.insert(player, GettingItem {}).expect("Unable to perform Get action");
}


pub fn valid_actions(ecs: &World, target: Entity) -> Result<Vec<Action>, ()> {
    let mut ret_val: Vec<Action> = Vec::new();

    let player = *ecs.fetch::<Entity>();
    let positions = ecs.read_storage::<Position>();
    let inventories = ecs.read_storage::<Inventory>();
    let _player_pos = positions.get(player).ok_or(())?;
    let _target_pos = positions.get(target).ok_or(())?;
    let player_inventory = inventories.get(player).ok_or(())?;

    for action in Action::into_enum_iter() {
        match action {
            Action::Examine => ret_val.push(action),
            Action::Throw => (),
            Action::Shoot => {
                let mut has_gun = false;
                for item in &*player_inventory.items {
                    let names = ecs.read_storage::<Name>();
                    let name = names.get(*item);
                    if name.unwrap().value == "Gun" {
                        has_gun = true;
                    }
                }
                if has_gun {
                    ret_val.push(action);
                }
            }
        }
    }

    Ok(ret_val)
}