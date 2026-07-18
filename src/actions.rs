use rltk::Point;
use crate::Entity;
use crate::Map;
use crate::components::*;
use crate::intent::*;
use crate::animation::*;
use crate::ability::*;
use crate::DrivingState;

pub type Action = fn(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect>;

// ---------------------------------------------------------------------------
// Read-only ammo query — returns (damage, range, shots_that_would_fire).
// Produces no side-effects; consume is expressed as Effect::ConsumeAmmo.
// ---------------------------------------------------------------------------
fn read_ammo(entity: &Entity, slot: SlotType, requested: u32) -> Option<(Damage, u32, u32)> {
    let item = entity.get_equipped_item_ref(slot)?;
    match &item.kind {
        ItemKind::Firearm { ammo, damage, range, .. } => {
            if *ammo == 0 { return None; }
            Some((*damage, *range, requested.min(*ammo)))
        },
        _ => None,
    }
}

fn extract_fire_intent(entity: &Entity) -> Option<(SlotType, Point, usize)> {
    match entity.intent.data {
        IntentData::TargetWithEquipment { slot, target }                    => Some((slot, target, 0)),
        IntentData::TargetBodypartWithEquipment { slot, target, bodypart_index } => Some((slot, target, bodypart_index)),
        _ => None,
    }
}

fn away_direction(from: Point, away_from: Point) -> Direction {
    let dx = (from.x - away_from.x).signum();
    let dy = (from.y - away_from.y).signum();
    match (dx, dy) {
        ( 0, -1) => Direction::Up,
        ( 1, -1) => Direction::UpRight,
        ( 1,  0) => Direction::Right,
        ( 1,  1) => Direction::DownRight,
        ( 0,  1) => Direction::Down,
        (-1,  1) => Direction::DownLeft,
        (-1,  0) => Direction::Left,
        (-1, -1) => Direction::UpLeft,
        _        => Direction::Up,
    }
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

pub fn move_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    if !entity.has_ability(Ability::HumanMove) && !entity.has_ability(Ability::VehicleMove) {
        return vec![Effect::Log(format!("{} tried to move, but couldn't", entity.name))];
    }
    let IntentData::Target(pos) = entity.intent.data else {
        unreachable!("move_action called with non-target intent")
    };
    if !entity.check_fit(pos, map) {
        return vec![];
    }
    let mut effects = vec![Effect::Move { entity_id: entity.index, pos }];
    if entity.has_ability(Ability::VehicleMove) {
        effects.push(Effect::Sound(SoundEvent { kind: SoundKind::Engine, pos, volume: 15 }));
    } else if !entity.has_ability(Ability::Stealth) {
        effects.push(Effect::Sound(SoundEvent { kind: SoundKind::Footstep, pos, volume: 5 }));
    }
    effects
}

pub fn turn_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::Direction(direction) = entity.intent.data else {
        unreachable!("turn_action called with non-direction intent")
    };
    if entity.has_ability(Ability::HumanMove) {
        return vec![Effect::SetFacing { entity_id: entity.index, direction }];
    }
    if entity.has_ability(Ability::VehicleMove) {
        let ok = entity.body.facing.clockwise() == direction
               || entity.body.facing.counter_clockwise() == direction;
        if ok {
            return vec![
                Effect::SetFacing { entity_id: entity.index, direction },
                Effect::Sound(SoundEvent { kind: SoundKind::Engine, pos: entity.position, volume: 15 }),
            ];
        } else {
            return vec![Effect::Log(format!("{} tried to turn, but couldn't", entity.name))];
        }
    }
    vec![Effect::Log(format!("{} tried to turn, but couldn't", entity.name))]
}

pub fn juke_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    const ENERGY_COST: u32 = 25;
    if entity.body.energy < ENERGY_COST {
        return vec![Effect::Log(format!("{} is too exhausted to Juke", entity.name))];
    }
    let IntentData::Target(pos) = entity.intent.data else { return vec![]; };
    if !entity.check_fit(pos, map) { return vec![]; }
    let mut effects = vec![
        Effect::SpendEnergy { entity_id: entity.index, amount: ENERGY_COST },
        Effect::Move { entity_id: entity.index, pos },
    ];
    if !entity.has_ability(Ability::Stealth) {
        effects.push(Effect::Sound(SoundEvent { kind: SoundKind::Footstep, pos, volume: 5 }));
    }
    effects
}

