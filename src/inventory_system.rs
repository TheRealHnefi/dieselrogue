use legion::*;
use super::*;

#[system(for_each)]
pub fn inventory_action (position: &Position,
                         inventory: &mut Inventory,
                         intent: &mut Intent,
                         world: &mut SubWorld,
                         #[resource] map: &mut Map,
                         #[resource] log: &mut GameLog) {
    match intent.action {
        Action::Get => {
            log.entries.push(format!("Picking up item"));
            let index = map.xy_idx(position.x, position.y);

            match map.tile_items[index] {
                Some(item) => {
                    map.tile_items[index] = None;
                    inventory.items.push(item);

                    let mut item_entry = world.entry_mut(item).unwrap();
                    let mut item_pos = item_entry.get_component_mut::<Position>().unwrap();
                    item_pos.valid = false;

                    log.entries.push(format!("Picked up item."));

                },
                None => {
                    log.entries.push(format!("Could not get item. Where did it go?"));
                    return;
                }
            }
            
            intent.action = Action::Idle;
        },
        _ => ()
    }
}

// pub struct InventorySystem {}

// impl<'a> System<'a> for InventorySystem {
//     type SystemData = (WriteExpect<'a, GameLog>,
//                         ReadExpect<'a, Map>,
//                         ReadStorage<'a, Name>,
//                         WriteStorage<'a, Equippable>,
//                         WriteStorage<'a, GettingItem>,
//                         WriteStorage<'a, DroppingItem>,
//                         WriteStorage<'a, EquippingItem>,
//                         WriteStorage<'a, HumanoidBody>,
//                         WriteStorage<'a, Position>,
//                         WriteStorage<'a, Inventory>);
                        
//     fn run(&mut self, data: Self::SystemData) {
//         let (mut game_log, map, names, mut equippables, mut item_getters, mut item_droppers, mut item_equippers, mut bodies, mut positions, mut inventories) = data;

//         // Get items
//         let mut getter_collection = vec![];
//         for (getter, position, name, inventory) in (&item_getters, &positions, &names, &mut inventories).join() {
//             getter_collection.push((getter, name, Position {x: position.x, y: position.y }, inventory));
//         }

//         for (_getter, name, position, inventory) in getter_collection {
//             let index = map.xy_idx(position.x, position.y);
//             match map.tile_items[index] {
//                 Some(item) => {
//                     let removed_position: Option<Position> = positions.remove(item);
//                     let item_name: Option<&Name> = names.get(item);
//                     assert!(removed_position.is_some(), "Item position expected but not found");
//                     assert!(item_name.is_some(), "Item name expected but not found");
//                     game_log.entries.push(format!("{} picked up {}", name.value, item_name.unwrap().value));
//                     inventory.items.push(item);
//                 }
//                 None => {
//                     game_log.entries.push(format!("{} tried to pick up item but failed to find any", name.value));
//                 }
//             }
//         }
//         item_getters.clear();

//         // Drop items
//         let mut dropper_collection = vec![];
//         for (dropper, position, name, inventory) in (&item_droppers, &positions, &names, &mut inventories).join() {
//             dropper_collection.push((dropper, name, Position {x: position.x, y: position.y }, inventory));
//         }

//         for (dropper, name, position, inventory) in dropper_collection {
//             let pos_index = map.xy_idx(position.x, position.y);
//             match map.tile_items[pos_index] {
//                 Some(_) => {
//                     game_log.entries.push(format!("{} tried to drop item but something was in the way", name.value));
//                 }
//                 None => {
//                     let is_equipped = match equippables.get(dropper.item) {
//                         Some(eq) => eq.equipped,
//                         None => false
//                     };
//                     if !is_equipped {
//                         let inv_index = inventory.items.iter().position(|i| *i == dropper.item).expect("Dropped item not found in inventory");
//                         inventory.items.swap_remove(inv_index);
//                         positions.insert(dropper.item, position).expect("Dropped item could not be given a position");
//                     } else {
//                         game_log.entries.push(format!("{} tried to drop item before unequipping it", name.value));
//                     }
//                 }
//             }
//         }
//         item_droppers.clear();

