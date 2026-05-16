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
        _ => unreachable!("throw_grenade_action called with non-inventory-target intent"),
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
        _ => unreachable!("drop_item_action called with non-inventory intent"),
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
        _ => unreachable!("equip_item_action called with non-inventory intent"),
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
        None => unreachable!("item from intent data was not found in entity inventory"),
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
        _ => unreachable!("unequip_item_action called with non-equipped-item intent"),
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

/// Extracts (slot, target_pos, bodypart_index) from fire-related intent data.
/// Returns None if the intent does not match a fire targeting variant.
fn extract_fire_intent(entity: &Entity) -> Option<(SlotType, rltk::Point, usize)> {
    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => Some((slot, target, 0)),
        IntentData::TargetBodypartWithEquipment{slot, target, bodypart_index} => Some((slot, target, bodypart_index)),
        _ => None,
    }
}

/// Consumes up to `requested` ammo from the firearm in `slot`.
/// Returns (damage, range, shots_fired) on success, where shots_fired may be less
/// than requested if the weapon is running low. Returns None if ammo is empty.
fn consume_ammo(entity: &mut Entity, slot: SlotType, requested: u32) -> Option<(Damage, u32, u32)> {
    let item = entity.get_equipped_item(slot)?;
    match item.kind {
        ItemKind::Firearm { ammo, max_ammo, damage, range } => {
            if ammo == 0 {
                return None;
            }
            let fired = requested.min(ammo);
            item.kind = ItemKind::Firearm { ammo: ammo - fired, max_ammo, damage, range };
            Some((damage, range, fired))
        },
        _ => None,
    }
}

pub fn single_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let (slot, target_pos, bodypart) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("single_fire_action called with non-fire intent"),
    };
    let (damage, _range, _fired) = match consume_ammo(entity, slot, 1) {
        Some(v) => v,
        None => {
            log.log(format!("{} pulled the trigger. 'Click'.", entity.name));
            return vec!();
        }
    };

    let mut result = vec!();
    if let Some(pawn) = &map.pawns[map.pos_idx(target_pos)] {
        result.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: bodypart, raw_damage: damage });
        log.log(format!("{} fired at {}", entity.name, pawn.name));
    }
    result.push(Effect::Animation(shot_animation(entity.position, target_pos, 1)));
    entity.clear_aiming();
    result
}

pub fn burst_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let (slot, target_pos, bodypart) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("burst_fire_action called with non-fire intent"),
    };
    let (damage, _range, shots) = match consume_ammo(entity, slot, 5) {
        Some(v) => v,
        None => {
            log.log(format!("{} pulled the trigger. 'Clickclickclickclickclick'.", entity.name));
            return vec!();
        }
    };

    let mut result = vec!();
    if let Some(pawn) = &map.pawns[map.pos_idx(target_pos)] {
        for _ in 0..shots {
            result.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: bodypart, raw_damage: damage });
        }
        log.log(format!("{} fired {} shots at {}", entity.name, shots, pawn.name));
    }
    result.push(Effect::Animation(shot_animation(entity.position, target_pos, shots as i32)));
    entity.clear_aiming();
    result
}

pub fn rocket_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let (slot, target_pos, _bodypart) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("rocket_fire_action called with non-fire intent"),
    };
    let (damage, _range, _fired) = match consume_ammo(entity, slot, 1) {
        Some(v) => v,
        None => {
            log.log(format!("{} pulled the trigger. 'Click'.", entity.name));
            return vec!();
        }
    };

    let mut result = vec!();
    if let Some(pawn) = &map.pawns[map.pos_idx(target_pos)] {
        for part_index in 0..pawn.body.parts.len() {
            result.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: part_index, raw_damage: damage });
        }
    }
    result.push(Effect::DestroyWall(target_pos));
    result.push(Effect::Animation(explosion_animation(target_pos)));
    entity.clear_aiming();
    result
}

pub fn fan_fire_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    let (slot, target_pos, _bodypart) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("fan_fire_action called with non-fire intent"),
    };
    let (damage, range, _fired) = match consume_ammo(entity, slot, 1) {
        Some(v) => v,
        None => {
            log.log(format!("{} pulled the trigger. 'Click'.", entity.name));
            return vec!();
        }
    };

    let src = entity.position;
    let dx = (target_pos.x - src.x) as f32;
    let dy = (target_pos.y - src.y) as f32;
    let dir_len = (dx * dx + dy * dy).sqrt();
    if dir_len == 0.0 {
        return vec!();
    }
    let dir_x = dx / dir_len;
    let dir_y = dy / dir_len;

    // cos(22.5 degrees) — half of a 45-degree arc
    const HALF_ARC_COS: f32 = 0.9239;

    let mut result = vec!();
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
                        raw_damage: damage,
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

    entity.clear_aiming();
    result
}

pub fn open_door_action(entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::Target(pos) => vec!(Effect::OpenDoor(pos)),
        _ => unreachable!("open_door_action called with non-target intent"),
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
                entity.clear_aiming();
            }
        },
        _ => unreachable!("move_action called with non-target intent"),
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
        _ => unreachable!("fast_turn_action called with non-direction intent"),
    }
    vec!()
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
        _ => unreachable!("slow_turn_action called with non-direction intent"),
    }
    vec!()
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
        DrivingState::DrivenBy(pilot) => vec!(Effect::Disembark{pilot_id: pilot, vehicle_id: entity.id}),
        _ => unreachable!("disembark_action called on entity that is not being driven"),
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
        _ => unreachable!("melee_action called with non-target intent"),
    }

    result
}

pub fn juke_action(entity: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
    const ENERGY_COST: u32 = 25;
    if entity.body.energy < ENERGY_COST {
        log.log(format!("{} is too exhausted to Juke", entity.name));
        return vec!();
    }
    if let IntentData::Target(pos) = entity.intent.data {
        if entity.check_fit(pos, map) {
            entity.body.energy -= ENERGY_COST;
            entity.set_position(pos, map);
            entity.clear_aiming();
        }
    }
    vec!()
}

pub fn aim_action(entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    match entity.intent.data {
        IntentData::TargetWithEquipment{slot, target} => {
            vec!(Effect::ApplyStatus {
                target_id: entity.id,
                status: StatusEffect::AimingAtGround(target, entity.get_equipped_item(slot).unwrap().clone())
            })
        },
        _ => unreachable!("aim_action called with non-equipment-target intent"),
    }
}