/// Rocket boots: instantly teleport onto the chosen tile (targeting already guarantees it
/// is visible and within range), landing with a loud engine roar. Stealth does not muffle
/// it, and it spends one charge. Nothing happens (and no charge is spent) if the tile is blocked.
pub fn rocket_boots_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::TargetWithEquipment { slot, target } = entity.intent.data else { return vec![]; };
    if !entity.check_fit(target, map) {
        return vec![Effect::Log(format!("{} can't land there.", entity.name))];
    }
    let Some(item_id) = entity.get_equipped_item_ref(slot).map(|i| i.id) else { return vec![]; };
    vec![
        Effect::Move { entity_id: entity.index, pos: target },
        Effect::Sound(SoundEvent { kind: SoundKind::Engine, pos: target, volume: 30 }),
        Effect::ConsumeCharge { entity_id: entity.index, item_id },
    ]
}

/// Jetpack Rocket Jump: teleport to the chosen tile — validated as a revealed Ground/Road
/// tile by targeting — falling back to the nearest free tile if it is occupied. Booms loudly
/// at both origin and destination, and spends one charge.
pub fn rocket_jump_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::TargetWithEquipment { slot, target } = entity.intent.data else { return vec![]; };
    let Some(item_id) = entity.get_equipped_item_ref(slot).map(|i| i.id) else { return vec![]; };
    let dest = match map.nearest_free_pawn_position(target) {
        Ok(pos) => pos,
        Err(_) => return vec![Effect::Log(format!("{} found nowhere to land.", entity.name))],
    };
    let origin = entity.position;
    vec![
        Effect::Sound(SoundEvent { kind: SoundKind::Explosion, pos: origin, volume: 30 }),
        Effect::Move { entity_id: entity.index, pos: dest },
        Effect::Sound(SoundEvent { kind: SoundKind::Explosion, pos: dest, volume: 30 }),
        Effect::ConsumeCharge { entity_id: entity.index, item_id },
    ]
}

/// Tactical helmet: open a long-range recon vision cone aimed at the chosen tile.
/// The cone persists (via the Scanning status) until the player moves or turns.
pub fn recon_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::TargetWithEquipment { target, .. } = entity.intent.data else { return vec![]; };
    vec![
        Effect::Log(format!("{} scans the area.", entity.name)),
        Effect::ApplyScan { entity_id: entity.index, target },
    ]
}

pub fn open_door_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::Target(pos) = entity.intent.data else {
        unreachable!("open_door_action called with non-target intent")
    };
    vec![Effect::OpenDoor { pos, actor_id: entity.index }]
}

pub fn embark_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::Target(pos) = entity.intent.data else { return vec![]; };
    match &map.pawns[map.pos_idx(pos)] {
        Some(pawn) => vec![Effect::Embark { pilot_id: entity.index, vehicle_id: pawn.entity_id }],
        None       => vec![],
    }
}

pub fn disembark_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    match entity.driving {
        DrivingState::DrivenBy(pilot) => vec![Effect::Disembark { pilot_id: pilot, vehicle_id: entity.index }],
        _ => unreachable!("disembark_action called on entity that is not being driven"),
    }
}

// ---------------------------------------------------------------------------
// Melee
// ---------------------------------------------------------------------------

pub fn melee_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    let IntentData::Target(pos) = entity.intent.data else {
        unreachable!("melee_action called with non-target intent")
    };
    let index = map.xy_idx(pos.x, pos.y);
    let pawn = map.pawns[index].as_ref().unwrap();
    let target_id = pawn.entity_id;
    let (bodypart_index, raw_damage) = entity.melee_strike(&entities[target_id]);
    vec![
        Effect::Log(format!("{} struck {}", entity.name, entities[target_id].name)),
        Effect::Damage { entity_id: target_id, bodypart_index, raw_damage },
    ]
}

// ---------------------------------------------------------------------------
// Ranged fire
// ---------------------------------------------------------------------------

pub fn single_fire_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    let (slot, target_pos, bodypart) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("single_fire_action called with non-fire intent"),
    };
    let (damage, _range, fired) = match read_ammo(entity, slot, 1) {
        Some(v) => v,
        None => return vec![Effect::Log(format!("{} pulled the trigger. 'Click'.", entity.name))],
    };
    let mut effects = vec![
        Effect::ConsumeAmmo { entity_id: entity.index, slot, shots: fired },
        Effect::Sound(SoundEvent { kind: SoundKind::Gunshot, pos: entity.position, volume: 20 }),
        Effect::Animation(shot_animation(entity.position, target_pos, 1)),
    ];
    if let Some(pawn) = &map.pawns[map.pos_idx(target_pos)] {
        effects.push(Effect::Log(format!("{} fired at {}", entity.name, entities[pawn.entity_id].name)));
        effects.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: bodypart, raw_damage: damage });
    }
    effects
}