//         for (equipper, inventory, name, body) in (&item_equippers, &inventories, &names, &mut bodies).join() {
//             let equippable = equippables.get_mut(equipper.item).unwrap();
//             let equipped_name = &names.get(equipper.item).unwrap().value;
//             let unequipped_item;
//             if inventory.items.iter().any(|&i| i == equipper.item) {
//                 match equippable.slot {
//                     ItemSlot::MainWeapon => {
//                         unequipped_item = body.right_arm.equipped_item;
//                         if body.right_arm.equipped_item.is_some() && body.right_arm.equipped_item.unwrap() == equipper.item {
//                             body.right_arm.equipped_item = EntityOption::from(None);
//                         } else {
//                             equippable.equipped = true;
//                             body.right_arm.equipped_item = EntityOption::<Entity>::from(Some(equipper.item));
//                             game_log.entries.push(format!("{} equipped {} in their right arm", name.value, equipped_name));
//                         }
//                     },
//                     ItemSlot::OffhandWeapon => {
//                         unequipped_item = body.left_arm.equipped_item;
//                         if body.left_arm.equipped_item.is_some() && body.left_arm.equipped_item.unwrap() == equipper.item {
//                             body.left_arm.equipped_item = EntityOption::from(None);
//                         } else {
//                             equippable.equipped = true;
//                             body.left_arm.equipped_item = EntityOption::<Entity>::from(Some(equipper.item));
//                             game_log.entries.push(format!("{} equipped {} in their left arm", name.value, equipped_name));
//                         }
//                     },
//                     ItemSlot::Head => {
//                         unequipped_item = body.head.equipped_item;
//                         if body.head.equipped_item.is_some() && body.head.equipped_item.unwrap() == equipper.item {
//                             body.head.equipped_item = EntityOption::from(None);
//                         } else {
//                             equippable.equipped = true;
//                             body.head.equipped_item = EntityOption::<Entity>::from(Some(equipper.item));
//                             game_log.entries.push(format!("{} equipped {} on their head", name.value, equipped_name));
//                         }
//                     },
//                     ItemSlot::Torso => {
//                         unequipped_item = body.torso.equipped_item;
//                         if body.torso.equipped_item.is_some() && body.torso.equipped_item.unwrap() == equipper.item {
//                             body.torso.equipped_item = EntityOption::from(None);
//                         } else {
//                             equippable.equipped = true;
//                             body.torso.equipped_item = EntityOption::<Entity>::from(Some(equipper.item));
//                             game_log.entries.push(format!("{} equipped {} on their torso", name.value, equipped_name));
//                         }
//                     },
//                     ItemSlot::Legs => {
//                         unequipped_item = body.legs.equipped_item;
//                         if body.legs.equipped_item.is_some() && body.legs.equipped_item.unwrap() == equipper.item {
//                             body.legs.equipped_item = EntityOption::from(None);
//                         } else {
//                             equippable.equipped = true;
//                             body.legs.equipped_item = EntityOption::<Entity>::from(Some(equipper.item));
//                             game_log.entries.push(format!("{} equipped {} on their legs", name.value, equipped_name));
//                         }
//                     }
//                 }

//                 match *unequipped_item {
//                     Some(unequip_entity) => {
//                         let unequippable = equippables.get_mut(unequip_entity).unwrap();
//                         unequippable.equipped = false;
//                         game_log.entries.push(format!("{} unequipped {}", name.value, names.get(unequip_entity).unwrap().value));
//                     },
//                     None => ()
//                 }
//             } else {
//                 game_log.entries.push(format!("{} tried to equip {} without carrying it", name.value, equipped_name));
//             }
//         }
//         item_equippers.clear();
//     }
// }
