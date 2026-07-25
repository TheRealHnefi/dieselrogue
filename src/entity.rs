use crate::components::*;
use rltk::Point;
use crate::Map;
use crate::Item;
use crate::Body;

/// Concrete type containing all data of something that acts and moves.
#[derive(Clone)]
pub struct Entity {
    pub id: usize,
    pub position: Point,
    pub renderable: Renderable,
    pub name: String,
    pub intent: Intent,
    pub facing: Facing,
    pub inventory: Vec<Item>,
    pub body: Body,
    pub declare_intent: fn (entity: &mut Entity, map: &Map)
}

impl Entity {
    pub fn new_human(id: usize, pos: Point, facing: Facing, name: String) -> Self {
        Self {
            id: id,
            position: pos,
            renderable: Renderable::new_glyph('5'),
            name: name,
            intent: idle_intent(),
            facing: facing,
            inventory: vec!(),
            body: Body::human_body(),
            declare_intent: Entity::declare_intent_noop
        }
    }

    pub fn new_patrolling_goon(id: usize, pos: Point, facing: Facing, name: String) -> Self {
        Self {
            id: id,
            position: pos,
            renderable: Renderable::new_glyph('5'),
            name: name,
            intent: idle_intent(),
            facing: facing,
            inventory: vec!(),
            body: Body::human_body(),
            declare_intent: Entity::declare_intent_patrolling_goon
        }
    }

    pub fn create_pawn(&self) -> Pawn {
        Pawn {
            entity_id: self.id,
            renderable: self.renderable,
            name: self.name.clone(),
            intent: self.intent.clone(),
            facing: self.facing
        }
    }

    pub fn take_item(&mut self, item: Item) -> Option<Item> {
        if let Some(item_index) = self.inventory.iter().position(|value| *value == item) {
            Some(self.inventory.swap_remove(item_index))
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
            Some(pawn) => result.push(Effect::Damage{entity_id: pawn.entity_id}),
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
    
        let target_pos = map.nearest_free_item_position(self.position).unwrap();
        let map_index = map.pos_idx(target_pos);
    
        map.items[map_index] = self.take_item(inventory_item);
    
        vec!()
    }

    pub fn resolve_get_item(&mut self, map: &mut Map) -> Vec<Effect> {
        let index = map.xy_idx(self.position.x, self.position.y);
        if map.items[index].is_some() {
            self.inventory.push(map.items[index].take().unwrap());
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
                            self.inventory.push(unequipped_item);
                        }
                    },
                    Err(_) => {
                        self.inventory.push(item);
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
            Some(item) => self.inventory.push(item),
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
    
        match self.get_equipped_item(item_slot) {
            Some(item) => {
                match item.kind {
                    ItemKind::Firearm {ammo, max_ammo} => {
                        if ammo < 1 {
                            return result;
                        }
                        item.kind = ItemKind::Firearm {ammo: ammo - 1, max_ammo: max_ammo};
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
            Some(pawn) => result.push(Effect::Damage{entity_id: pawn.entity_id}),
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
    
        match self.get_equipped_item(item_slot) {
            Some(item) => {
                match item.kind {
                    ItemKind::Firearm {ammo, max_ammo} => {
                        if ammo < 5 {
                            return result;
                        }
                        item.kind = ItemKind::Firearm {ammo: ammo - 5, max_ammo: max_ammo};
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
            Some(pawn) => result.push(Effect::Damage{entity_id: pawn.entity_id}),
            _ => return result
        }
        result
    }

    pub fn resolve_move(&mut self, map: &mut Map) -> Vec<Effect> {
        match self.intent.data {
            IntentData::Target(pos) => {
                if !map.blocked(pos.x, pos.y) {
                    let old_index = map.xy_idx(self.position.x, self.position.y);
                    let new_index = map.xy_idx(pos.x, pos.y);
                    self.position = pos;
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
        match self.intent.data {
            IntentData::Direction(direction) => {
                self.facing.direction = direction;
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
                let index = map.xy_idx(self.position.x, self.position.y);
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
                result.push(Effect::Damage{entity_id: id})
            },
            _ => {
                debug_assert!(false);
                return vec!();
            }
        }

        result
    }

    pub fn kill(&mut self, map: &mut Map) {
        let index = map.xy_idx(self.position.x, self.position.y);
        map.pawns[index] = None;
    }


    fn declare_intent_noop(&mut self, _map: &Map) {
        return;
    }

    fn declare_intent_patrolling_goon(&mut self, _map: &Map) {
        let new_direction = match self.facing.direction {
            Direction::Left => Direction::UpLeft,
            Direction::UpLeft => Direction::Up,
            Direction::Up => Direction::UpRight,
            Direction::UpRight => Direction::Right,
            Direction::Right => Direction::DownRight,
            Direction::DownRight => Direction::Down,
            Direction::Down => Direction::DownLeft,
            Direction::DownLeft => Direction::Left
        };
        self.intent = Intent {
            phase: IntentPhase::Movement,
            data: IntentData::Direction(new_direction),
            action: Self::resolve_turn
        };
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
    pub facing: Facing
}
