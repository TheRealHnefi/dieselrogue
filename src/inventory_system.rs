use specs::prelude::*;
use super::{GettingItem, Position, GameLog, Name, Map, Inventory};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    type SystemData = (WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, GettingItem>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Inventory>);
                        
    fn run(&mut self, data: Self::SystemData) {
        let (mut game_log, map, names, mut item_getters, mut positions, mut inventories) = data;

        // TODO: Profile with large maps. This is setting off warning bells.
        let mut getter_collection = vec![];
        for (getter, position, name, inventory) in (&item_getters, &positions, &names, &mut inventories).join() {
            getter_collection.push((getter, name, Position {x: position.x, y: position.y }, inventory));
        }

        for (getter, name, position, inventory) in getter_collection {
            let index = map.xy_idx(position.x, position.y);
            match map.tile_items[index] {
                Some(item) => {
                    let removed_position: Option<Position> = positions.remove(item);
                    let item_name: Option<&Name> = names.get(item);
                    assert!(removed_position.is_some(), "Item position expected but not found");
                    assert!(item_name.is_some(), "Item name expected but not found");
                    game_log.entries.push(format!("{} picked up {}", name.value, item_name.unwrap().value));
                    inventory.items.push(item);
                }
                None => {
                    game_log.entries.push(format!("{} tried to pick up item but failed to find any", name.value));
                }
            }
        }

        item_getters.clear();
    }

}