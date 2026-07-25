use rltk::Point;
use crate::components::*;
use crate::entity::Entity;
use crate::Map;
use crate::intent::*;

#[derive(Clone)]
pub struct Item {
    pub id: usize,
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<IntentAction>,
    pub equip_actions: Vec<IntentAction>,
    pub equip_slots: Vec<SlotType>,
    pub kind: ItemKind,
    pub proxy: bool
}

impl Item {
    pub fn revolver() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('p'),
            name: String::from("Revolver"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action()
            ),
            equip_slots: vec!(SlotType::PrimaryHand),
            kind: ItemKind::Firearm {
                ammo: 6,
                max_ammo: 6,
                damage: Damage::new(15, 0, 0, 0),
                range: 5
            },
            proxy: false
        }
    }

    pub fn pistol() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('p'),
            name: String::from("Pistol"),
            inventory_actions: vec!(
                Item::equip_action(), Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action()
            ),
            equip_slots: vec!(SlotType::PrimaryHand),
            kind: ItemKind::Firearm {
                ammo: 12,
                max_ammo: 12,
                damage: Damage::new(10, 0, 0, 0),
                range: 5
            },
            proxy: false
        }
    }

    pub fn submachine_gun() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('p'),
            name: String::from("SMG"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action(),
                Item::fire_burst_action()
            ),
            equip_slots: vec!(SlotType::PrimaryHand),
            kind: ItemKind::Firearm {
                ammo: 25,
                max_ammo: 25,
                damage: Damage::new(10, 0, 0, 0),
                range: 5
            },
            proxy: false
        }
    }

    pub fn bolt_action_rifle() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('p'),
            name: String::from("Bolt action rifle"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 5,
                max_ammo: 5,
                damage: Damage::new(25, 0, 0, 0),
                range: 15
            },
            proxy: false
        }
    }

    pub fn semi_auto_rifle() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('p'),
            name: String::from("Semi-automatic rifle"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 10,
                max_ammo: 10,
                damage: Damage::new(20, 0, 0, 0),
                range: 15
            },
            proxy: false
        }
    }

    pub fn assault_rifle() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('m'),
            name: String::from("Assault rifle"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action(),
                Item::fire_burst_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 25,
                max_ammo: 25,
                damage: Damage::new(15, 0, 0, 0),
                range: 12
            },
            proxy: false
        }
    }

    pub fn machinegun() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('m'),
            name: String::from("Machine gun"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_burst_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 30,
                max_ammo: 30,
                damage: Damage::new(15, 0, 0, 0),
                range: 10
            },
            proxy: false
        }
    }

    pub fn rotary_machinegun() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('m'),
            name: String::from("Rotary machine gun"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_burst_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 100,
                max_ammo: 100,
                damage: Damage::new(12, 0, 0, 0),
                range: 10
            },
            proxy: false
        }
    }

    pub fn rocket_launcher() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Rocket launcher"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_rocket_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 1,
                max_ammo: 1,
                damage: Damage::new(500, 0, 0, 0),
                range: 15
            },
            proxy: false
        }
    }

    pub fn multi_rocket_launcher() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Multi-rocket launcher"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_rocket_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 4,
                max_ammo: 4,
                damage: Damage::new(100, 0, 0, 0),
                range: 12
            },
            proxy: false
        }
    }

    pub fn flamethrower() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Flamethrower"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fan_fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 10,
                max_ammo: 10,
                damage: Damage::new(0, 0, 10, 0),
                range: 5
            },
            proxy: false
        }
    }

    pub fn flare_gun() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Flare gun"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 1,
                max_ammo: 1,
                damage: Damage::new(0, 0, 10, 0),
                range: 5
            },
            proxy: false
        }
    }

    pub fn shock_pistol() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Shock pistol"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fan_fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 5,
                max_ammo: 5,
                damage: Damage::new(0, 5, 0, 0),
                range: 3
            },
            proxy: false
        }
    }

    pub fn shock_carbine() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Shock carbine"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fan_fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 15,
                max_ammo: 15,
                damage: Damage::new(0, 10, 0, 0),
                range: 6
            },
            proxy: false
        }
    }

    pub fn shock_cannon() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('r'),
            name: String::from("Shock cannon"),
            inventory_actions: vec!(
                Item::equip_action(),
                Item::drop_action()
            ),
            equip_actions: vec!(
                Item::aim_action(),
                Item::fan_fire_action()
            ),
            equip_slots: vec!(
                SlotType::PrimaryHand,
                SlotType::SecondaryHand
            ),
            kind: ItemKind::Firearm {
                ammo: 2,
                max_ammo: 2,
                damage: Damage::new(0, 25, 0, 0),
                range: 8
            },
            proxy: false
        }
    }

    // TODO:
    // grenade_launcher
    // frag_grenade
    // shock_grenade
    // fire_grenade
    // flashbang

    pub fn bulletproof_vest() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('v'),
            name: String::from("Bulletproof vest"),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions: vec!(),
            equip_slots: vec!(SlotType::Bodywear),
            kind: ItemKind::Wearable {
                armor: Armor::new(3, 0.25, 0, 0.1, 0, 0.0)
            },
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

    fn equip_action() -> IntentAction {
        IntentAction {
            name: "Equip".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            precondition: precondition_ok,
            effects: Entity::resolve_equip_item
        }
    }
    fn drop_action() -> IntentAction {
        IntentAction {
            name: "Drop".to_string(),
            targeting: Targeting::None,
            phase: IntentPhase::Inventory,
            precondition: precondition_ok,
            effects: Entity::resolve_drop_item
        }
    }
    fn aim_action() -> IntentAction {
        IntentAction {
            name: "Aim at position".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            precondition: precondition_ok,
            effects: Entity::resolve_aim
        }
    }
    fn fire_action() -> IntentAction {
        IntentAction {
            name: "Fire shot".to_string(),
            targeting: Targeting::Detailed,
            phase: IntentPhase::Attack,
            precondition: precondition_is_aiming,
            effects: Entity::resolve_single_fire
        }
    }
    fn fire_burst_action() -> IntentAction {
        IntentAction {
            name: "Fire burst".to_string(),
            targeting: Targeting::Detailed,
            phase: IntentPhase::Attack,
            precondition: precondition_is_aiming,
            effects: Entity::resolve_burst_fire
        }
    }
    fn fire_rocket_action() -> IntentAction {
        IntentAction {
            name: "Fire rocket".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            precondition: precondition_is_aiming,
            effects: Entity::resolve_rocket_fire
        }
    }
    fn fan_fire_action() -> IntentAction {
        IntentAction {
            name: "Fan fire".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            precondition: precondition_ok,
            effects: Entity::resolve_fan_fire
        }
    }
    fn throw_action() -> IntentAction {
            IntentAction {
            name: "Throw".to_string(),
            targeting: Targeting::Positional,
            phase: IntentPhase::Attack,
            precondition: precondition_ok,
            effects: Entity::resolve_throw_grenade
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub fn precondition_is_aiming(self_ref: &Entity, _map: &Map) -> bool {
    let aiming = self_ref.body.get_status_effect(&StatusEffect::AimingAtGround(Point {x: 0, y: 0}));
    match aiming {
        Some(_) => true,
        None => false
    }
}
