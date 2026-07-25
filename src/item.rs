use crate::components::*;
use crate::Map;
use rltk::Point;

#[derive(Clone)]
pub struct Item {
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<ItemAction>
}

impl Item {
    pub fn grenade() -> Self {
        let throw_action = ItemAction::Throw(throw_grenade_effect);
        Item {
            renderable: Renderable::new_glyph('g'),
            name: String::from("Grenade"),
            inventory_actions: vec![throw_action]
        }
    }
}

fn throw_grenade_effect(source_position: Point, target_position: Point, map: &Map) -> Option<Effect> {
    let target_map_index = map.pos_idx(target_position);
    match &map.pawns[target_map_index] {
        Some(pawn) => Some(Effect::Damage(pawn.entity_id)),
        _ => None
    }
}