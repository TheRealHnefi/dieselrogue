use crate::components::*;
use crate::ability::*;
use crate::Map;
use rltk::Point;

#[derive(Clone)]
pub struct Item {
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<ItemAction>,
    pub equip_slots: Vec<SlotType>,
    pub equip_abilities: Vec<Ability>,
    pub proxy: bool
}

impl Item {
    pub fn grenade() -> Self {
        let throw_action = ItemAction::Throw(throw_grenade_effect);
        let drop_action = ItemAction::Drop;
        Item {
            renderable: Renderable::new_glyph('g'),
            name: String::from("Grenade"),
            inventory_actions: vec![throw_action, drop_action],
            equip_slots: vec!(),
            equip_abilities: vec!(),
            proxy: false
        }
    }

    pub fn pistol() -> Self {
        let equip_action = ItemAction::Equip;
        let drop_action = ItemAction::Drop;
        let fire_ability = Ability { name: "Fire".to_string() };
        Item {
            renderable: Renderable::new_glyph('p'),
            name: String::from("Pistol"),
            inventory_actions: vec![equip_action, drop_action],
            equip_slots: vec!(SlotType::PrimaryHand),
            equip_abilities: vec!(fire_ability),
            proxy: false
        }
    }

    pub fn rifle() -> Self {
        let equip_action = ItemAction::Equip;
        let drop_action = ItemAction::Drop;
        let fire_ability = Ability { name: "Fire".to_string() };
        Item {
            renderable: Renderable::new_glyph('r'),
            name: String::from("Rifle"),
            inventory_actions: vec![equip_action, drop_action],
            equip_slots: vec!(SlotType::PrimaryHand, SlotType::SecondaryHand),
            equip_abilities: vec!(fire_ability),
            proxy: false
        }
    }

    pub fn proxy(&self) -> Self {
        Item {
            renderable: self.renderable.clone(),
            name: self.name.clone(),
            inventory_actions: vec!(),
            equip_slots: self.equip_slots.clone(),
            equip_abilities: vec!(),
            proxy: true
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn throw_grenade_effect(_source_position: Point, target_position: Point, map: &Map) -> Option<Effect> {
    let target_map_index = map.pos_idx(target_position);
    match &map.pawns[target_map_index] {
        Some(pawn) => Some(Effect::Damage(pawn.entity_id)),
        _ => None
    }
}