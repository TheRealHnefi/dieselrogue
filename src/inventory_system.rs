use specs::prelude::*;
use super::{GettingItem, DroppingItem, Equippable, Position, GameLog, Name, Map, Inventory};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    type SystemData = (WriteExpect<'a, GameLog>,
                        ReadExpect<'a, Map>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, Equippable>,
                        WriteStorage<'a, GettingItem>,
                        WriteStorage<'a, DroppingItem>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Inventory>);
                        
    fn run(&mut self, data: Self::SystemData) {
        let (mut game_log, map, names, equippables, mut item_getters, mut item_droppers, mut positions, mut inventories) = data;

        // TODO: Profile with large maps. This is setting off warning bells.
        let mut getter_collection = vec![];
        for (getter, position, name, inventory) in (&item_getters, &positions, &names, &mut inventories).join() {
            getter_collection.push((getter, name, Position {x: position.x, y: position.y }, inventory));
        }

        for (_getter, name, position, inventory) in getter_collection {
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

        let mut dropper_collection = vec![];
        for (dropper, position, name, inventory) in (&item_droppers, &positions, &names, &mut inventories).join() {
            dropper_collection.push((dropper, name, Position {x: position.x, y: position.y }, inventory));
        }

        for (dropper, name, position, inventory) in dropper_collection {
            let pos_index = map.xy_idx(position.x, position.y);
            match map.tile_items[pos_index] {
                Some(_) => {
                    game_log.entries.push(format!("{} tried to drop item but something was in the way", name.value));
                }
                None => {
                    let is_equipped = match equippables.get(dropper.item) {
                        Some(eq) => eq.equipped,
                        None => false
                    };
                    if !is_equipped {
                        let inv_index = inventory.items.iter().position(|i| *i == dropper.item).expect("Dropped item not found in inventory");
                        inventory.items.swap_remove(inv_index);
                        positions.insert(dropper.item, position).expect("Dropped item could not be given a position");
                    } else {
                        game_log.entries.push(format!("{} tried to drop item before unequipping it", name.value));
                    }
                }
            }
        }

        item_droppers.clear();
    }

}