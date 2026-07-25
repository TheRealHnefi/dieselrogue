use crate::components::*;
use rltk::Point;
use crate::Map;
use crate::Item;
use crate::Body;
use crate::ai::*;
use crate::Viewshed;
use crate::Ability;

/// Concrete type containing all data of something that acts and moves.
pub struct Entity {
    pub id: usize,
    pub renderable: Renderable,
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
            renderable: Renderable::new_glyph('5'),
            name: name,
            intent: idle_intent(),
            body: Body::human_body(pos, facing),
            viewshed: Viewshed::new(),
            ai: AI::None
        }
    }

    pub fn new_patrolling_goon(id: usize, pos: Point, facing: Direction, name: String, waypoints: Vec<Point>) -> Self {
        Self {
            id: id,
            renderable: Renderable::new_glyph('5'),
            name: name,
            intent: idle_intent(),
            body: Body::human_body(pos, facing),
            viewshed: Viewshed::new(),
            ai: AI::Patrolling(PatrollingAI::new(waypoints))
        }
    }

    pub fn create_pawn(&self) -> Pawn {
        Pawn {
            entity_id: self.id,
            renderable: self.renderable,
            name: self.name.clone(),
            intent: self.intent.clone(),
            body: self.body.clone()
        }
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
                self.intent = ai.declare_intent(&self.body, map);
            }
            AI::None => ()
        }
    }

    pub fn update_view(&mut self, map: &Map) {
        self.viewshed.update(self.body.position, self.body.facing, map);
    }

    pub fn resolve_throw_grenade(&mut self, map: &mut Map) -> Vec<Effect> {
        let mut result = vec!();
    
        let used_item;
        let target_map_index;
        match self.intent.data.clone() {
            IntentData::TargetWithInventory{item, target} => {
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
            _ => return result
        }
    
        result
    }
    
    pub fn resolve_drop_item(&mut self, map: &mut Map) -> Vec<Effect> {
    
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
    
        let target_pos = map.nearest_free_item_position(self.body.position).unwrap();
        let map_index = map.pos_idx(target_pos);
    
        map.items[map_index] = self.take_item(inventory_item);
    
        vec!()
    }

    pub fn resolve_get_item(&mut self, map: &mut Map) -> Vec<Effect> {
        let index = map.xy_idx(self.body.position.x, self.body.position.y);
        if map.items[index].is_some() {
            self.body.inventory.push(map.items[index].take().unwrap());
        }

        vec!()
    }
    
    pub fn resolve_equip_item(&mut self, _map: &mut Map) -> Vec<Effect> {
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
                match unequipped_result {
                    Ok(unequipped_items) => {
                        for unequipped_item in unequipped_items {
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

    pub fn resolve_unequip_item(&mut self, _map: &mut Map) -> Vec<Effect> {
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
            Some(item) => self.body.inventory.push(item),
            None => ()
        }
        
        vec!()
    }


    pub fn resolve_single_fire(&mut self, map: &mut Map) -> Vec<Effect> {
        let mut result = vec!();
    
        let target_map_index;
        let item_slot;
        match self.intent.data {
            IntentData::TargetWithEquipment{slot, target} => {
                item_slot = slot;
                target_map_index = map.pos_idx(target);
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
            Some(pawn) => result.push(Effect::Damage {
                entity_id: pawn.entity_id,
                bodypart_index: 4,
                raw_damage: shot_damage
            }),
            _ => return result
        }
        result
    }
    
    pub fn resolve_burst_fire(&mut self, map: &mut Map) -> Vec<Effect> {
        let mut result = vec!();
    
        let target_map_index;
        let item_slot;
        match self.intent.data {
            IntentData::TargetWithEquipment{slot, target} => {
                item_slot = slot;
                target_map_index = map.pos_idx(target);
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
            Some(pawn) => result.push(Effect::Damage {
                entity_id: pawn.entity_id,
                bodypart_index: 1,
                raw_damage: burst_damage
            }),
            _ => return result
        }
        result
    }

    pub fn resolve_move(&mut self, map: &mut Map) -> Vec<Effect> {
        if !self.has_ability(Ability::Move) {
            return vec!();
        }

        match self.intent.data {
            IntentData::Target(pos) => {
                if !map.blocked(pos.x, pos.y) {
                    let old_index = map.xy_idx(self.body.position.x, self.body.position.y);
                    let new_index = map.xy_idx(pos.x, pos.y);
                    self.body.position = pos;
                    map.pawns[old_index] = None;
                    map.pawns[new_index] = Some(self.create_pawn());
                }
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }

        vec!()
    }

    pub fn resolve_turn(&mut self, map: &mut Map) -> Vec<Effect> {
        if !self.has_ability(Ability::Move) {
            return vec!();
        }
        
        match self.intent.data {
            IntentData::Direction(direction) => {
                self.body.facing = direction;
                match direction {
                    Direction::Up => {self.renderable.glyph = rltk::to_cp437('8')},
                    Direction::UpRight => {self.renderable.glyph = rltk::to_cp437('9')},
                    Direction::Right => {self.renderable.glyph = rltk::to_cp437('6')},
                    Direction::DownRight => {self.renderable.glyph = rltk::to_cp437('3')},
                    Direction::Down => {self.renderable.glyph = rltk::to_cp437('2')},
                    Direction::DownLeft => {self.renderable.glyph = rltk::to_cp437('1')},
                    Direction::Left => {self.renderable.glyph = rltk::to_cp437('4')},
                    Direction::UpLeft => {self.renderable.glyph = rltk::to_cp437('7')},
                }
                let index = map.xy_idx(self.body.position.x, self.body.position.y);
                map.pawns[index] = Some(self.create_pawn());
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }
        
        vec!()
    }

    pub fn resolve_melee(&mut self, map: &mut Map) -> Vec<Effect> {
        let mut result = vec!();

        match self.intent.data {
            IntentData::Target(pos) => {
                let index = map.xy_idx(pos.x, pos.y);
                let id = map.pawns[index].as_ref().unwrap().entity_id;
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
        let index = map.xy_idx(self.body.position.x, self.body.position.y);
        map.pawns[index] = None;
    }
}

/// Contains information typically needed to be referenced by others. Placed on the map for quick
/// indexing.
#[derive(Clone)]
pub struct Pawn {
    pub entity_id: usize,
    pub renderable: Renderable,
    pub name: String,
    pub intent: Intent,
    pub body: Body
}
