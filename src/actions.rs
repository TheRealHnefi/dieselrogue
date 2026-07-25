use rltk::console;
use crate::Entity;
use crate::Map;
use crate::GameLog;
use crate::components::*;
use crate::intent::*;
use crate::animation::*;
use crate::ability::*;
use crate::DrivingState;

pub type Action = fn (entity_ref: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect>;

pub fn throw_grenade_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    log.log(format!("{} threw a grenade", entity.name));
    let mut result = vec!();

    let used_item;
    let target_map_index;
    let target_pos;
    match entity.intent.data.clone() {
        IntentData::TargetWithInventory{item, target} => {
            target_pos = target;
            used_item = item;
            target_map_index = map.pos_idx(target);
        },
        _ => {
            debug_assert!(false);
            return result;
        }
    }

    entity.take_item(used_item);
    
    match &map.pawns[target_map_index] {
        Some(pawn) => {
            for part_index in 0..pawn.body.parts.len() {
                result.push(Effect::Damage{
                    entity_id: pawn.entity_id,
                    bodypart_index: part_index,
                    raw_damage: Damage::new(5, 0, 0, 0)
                });
            }
        }
        _ => ()
    }

    result.push(Effect::Animation(explosion_animation(target_pos)));

    result
}

pub fn drop_item_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let inventory_item;
    match entity.intent.data.clone() {
        IntentData::InventoryItem(item) => {
            inventory_item = item;
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }

    log.log(format!("{} dropped {}", entity.name, inventory_item.name));

    let target_pos = map.nearest_free_item_position(entity.position).unwrap();
    let map_index = map.pos_idx(target_pos);

    map.items[map_index] = entity.take_item(inventory_item);

    vec!()
}

pub fn get_item_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let index = map.xy_idx(entity.position.x, entity.position.y);
    if map.items[index].is_some() {
        let item = map.items[index].take().unwrap();

        log.log(format!("{} picked up {}", entity.name, item.name));
        entity.body.inventory.push(item);
    }

    vec!()
}

pub fn equip_item_action(entity: &mut Entity, _map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let inventory_item;
    match entity.intent.data.clone() {
        IntentData::InventoryItem(item) => {
            inventory_item = item;
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }

    match entity.take_item(inventory_item) {
        Some(item) => {
            let unequipped_result = entity.body.equip(item.clone());
            log.log(format!("{} equipped {}", entity.name, item.name));
            match unequipped_result {
                Ok(unequipped_items) => {
                    for unequipped_item in unequipped_items {
                        log.log(format!("{} unequipped {}", entity.name, unequipped_item.name));
                        entity.body.inventory.push(unequipped_item);
                    }
                },
                Err(_) => {
                    entity.body.inventory.push(item);
                }
            }
        }
        None => {
            debug_assert!(false);
            return vec!();
        }
    }

    entity.body.update_armor();

    vec!()
}

pub fn unequip_item_action(entity: &mut Entity, _map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let equipped_item;
    match entity.intent.data.clone() {
        IntentData::EquippedItem(item) => {
            equipped_item = item;
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }

    match entity.body.unequip(equipped_item) {
        Some(item) => {
            log.log(format!("{} unequipped {}", entity.name, item.name));
            entity.body.inventory.push(item);
        },
        None => ()
    }

    entity.body.update_armor();

    vec!()
}

pub fn single_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let mut result = vec!();

    let target_map_index;
    let target_pos;
    let item_slot;
    let bodypart;
    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => {
            item_slot = slot;
            target_pos = target;
            target_map_index = map.pos_idx(target);
            bodypart = 0;
        },
        IntentData::TargetBodypartWithEquipment{slot, target, bodypart_index} => {
            item_slot = slot;
            target_pos = target;
            target_map_index = map.pos_idx(target);
            bodypart = bodypart_index;
        },
        _ => {
            debug_assert!(false);
            return result;
        }
    }

    let shot_damage;
    match entity.get_equipped_item(item_slot) {
        Some(item) => {
            match item.kind {
                ItemKind::Firearm {ammo, max_ammo, damage, range} => {
                    if ammo < 1 {
                        log.log(format!("{} pulled the trigger. 'Click'.", entity.name));
                        return result;
                    }
                    item.kind = ItemKind::Firearm {ammo: ammo - 1, max_ammo, damage, range};
                    shot_damage = damage;
                },
                _ => {
                    debug_assert!(false);
                    return result;
                }
            }
        },
        None => {
            debug_assert!(false);
            return result;
        }
    }

    match &map.pawns[target_map_index] {
        Some(pawn) => {
            result.push(Effect::Damage {
                entity_id: pawn.entity_id,
                bodypart_index: bodypart,
                raw_damage: shot_damage
            });
            log.log(format!("{} fired at {}", entity.name, pawn.name));
        },
        _ => ()
    }

    result.push(Effect::Animation(shot_animation(entity.position, target_pos, 1)));

    result
}

