use rltk::Point;
use crate::components::*;
use crate::ai::*;
use crate::intent::*;
use crate::sprite::*;
use crate::GameLog;
use crate::Viewshed;
use crate::Ability;
use crate::Map;
use crate::Item;
use crate::Body;
use crate::animation::*;

/// Concrete type containing all data of something that acts and moves.
pub struct Entity {
    pub id: usize,
    pub sprite: Sprite,
    pub size: u32,
    pub position: Point,
    pub name: String,
    pub intent: Intent,
    pub body: Body,
    pub viewshed: Viewshed,
    pub ai: AI
}

impl Entity {
    pub fn new_human(id: usize, pos: Point, facing: Direction, name: String) -> Self {
        Self {
            id: id,
            sprite: Sprite::Human,
            size: 1,
            position: pos,
            name: name,
            intent: idle_intent(),
            body: Body::human_body(facing),
            viewshed: Viewshed::new(),
            ai: AI::None
        }
    }

    pub fn new_patrolling_goon(id: usize, pos: Point, facing: Direction, name: String, waypoints: Vec<Point>) -> Self {
        Self {
            id: id,
            sprite: Sprite::Human,
            size: 1,
            position: pos,
            name: name,
            intent: idle_intent(),
            body: Body::human_body(facing),
            viewshed: Viewshed::new(),
            ai: AI::Patrolling(PatrollingAI::new(waypoints))
        }
    }

    pub fn new_tank(id: usize, pos: Point, facing: Direction, name: String) -> Self {
        Self {
            id: id,
            sprite: Sprite::Tank,
            size: 3,
            position: pos,
            name: name,
            intent: idle_intent(),
            body: Body::tank_body(facing),
            viewshed: Viewshed::new(),
            ai: AI::Rotator
        }
    }

    pub fn check_fit(&self, pos: Point, map: &mut Map) -> bool {
        for x in 0..self.size {
            for y in 0..self.size {
                let index = map.xy_idx(pos.x + x as i32, pos.y + y as i32);
                if map.pawns[index].is_some() || map.blocked_idx(index) {
                    return false;
                }
            }
        }

        return true;
    }

    pub fn create_pawns(&self, map: &mut Map) {
        for x in 0..self.size {
            for y in 0..self.size {
                let index = map.xy_idx(self.position.x + x as i32, self.position.y + y as i32);
                map.pawns[index] = Some(Pawn {
                    entity_id: self.id,
                    sprite: self.sprite.clone(),
                    sprite_index: x + y * self.size,
                    name: self.name.clone(),
                    intent: self.intent.clone(),
                    body: self.body.clone()
                });
            }
        }
    }

    pub fn clear_pawns(&self, map: &mut Map) {
        for x in 0..self.size {
            for y in 0..self.size {
                let index = map.xy_idx(self.position.x + x as i32, self.position.y + y as i32);
                map.pawns[index] = None;
            }
        }
    }

    pub fn set_position(&mut self, pos: Point, map: &mut Map) {
        self.clear_pawns(map);
        self.position = pos;
        self.create_pawns(map);
    }

    pub fn take_item(&mut self, item: Item) -> Option<Item> {
        if let Some(item_index) = self.body.inventory.iter().position(|value| *value == item) {
            Some(self.body.inventory.swap_remove(item_index))
        }
        else {
            None
        }
    }

    pub fn get_equipped_item(&mut self, slot: SlotType) -> Option<&mut Item> {
        if let Some(item_index) = self.body.item_slots.iter().position(|value| value.slot_type == slot) {
            self.body.item_slots[item_index].item.as_mut()
        }
        else {
            None
        }
    }

    pub fn declare_intent(&mut self, map: &Map) {
        match &mut self.ai {
            AI::Patrolling(ai) => {
                self.intent = ai.declare_intent(self.position, &self.body, map);
            },
            AI::Rotator => {
                self.intent = Intent {
                    phase: IntentPhase::Movement,
                    data: IntentData::Direction(self.body.facing.clockwise()),
                    action: Entity::resolve_turn
                };
            },
            AI::None => ()
        }
    }

    pub fn update_view(&mut self, map: &Map) {
        self.viewshed.update(self.position, self.body.facing, map);
    }

