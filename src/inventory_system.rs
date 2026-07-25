use specs::prelude::*;
use super::{GettingItem, GettableItem, Position, GameLog, Name, InInventory, Map};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    type SystemData = (ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, GettingItem>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InInventory>);
                        
    fn run(&mut self, data: Self::SystemData) {
        let (_player, mut game_log, map, _names, mut item_getters, mut positions, mut _in_inventory_flags) = data;

        // TODO: Profile with large maps. This is setting off warning bells.
        let mut getter_collection = vec![];
        for (getter, position) in (&item_getters, &positions).join() {
            getter_collection.push((getter, Position {x: position.x, y: position.y }));
        }

        for (getter, position) in getter_collection {
            let index = map.xy_idx(position.x, position.y);
            match map.tile_items[index] {
                Some(item_pos) => {
                    let removed_position: Option<Position> = positions.remove(item_pos);
                    if removed_position.is_none() {
                        // TODO: Error handling maybe?
                        game_log.entries.push(format!("ERROR! ITEM POSITION NOT FOUND"));
                    }
                    game_log.entries.push(format!("Removed item from position {},{}", position.x, position.y));
                }
                None => {
                    game_log.entries.push(format!("Tried to pick up item but failed to find any"));
                }
            }
        }

        item_getters.clear();
    }

}