pub fn burst_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let mut result = vec!();

    let target_map_index;
    let target_pos;
    let item_slot;
    let bodypart;
    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => {
            item_slot = slot;
            target_pos = target;
            target_map_index = map.pos_idx(target);
            bodypart = 0;
        },
        IntentData::TargetBodypartWithEquipment{slot, target, bodypart_index} => {
            item_slot = slot;
            target_pos = target;
            target_map_index = map.pos_idx(target);
            bodypart = bodypart_index;
        },
        _ => {
            debug_assert!(false);
            return result;
        }
    }

    let shot_damage;
    let mut shots = 5;
    match entity.get_equipped_item(item_slot) {
        Some(item) => {
            match item.kind {
                ItemKind::Firearm {ammo, max_ammo, damage, range} => {
                    shot_damage = damage;
                    if ammo == 0 {
                        log.log(format!("{} pulled the trigger. 'Clickclickclickclickclick'.", entity.name));
                        return result;
                    }
                    else if ammo < 5 {
                        item.kind = ItemKind::Firearm {ammo: 0, max_ammo, damage, range};
                        shots = ammo;
                    } else {
                        item.kind = ItemKind::Firearm {ammo: ammo - 5, max_ammo, damage, range};
                    }
                },
                _ => {
                    debug_assert!(false);
                    return result;
                }
            }
        },
        None => {
            debug_assert!(false);
            return result;
        }
    }

    match &map.pawns[target_map_index] {
        Some(pawn) => {
            for _ in 0..shots {
                result.push(Effect::Damage {
                    entity_id: pawn.entity_id,
                    bodypart_index: bodypart,
                    raw_damage: shot_damage
                });
            }
            log.log(format!("{} fired {} shots at {}", entity.name, shots, pawn.name));
        },
        _ => ()
    }

    result.push(Effect::Animation(shot_animation(entity.position, target_pos, 5)));

    result
}

pub fn rocket_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let mut result = vec!();

    let target_pos;
    let target_map_index;
    let item_slot;

    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => {
            item_slot = slot;
            target_map_index = map.pos_idx(target);
            target_pos = target;
        },
        _ => {
            debug_assert!(false);
            return result;
        }
    }

    let shot_damage;
    match entity.get_equipped_item(item_slot) {
        Some(item) => {
            match item.kind {
                ItemKind::Firearm {ammo, max_ammo, damage, range} => {
                    if ammo < 1 {
                        log.log(format!("{} pulled the trigger. 'Click'.", entity.name));
                        return result;
                    }
                    item.kind = ItemKind::Firearm {ammo: ammo - 1, max_ammo, damage, range};
                    shot_damage = damage;
                },
                _ => {
                    debug_assert!(false);
                    return result;
                }
            }
        },
        None => {
            debug_assert!(false);
            return result;
        }
    }

    match &map.pawns[target_map_index] {
        Some(pawn) => {
            for part_index in 0..pawn.body.parts.len() {
                result.push(Effect::Damage {
                    entity_id: pawn.entity_id,
                    bodypart_index: part_index,
                    raw_damage: shot_damage
                });
            }
        }
        _ => ()
    }

    result.push(Effect::DestroyWall(target_pos));
    result.push(Effect::Animation(explosion_animation(target_pos)));

    result
}