pub fn burst_fire_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    let (slot, target_pos, bodypart) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("burst_fire_action called with non-fire intent"),
    };
    let (damage, _range, shots) = match read_ammo(entity, slot, 5) {
        Some(v) => v,
        None => return vec![Effect::Log(format!("{} pulled the trigger. 'Clickclickclickclickclick'.", entity.name))],
    };
    let mut effects = vec![
        Effect::ConsumeAmmo { entity_id: entity.index, slot, shots },
        Effect::Sound(SoundEvent { kind: SoundKind::Burst, pos: entity.position, volume: 25 }),
        Effect::Animation(shot_animation(entity.position, target_pos, shots as i32)),
    ];
    if let Some(pawn) = &map.pawns[map.pos_idx(target_pos)] {
        effects.push(Effect::Log(format!("{} fired {} shots at {}", entity.name, shots, entities[pawn.entity_id].name)));
        for _ in 0..shots {
            effects.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: bodypart, raw_damage: damage });
        }
    }
    effects
}

pub fn rocket_fire_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    let (slot, target_pos, _) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("rocket_fire_action called with non-fire intent"),
    };
    let (damage, _range, fired) = match read_ammo(entity, slot, 1) {
        Some(v) => v,
        None => return vec![Effect::Log(format!("{} pulled the trigger. 'Click'.", entity.name))],
    };
    let mut effects = vec![
        Effect::ConsumeAmmo { entity_id: entity.index, slot, shots: fired },
        Effect::Sound(SoundEvent { kind: SoundKind::Explosion, pos: entity.position, volume: 30 }),
        Effect::DestroyWall(target_pos),
        Effect::Animation(explosion_animation(target_pos, 1)),
    ];
    if let Some(pawn) = &map.pawns[map.pos_idx(target_pos)] {
        for part_index in 0..entities[pawn.entity_id].body.parts.len() {
            effects.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: part_index, raw_damage: damage });
        }
    }
    effects
}

pub fn fan_fire_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    let (slot, target_pos, _) = match extract_fire_intent(entity) {
        Some(v) => v,
        None => unreachable!("fan_fire_action called with non-fire intent"),
    };
    let (damage, range, fired) = match read_ammo(entity, slot, 1) {
        Some(v) => v,
        None => return vec![Effect::Log(format!("{} pulled the trigger. 'Click'.", entity.name))],
    };
    let src = entity.position;
    let dx = (target_pos.x - src.x) as f32;
    let dy = (target_pos.y - src.y) as f32;
    let dir_len = (dx * dx + dy * dy).sqrt();
    if dir_len == 0.0 { return vec![]; }
    let dir_x = dx / dir_len;
    let dir_y = dy / dir_len;
    const HALF_ARC_COS: f32 = 0.9239;

    let mut effects = vec![
        Effect::ConsumeAmmo { entity_id: entity.index, slot, shots: fired },
        Effect::Sound(SoundEvent { kind: SoundKind::Gunshot, pos: src, volume: 15 }),
    ];
    let range_i = range as i32;
    let mut arc_positions = vec![];
    for ty in (src.y - range_i)..=(src.y + range_i) {
        for tx in (src.x - range_i)..=(src.x + range_i) {
            if tx < 0 || ty < 0 || tx >= map.width as i32 || ty >= map.height as i32 { continue; }
            let tdx = (tx - src.x) as f32;
            let tdy = (ty - src.y) as f32;
            let tile_dist = (tdx * tdx + tdy * tdy).sqrt();
            if tile_dist < 0.5 || tile_dist > range as f32 { continue; }
            if (dir_x * tdx + dir_y * tdy) / tile_dist < HALF_ARC_COS { continue; }
            let tile_pos = Point::new(tx, ty);
            let ray = rltk::line2d(rltk::LineAlg::Bresenham, src, tile_pos);
            let n = ray.len();
            if ray[1..n.saturating_sub(1)].iter().any(|p| map.blocked(p.x, p.y)) { continue; }
            if let Some(pawn) = &map.pawns[map.pos_idx(tile_pos)] {
                effects.push(Effect::Log(format!("{} hit {} with fan fire", entity.name, entities[pawn.entity_id].name)));
                for part in 0..entities[pawn.entity_id].body.parts.len() {
                    effects.push(Effect::Damage { entity_id: pawn.entity_id, bodypart_index: part, raw_damage: damage });
                }
            }
            arc_positions.push(tile_pos);
        }
    }
    if !arc_positions.is_empty() {
        effects.push(Effect::Animation(fan_fire_animation(arc_positions)));
    }
    effects
}

