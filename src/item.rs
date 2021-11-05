use crate::components::*;
use crate::Map;
use rltk::Point;

#[derive(Clone)]
pub struct Item {
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<ItemAction>,
    pub equip_slots: Vec<SlotType>,
    pub equip_abilities: Vec<ItemAbility>,
    pub kind: ItemKind,
    pub proxy: bool
}

#[derive(Clone)]
pub enum ItemKind {
    Firearm {ammo: u32, max_ammo: u32},
    Misc
}

#[derive(Clone)]
pub struct ItemAbility {
    pub name: String,
    pub effect: fn (source_position: Point, target_position: Point, map: &Map, item: &mut Item) -> Option<Effect>
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
            kind: ItemKind::Misc,
            proxy: false
        }
    }

    pub fn pistol() -> Self {
        let equip_action = ItemAction::Equip;
        let drop_action = ItemAction::Drop;
        let fire_ability = ItemAbility { name: "Fire".to_string(), effect: single_fire_effect};
        Item {
            renderable: Renderable::new_glyph('p'),
            name: String::from("Pistol"),
            inventory_actions: vec![equip_action, drop_action],
            equip_slots: vec!(SlotType::PrimaryHand),
            equip_abilities: vec!(fire_ability),
            kind: ItemKind::Firearm {ammo: 5, max_ammo: 5},
            proxy: false
        }
    }

    pub fn machinegun() -> Self {
        let equip_action = ItemAction::Equip;
        let drop_action = ItemAction::Drop;
        let fire_ability = ItemAbility { name: "Fire shot".to_string(), effect: single_fire_effect };
        let fire_burst_ability = ItemAbility { name: "Fire burst".to_string(), effect: burst_fire_effect};
        Item {
            renderable: Renderable::new_glyph('m'),
            name: String::from("Machinegun"),
            inventory_actions: vec![equip_action, drop_action],
            equip_slots: vec!(SlotType::PrimaryHand, SlotType::SecondaryHand),
            equip_abilities: vec!(fire_ability, fire_burst_ability),
            kind: ItemKind::Firearm {ammo: 30, max_ammo: 30},
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
            kind: ItemKind::Misc,
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

fn single_fire_effect(_source_position: Point, target_position: Point, map: &Map, item: &mut Item) -> Option<Effect> {
    match item.kind {
        ItemKind::Firearm{ammo, max_ammo} => {
            if ammo <= 0 {
                return None
            }
            item.kind = ItemKind::Firearm {ammo: ammo - 1, max_ammo: max_ammo};
        },
        _ => return None
    }

    let target_map_index = map.pos_idx(target_position);
    match &map.pawns[target_map_index] {
        Some(pawn) => Some(Effect::Damage(pawn.entity_id)),
        _ => None
    }
}

fn burst_fire_effect(_source_position: Point, target_position: Point, map: &Map, item: &mut Item) -> Option<Effect> {
    match item.kind {
        ItemKind::Firearm{ammo, max_ammo} => {
            if ammo <= 5 {
                return None
            }
            item.kind = ItemKind::Firearm {ammo: ammo - 5, max_ammo: max_ammo};
        },
        _ => return None
    }

    let target_map_index = map.pos_idx(target_position);
    match &map.pawns[target_map_index] {
        Some(pawn) => Some(Effect::Damage(pawn.entity_id)),
        _ => None
    }
}