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
    pub body: Body
}

impl Entity {
    pub fn create_pawn(&self) -> Pawn {
        Pawn {
            entity_id: self.id,
            renderable: self.renderable,
            name: self.name.clone(),
            intent: self.intent,
            facing: self.facing
        }
    }

    pub fn resolve_movement(&mut self, map: &mut Map) -> Option<Effect> {
        let old_index = map.xy_idx(self.position.x, self.position.y);
        let mut new_index = old_index;

        match self.intent {
            Intent::Move(pos) => {
                if !map.blocked(pos.x, pos.y) {
                    new_index = map.xy_idx(pos.x, pos.y);
                    self.position = pos;
                }

                self.intent = Intent::Idle;
            },
            Intent::Turn(direction) => {
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

                self.intent = Intent::Idle;
            },
            _ => {}
        }

        map.pawns[old_index] = None;
        map.pawns[new_index] = Some(self.create_pawn());

        None
    }

    pub fn resolve_melee(&mut self, map: &mut Map) -> Option<Effect> {
        match self.intent {
            Intent::Melee(pos) => {
                self.intent = Intent::Idle;

                let index = map.xy_idx(pos.x, pos.y);
                // TODO: check existence
                let id = map.pawns[index].as_ref().unwrap().entity_id;
                Some(Effect::Damage(id))
            },
            _ => None
        }
    }

    pub fn resolve_inventory(&mut self, map: &mut Map) -> Option<Effect> {
        match self.intent {
            Intent::GetItem => {
                self.intent = Intent::Idle;

                let index = map.xy_idx(self.position.x, self.position.y);
                if map.items[index].is_some() {
                    self.inventory.push(map.items[index].take().unwrap());
                }
                return None;
            },
            Intent::Drop(item_index) => {
                self.intent = Intent::Idle;
                
                let target_pos = map.nearest_free_item_position(self.position).unwrap();
                let map_index = map.pos_idx(target_pos);

                let item = self.inventory.remove(item_index);
                map.items[map_index] = Some(item);

                return None;
            },
            Intent::Equip(item_index) => {
                self.intent = Intent::Idle;

                let item = self.inventory.remove(item_index);
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
                return None;
            },
            Intent::Unequip(slot) => {
                self.intent = Intent::Idle;

                match self.body.unequip(slot) {
                    Some(item) => self.inventory.push(item),
                    None => ()
                }
                return None;
            },
            _ => None
        }
    }

    pub fn resolve_throw(&mut self, map: &mut Map) -> Option<Effect> {
        match self.intent {
            Intent::Throw(item_index, position) => {
                assert!(item_index < self.inventory.len());
                self.intent = Intent::Idle;

                let item = self.inventory.swap_remove(item_index);

                for item_action in item.inventory_actions {
                    match item_action {
                        ItemAction::Throw(effect_fn) => {
                            return effect_fn(self.position, position, map);
                        },
                        _ => return None
                    };
                }
                
                return None;
            },
            _ => None
        }
    }


    pub fn kill(&mut self, map: &mut Map) {
        let index = map.xy_idx(self.position.x, self.position.y);
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
    pub facing: Facing
}