// ---------------------------------------------------------------------------
// Aim
// ---------------------------------------------------------------------------

pub fn aim_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::TargetWithEquipment { slot, target } = entity.intent.data else {
        unreachable!("aim_action called with non-equipment-target intent")
    };
    let item = entity.get_equipped_item_ref(slot).unwrap().clone();
    let status = match &map.pawns[map.pos_idx(target)] {
        Some(pawn) => StatusEffect::AimingAtEntity(pawn.entity_id, item),
        None       => StatusEffect::AimingAtGround(target, item),
    };
    vec![Effect::ApplyStatus { target_id: entity.index, status }]
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

pub fn get_item_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let index = map.xy_idx(entity.position.x, entity.position.y);
    if map.items[index].is_none() { return vec![]; }
    if entity.body.inventory.len() >= crate::components::INVENTORY_MAX {
        return vec![Effect::Log(format!("{} can't carry any more items.", entity.name))];
    }
    vec![Effect::PickUpItem { entity_id: entity.index }]
}

pub fn drop_item_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::InventoryItem(ref item) = entity.intent.data else {
        unreachable!("drop_item_action called with non-inventory intent")
    };
    vec![
        Effect::Log(format!("{} dropped {}", entity.name, item.name)),
        Effect::DropItem { entity_id: entity.index, item_id: item.id },
    ]
}

pub fn equip_item_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::InventoryItem(ref item) = entity.intent.data else {
        unreachable!("equip_item_action called with non-inventory intent")
    };
    vec![Effect::EquipItem { entity_id: entity.index, item_id: item.id }]
}

pub fn unequip_item_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::EquippedItem(slot) = entity.intent.data else {
        unreachable!("unequip_item_action called with non-equipped-item intent")
    };
    if let Some(item) = entity.get_equipped_item_ref(slot) {
        vec![Effect::UnequipItem { entity_id: entity.index, item_id: item.id }]
    } else {
        vec![]
    }
}

pub fn prime_grenade_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::InventoryItem(ref item) = entity.intent.data else {
        unreachable!("prime_grenade_action called with non-inventory intent")
    };
    vec![
        Effect::Log(format!("{} primed the {}", entity.name, item.name)),
        Effect::PrimeItem { entity_id: entity.index, item_id: item.id },
    ]
}

pub fn throw_grenade_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::TargetWithInventory { ref item, target } = entity.intent.data else {
        unreachable!("throw_grenade_action called with non-inventory-target intent")
    };
    vec![
        Effect::Log(format!("{} threw a {}", entity.name, item.name)),
        Effect::ThrowItem { entity_id: entity.index, item_id: item.id, target_pos: target },
    ]
}

/// Reload initiated from a firearm — equipped (EquippedItem) or carried (InventoryItem).
/// Resolves the firearm's id and hands off to the ReloadWeapon effect.
pub fn reload_weapon_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let weapon_id = match &entity.intent.data {
        IntentData::InventoryItem(item) => item.id,
        IntentData::EquippedItem(slot)  => match entity.get_equipped_item_ref(*slot) {
            Some(item) => item.id,
            None => return vec![],
        },
        _ => unreachable!("reload_weapon_action called with non-item intent"),
    };
    vec![Effect::ReloadWeapon { entity_id: entity.index, weapon_id }]
}

/// Reload initiated from an ammo box — finds a matching firearm to fill.
pub fn reload_from_ammo_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::InventoryItem(ref ammo) = entity.intent.data else {
        unreachable!("reload_from_ammo_action called with non-inventory intent")
    };
    let ItemKind::Ammo { kind, .. } = ammo.kind else { return vec![] };
    match crate::item::find_reloadable_weapon_id(entity, kind) {
        Some(weapon_id) => vec![Effect::ReloadWeapon { entity_id: entity.index, weapon_id }],
        None => vec![],
    }
}

/// Use a healing consumable. A per-part item (SelfBodypart targeting) arrives as a
/// bodypart intent; a whole-body item (elixir) arrives as a plain inventory intent.
pub fn use_healing_item_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let (item, bodypart_index) = match &entity.intent.data {
        IntentData::TargetBodypartWithInventory { item, bodypart_index, .. } => (item, Some(*bodypart_index)),
        IntentData::InventoryItem(item) => (item, None),
        _ => unreachable!("use_healing_item_action called with non-healing intent"),
    };
    let ItemKind::Healing { turns } = item.kind else { return vec![] };
    let scope = match bodypart_index {
        Some(i) => entity.body.parts.get(i).map(|p| p.name.clone()).unwrap_or_default(),
        None => "whole body".to_string(),
    };
    vec![
        Effect::Log(format!("{} used {} ({})", entity.name, item.name, scope)),
        Effect::ApplyRegeneration { entity_id: entity.index, bodypart_index, turns },
        Effect::ConsumeItem { entity_id: entity.index, item_id: item.id },
    ]
}

