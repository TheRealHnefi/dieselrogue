use crate::components::*;
use crate::entity::Entity;
use crate::map::Map;
use crate::intent::*;
use crate::GameLog;

#[derive(Clone)]
pub struct Item {
    pub id: usize,
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<ItemAction>,
    pub equip_actions: Vec<ItemAction>,
    pub equip_slots: Vec<SlotType>,
    pub kind: ItemKind,
    pub proxy: bool
}

#[derive(Copy, Clone, PartialEq)]
pub enum Targeting {
    None,
    Positional,
    Detailed
}

#[derive(Clone)]
pub struct ItemAction {
    pub name: String,
    pub targeting: Targeting,
    pub phase: IntentPhase,
    pub effects: fn (self_ref: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect>
}

impl Item {
    pub fn grenade(id: usize) -> Self {
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
            id: id,
            renderable: Renderable::new_char('g'),
            name: String::from("Grenade"),
            inventory_actions: vec![throw_action, drop_action],
            equip_actions: vec!(),
            equip_slots: vec!(),
            kind: ItemKind::Misc,
            proxy: false
        }
    }

    pub fn rocket_launcher(id: usize) -> Self {
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
            effects: Entity::resolve_rocket_fire
        };
        Item {
            id: id,
            renderable: Renderable::new_char('r'),
            name: String::from("Rocket launcher"),
            inventory_actions: vec![equip_action, drop_action],
            equip_actions: vec!(fire_action),
            equip_slots: vec!(SlotType::PrimaryHand, SlotType::SecondaryHand),
            kind: ItemKind::Firearm {ammo: 1, max_ammo: 1, damage: 500},
            proxy: false
        }
    }

    pub fn pistol(id: usize) -> Self {
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
            targeting: Targeting::Detailed,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_single_fire
        };
        Item {
            id: id,
            renderable: Renderable::new_char('p'),
            name: String::from("Pistol"),
            inventory_actions: vec![equip_action, drop_action],
            equip_actions: vec!(fire_action),
            equip_slots: vec!(SlotType::PrimaryHand),
            kind: ItemKind::Firearm {ammo: 5, max_ammo: 5, damage: 5},
            proxy: false
        }
    }

    pub fn machinegun(id: usize) -> Self {
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
            targeting: Targeting::Detailed,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_single_fire
        };
        let fire_burst_action = ItemAction {
            name: "Fire burst".to_string(),
            targeting: Targeting::Detailed,
            phase: IntentPhase::Attack,
            effects: Entity::resolve_burst_fire
        };
        Item {
            id: id,
            renderable: Renderable::new_char('m'),
            name: String::from("Machinegun"),
            inventory_actions: vec![equip_action, drop_action],
            equip_actions: vec!(fire_action, fire_burst_action),
            equip_slots: vec!(SlotType::PrimaryHand, SlotType::SecondaryHand),
            kind: ItemKind::Firearm {ammo: 30, max_ammo: 30, damage: 5},
            proxy: false
        }
    }

    pub fn proxy(&self) -> Self {
        Item {
            id: self.id,
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
        self.id == other.id
    }
}