pub fn fan_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let mut result = vec!();

    let item_slot;
    let target_pos;
    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => {
            item_slot = slot;
            target_pos = target;
        },
        _ => {
            debug_assert!(false);
            return result;
        }
    }

    let shot_damage;
    let range;
    match entity.get_equipped_item(item_slot) {
        Some(item) => {
            match item.kind {
                ItemKind::Firearm {ammo, max_ammo, damage, range: r} => {
                    if ammo < 1 {
                        log.log(format!("{} pulled the trigger. 'Click'.", entity.name));
                        return result;
                    }
                    item.kind = ItemKind::Firearm {ammo: ammo - 1, max_ammo, damage, range: r};
                    shot_damage = damage;
                    range = r;
                },
                _ => {
                    debug_assert!(false);
                    return result;
                }
            }
        },
        None => {
            debug_assert!(false);
            return result;
        }
    }

    let src = entity.position;
    let dx = (target_pos.x - src.x) as f32;
    let dy = (target_pos.y - src.y) as f32;
    let dir_len = (dx * dx + dy * dy).sqrt();
    if dir_len == 0.0 {
        return result;
    }
    let dir_x = dx / dir_len;
    let dir_y = dy / dir_len;

    // cos(22.5 degrees) — half of a 45-degree arc
    const HALF_ARC_COS: f32 = 0.9239;

    let range_i = range as i32;
    let mut arc_positions = vec!();
    for ty in (src.y - range_i)..=(src.y + range_i) {
        for tx in (src.x - range_i)..=(src.x + range_i) {
            if tx < 0 || ty < 0 || tx >= map.width as i32 || ty >= map.height as i32 {
                continue;
            }
            let tdx = (tx - src.x) as f32;
            let tdy = (ty - src.y) as f32;
            let tile_dist = (tdx * tdx + tdy * tdy).sqrt();
            if tile_dist < 0.5 || tile_dist > range as f32 {
                continue;
            }
            let dot = (dir_x * tdx + dir_y * tdy) / tile_dist;
            if dot < HALF_ARC_COS {
                continue;
            }

            let tile_pos = rltk::Point::new(tx, ty);
            let tile_idx = map.pos_idx(tile_pos);
            if let Some(pawn) = &map.pawns[tile_idx] {
                for part_index in 0..pawn.body.parts.len() {
                    result.push(Effect::Damage {
                        entity_id: pawn.entity_id,
                        bodypart_index: part_index,
                        raw_damage: shot_damage,
                    });
                }
                log.log(format!("{} hit {} with fan fire", entity.name, pawn.name));
            }
            arc_positions.push(tile_pos);
        }
    }

    if !arc_positions.is_empty() {
        result.push(Effect::Animation(fan_fire_animation(arc_positions)));
    }

    result
}

pub fn open_door_action(entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::Target(pos) => {
            return vec!(Effect::OpenDoor(pos));
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }
}

pub fn move_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    if !entity.has_ability(Ability::HumanMove) && !entity.has_ability(Ability::VehicleMove) {
        log.log(format!("{} tried to move, but couldn't", entity.name));
        return vec!();
    }

    match entity.intent.data {
        IntentData::Target(pos) => {
            if entity.check_fit(pos, map) {
                entity.set_position(pos, map);
            }
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }

    vec!()
}

pub fn turn_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    if entity.has_ability(Ability::HumanMove) {
        return fast_turn_action(entity, map);
    }
    else if entity.has_ability(Ability::VehicleMove) {
        return slow_turn_action(entity, map, log);
    }
    else {
        log.log(format!("{} tried to turn, but couldn't", entity.name));
        return vec!();
    }
}

fn fast_turn_action(entity: &mut Entity, map: &mut Map) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::Direction(direction) => {
            entity.body.facing = direction;
            entity.set_position(entity.position, map);
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }
    return vec!();
}

fn slow_turn_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::Direction(direction) => {
            if entity.body.facing.clockwise() == direction
                || entity.body.facing.counter_clockwise() == direction {
                entity.body.facing = direction;
                entity.set_position(entity.position, map);
            } else {
                log.log(format!("{} tried to turn, but couldn't", entity.name));
                return vec!();
            }
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }
    return vec!();
}

pub fn embark_action(entity: &mut Entity, map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::Target(pos) => {
            let index = map.pos_idx(pos);
            match &map.pawns[index] {
                Some(pawn) => {
                    let vehicle_id = pawn.entity_id;
                    return vec!(Effect::Embark{pilot_id: entity.id, vehicle_id: vehicle_id});
                },
                None => return vec!()
            }
        },
        _ => return vec!()
    }
}

pub fn disembark_action(entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    match entity.driving {
        DrivingState::DrivenBy(pilot) => {
            return vec!(Effect::Disembark{pilot_id: pilot, vehicle_id: entity.id});        
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }
}

pub fn melee_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let mut result = vec!();

    match entity.intent.data {
        IntentData::Target(pos) => {
            let index = map.xy_idx(pos.x, pos.y);
            let id = map.pawns[index].as_ref().unwrap().entity_id;
            log.log(format!("{} struck {}", entity.name, map.pawns[index].as_ref().unwrap().name));
            result.push(Effect::Damage {
                entity_id: id,
                bodypart_index: 1,
                raw_damage: Damage::new(1, 0, 0, 0)
            });
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }

    result
}

pub fn aim_action(entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => {
            {
                let item = entity.get_equipped_item(slot).unwrap().clone();
                console::log(format!("Item id: {}", item.id));
            }
            return vec!(Effect::ApplyStatus {
                target_id: entity.id,
                status: StatusEffect::AimingAtGround(target, entity.get_equipped_item(slot).unwrap().clone())
            });
        },
        _ => {
            debug_assert!(false);
            return vec!();
        }
    }
}