/// Use a stimpack to instantly restore energy.
pub fn use_stimpack_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    let IntentData::InventoryItem(ref item) = entity.intent.data else {
        unreachable!("use_stimpack_action called with non-inventory intent")
    };
    let ItemKind::Stimpack { energy } = item.kind else { return vec![] };
    vec![
        Effect::Log(format!("{} used {}", entity.name, item.name)),
        Effect::RestoreEnergy { entity_id: entity.index, amount: energy },
        Effect::ConsumeItem { entity_id: entity.index, item_id: item.id },
    ]
}

// ---------------------------------------------------------------------------
// Abilities
// ---------------------------------------------------------------------------

pub fn shout_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    vec![Effect::Sound(SoundEvent { kind: SoundKind::Shout, pos: entity.position, volume: 15 })]
}

pub fn iron_body_action(entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    const ENERGY_COST: u32 = 50;
    if entity.body.energy < ENERGY_COST {
        return vec![Effect::Log(format!("{} is too exhausted to use Iron Body", entity.name))];
    }
    vec![
        Effect::SpendEnergy { entity_id: entity.index, amount: ENERGY_COST },
        Effect::ApplyStatus { target_id: entity.index, status: StatusEffect::IronBody(3) },
    ]
}

pub fn distract_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    const ENERGY_COST: u32 = 10;
    if entity.body.energy < ENERGY_COST {
        return vec![Effect::Log(format!("{} is too exhausted to Distract", entity.name))];
    }
    let IntentData::Target(target_pos) = entity.intent.data else { return vec![]; };
    let target_id = match &map.pawns[map.pos_idx(target_pos)] {
        Some(pawn) => pawn.entity_id,
        None => return vec![],
    };
    if !entities[target_id].can_see(entity.position) {
        return vec![Effect::Log(format!("{} cannot be distracted — they can't see you", entities[target_id].name))];
    }
    vec![
        Effect::SpendEnergy { entity_id: entity.index, amount: ENERGY_COST },
        Effect::Distract { entity_id: target_id },
    ]
}

pub fn twist_action(entity: &Entity, map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    const ENERGY_COST: u32 = 10;
    if entity.body.energy < ENERGY_COST {
        return vec![Effect::Log(format!("{} is too exhausted to Twist", entity.name))];
    }
    let IntentData::Target(target_pos) = entity.intent.data else { return vec![]; };
    let target_id = match &map.pawns[map.pos_idx(target_pos)] {
        Some(pawn) => pawn.entity_id,
        None => return vec![],
    };
    let direction = away_direction(target_pos, entity.position);
    vec![
        Effect::SpendEnergy { entity_id: entity.index, amount: ENERGY_COST },
        Effect::Twist { entity_id: target_id, direction },
    ]
}

pub fn rush_action(entity: &Entity, map: &Map, entities: &[Entity]) -> Vec<Effect> {
    const ENERGY_COST: u32 = 25;
    if entity.body.energy < ENERGY_COST {
        return vec![Effect::Log(format!("{} is too exhausted to Rush", entity.name))];
    }
    let IntentData::Target(target_pos) = entity.intent.data else { return vec![]; };
    let target_id = match &map.pawns[map.pos_idx(target_pos)] {
        Some(pawn) => pawn.entity_id,
        None => return vec![],
    };
    let current_pos = entity.position;
    let deltas: [(i32, i32); 8] = [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)];
    let best_pos = deltas.iter()
        .map(|(dx, dy)| Point { x: target_pos.x + dx, y: target_pos.y + dy })
        .filter(|&p| entity.check_fit(p, map))
        .min_by_key(|p| { let dx = p.x - current_pos.x; let dy = p.y - current_pos.y; dx*dx + dy*dy });
    let (bodypart_index, raw_damage) = entity.melee_strike(&entities[target_id]);
    let mut effects = vec![Effect::SpendEnergy { entity_id: entity.index, amount: ENERGY_COST }];
    if let Some(pos) = best_pos {
        effects.push(Effect::Move { entity_id: entity.index, pos });
    }
    effects.push(Effect::Damage { entity_id: target_id, bodypart_index, raw_damage });
    effects
}