    pub fn resolve_throw_grenade(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        log.log(format!("{} threw a grenade", self.name));
        let mut result = vec!();
    
        let used_item;
        let target_map_index;
        let target_pos;
        match self.intent.data.clone() {
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
    
        self.take_item(used_item);
        
        match &map.pawns[target_map_index] {
            Some(pawn) => {
                for part_index in 0..pawn.body.parts.len() {
                    result.push(Effect::Damage{
                        entity_id: pawn.entity_id,
                        bodypart_index: part_index,
                        raw_damage: 5
                    });
                }
            }
            _ => ()
        }

        result.push(Effect::Animation(explosion_animation(target_pos)));
    
        result
    }
    
    pub fn resolve_drop_item(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let inventory_item;
        match self.intent.data.clone() {
            IntentData::InventoryItem(item) => {
                inventory_item = item;
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }

        log.log(format!("{} dropped {}", self.name, inventory_item.name));

        let target_pos = map.nearest_free_item_position(self.position).unwrap();
        let map_index = map.pos_idx(target_pos);

        map.items[map_index] = self.take_item(inventory_item);

        vec!()
    }

    pub fn resolve_get_item(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let index = map.xy_idx(self.position.x, self.position.y);
        if map.items[index].is_some() {
            let item = map.items[index].take().unwrap();

            log.log(format!("{} picked up {}", self.name, item.name));
            self.body.inventory.push(item);
        }

        vec!()
    }
    
    pub fn resolve_equip_item(&mut self, _map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let inventory_item;
        match self.intent.data.clone() {
            IntentData::InventoryItem(item) => {
                inventory_item = item;
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }
    
        match self.take_item(inventory_item) {
            Some(item) => {
                let unequipped_result = self.body.equip(item.clone());
                log.log(format!("{} equipped {}", self.name, item.name));
                match unequipped_result {
                    Ok(unequipped_items) => {
                        for unequipped_item in unequipped_items {
                            log.log(format!("{} unequipped {}", self.name, unequipped_item.name));
                            self.body.inventory.push(unequipped_item);
                        }
                    },
                    Err(_) => {
                        self.body.inventory.push(item);
                    }
                }
            }
            None => {
                debug_assert!(false);
                return vec!();
            }
        }
        
        vec!()
    }

    pub fn resolve_unequip_item(&mut self, _map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let equipped_item;
        match self.intent.data.clone() {
            IntentData::EquippedItem(item) => {
                equipped_item = item;
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }
    
        match self.body.unequip(equipped_item) {
            Some(item) => {
                log.log(format!("{} unequipped {}", self.name, item.name));
                self.body.inventory.push(item);
            },
            None => ()
        }
        
        vec!()
    }


    pub fn resolve_single_fire(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let mut result = vec!();
    
        let target_map_index;
        let target_pos;
        let item_slot;
        let bodypart;
        match self.intent.data {
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
        match self.get_equipped_item(item_slot) {
            Some(item) => {
                match item.kind {
                    ItemKind::Firearm {ammo, max_ammo, damage} => {
                        if ammo < 1 {
                            log.log(format!("{} pulled the trigger. 'Click'.", self.name));
                            return result;
                        }
                        item.kind = ItemKind::Firearm {ammo: ammo - 1, max_ammo, damage};
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
                log.log(format!("{} fired at {}", self.name, pawn.name));
            },
            _ => ()
        }

        result.push(Effect::Animation(shot_animation(self.position, target_pos, 1)));

        result
    }
    
    pub fn resolve_burst_fire(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let mut result = vec!();
    
        let target_map_index;
        let target_pos;
        let item_slot;
        let bodypart;
        match self.intent.data {
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
    
        let burst_damage;
        match self.get_equipped_item(item_slot) {
            Some(item) => {
                match item.kind {
                    ItemKind::Firearm {ammo, max_ammo, damage} => {
                        if ammo < 5 {
                            log.log(format!("{} pulled the trigger. 'Clickclickclickclickclick'.", self.name));
                            return result;
                        }
                        item.kind = ItemKind::Firearm {ammo: ammo - 5, max_ammo, damage};
                        burst_damage = damage * 5;
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
                    raw_damage: burst_damage
                });
                log.log(format!("{} fired at {}", self.name, pawn.name));
            },
            _ => ()
        }

        result.push(Effect::Animation(shot_animation(self.position, target_pos, 5)));

        result
    }

    pub fn resolve_move(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        if !self.has_ability(Ability::Move) {
            log.log(format!("{} tried to move, but couldn't", self.name));
            return vec!();
        }

        match self.intent.data {
            IntentData::Target(pos) => {
                if !map.blocked(pos.x, pos.y) {
                    self.set_position(pos, map);
                }
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }

        vec!()
    }

    pub fn resolve_turn(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        if !self.has_ability(Ability::Move) {
            log.log(format!("{} tried to turn, but couldn't", self.name));
            return vec!();
        }

        match self.intent.data {
            IntentData::Direction(direction) => {
                self.body.facing = direction;
                self.create_pawns(map);
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }
        
        vec!()
    }

    pub fn resolve_melee(&mut self, map: &mut Map, log: &mut GameLog) -> Vec<Effect> {
        let mut result = vec!();

        match self.intent.data {
            IntentData::Target(pos) => {
                let index = map.xy_idx(pos.x, pos.y);
                let id = map.pawns[index].as_ref().unwrap().entity_id;
                log.log(format!("{} struck {}", self.name, map.pawns[index].as_ref().unwrap().name));
                result.push(Effect::Damage {
                    entity_id: id,
                    bodypart_index: 1,
                    raw_damage: 1
                });
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }

        result
    }

    pub fn update_abilities(&mut self) {
        self.body.update_abilities();
    }

    pub fn has_ability(&self, ability: Ability) -> bool {
        self.body.has_ability(ability)
    }

    pub fn apply_damage(&mut self, bodypart_index: usize, raw_damage: u32) {
        let mut bodypart = &mut self.body.parts[bodypart_index];
        bodypart.damage += raw_damage;

        if bodypart.damage >= bodypart.max_damage {
            self.update_abilities();
        }

        println!("{} was hit in {} for {} damage, now has {} damage",
            self.name,
            self.body.parts[bodypart_index].name,
            raw_damage,
            self.body.parts[bodypart_index].damage);
    }

    pub fn mortally_wounded(&self) -> bool {
        for bodypart in &self.body.parts {
            if bodypart.damage > bodypart.max_damage && bodypart.vital {
                return true;
            }
        }
        return false;
    }

    pub fn kill(&mut self, map: &mut Map) {
        self.clear_pawns(map);
    }
}

/// Contains information typically needed to be referenced by others. Placed on the map for quick
/// indexing.
#[derive(Clone)]
pub struct Pawn {
    pub entity_id: usize,
    pub sprite: Sprite,
    pub sprite_index: u32,
    pub name: String,
    pub intent: Intent,
    pub body: Body
}
