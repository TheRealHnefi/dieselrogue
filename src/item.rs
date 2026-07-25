use crate::components::*;
use crate::entity::Entity;
use crate::map::Map;

#[derive(Clone)]
pub struct Item {
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<ItemAction>,
    pub equip_actions: Vec<ItemAction>,
    pub equip_slots: Vec<SlotType>,
    pub kind: ItemKind,
    pub proxy: bool
}

#[derive(Copy, Clone)]
pub enum Targeting {
    None,
    Positional
}

#[derive(Clone)]
pub struct ItemAction {
    pub name: String,
    // pub preconditions <- check item charges and such
    pub targeting: Targeting,
    pub phase: IntentPhase,
    pub effects: fn (self_ref: &mut Entity, map: &mut Map) -> Vec<Effect>
}

impl Item {
    pub fn grenade() -> Self {
        let throw_action = ItemAction {
            name: "Throw".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_throw_grenade
        };
        let drop_action = ItemAction {
            name: "Drop".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            effects: Entity::resolve_drop_item
        };
        Item {
            renderable: Renderable::new_glyph('g'),
            name: String::from("Grenade"),
            inventory_actions: vec![throw_action, drop_action],
            equip_actions: vec!(),
            equip_slots: vec!(),
            kind: ItemKind::Misc,
            proxy: false
        }
    }

    pub fn pistol() -> Self {
        let equip_action = ItemAction {
            name: "Equip".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            effects: Entity::resolve_equip_item
        };
        let drop_action = ItemAction {
            name: "Drop".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            effects: Entity::resolve_drop_item
        };
        let fire_action = ItemAction {
            name: "Fire".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_single_fire
        };
        Item {
            renderable: Renderable::new_glyph('p'),
            name: String::from("Pistol"),
            inventory_actions: vec![equip_action, drop_action],
            equip_actions: vec!(fire_action),
            equip_slots: vec!(SlotType::PrimaryHand),
            kind: ItemKind::Firearm {ammo: 5, max_ammo: 5},
            proxy: false
        }
    }

    pub fn machinegun() -> Self {
        let equip_action = ItemAction {
            name: "Equip".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            effects: Entity::resolve_equip_item
        };
        let drop_action = ItemAction {
            name: "Drop".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            effects: Entity::resolve_drop_item
        };
        let fire_action = ItemAction {
            name: "Fire shot".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_single_fire
        };
        let fire_burst_action = ItemAction {
            name: "Fire burst".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_burst_fire
        };
        Item {
            renderable: Renderable::new_glyph('m'),
            name: String::from("Machinegun"),
            inventory_actions: vec![equip_action, drop_action],
            equip_actions: vec!(fire_action, fire_burst_action),
            equip_slots: vec!(SlotType::PrimaryHand, SlotType::SecondaryHand),
            kind: ItemKind::Firearm {ammo: 30, max_ammo: 30},
            proxy: false
        }
    }

    pub fn proxy(&self) -> Self {
        Item {
            renderable: self.renderable.clone(),
            name: self.name.clone(),
            inventory_actions: vec!(),
            equip_actions: vec!(),
            equip_slots: self.equip_slots.clone(),
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

