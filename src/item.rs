use rltk::Point;
use crate::components::*;
use crate::entity::Entity;
use crate::Map;
use crate::intent::*;
use crate::actions;

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

// ---- Firearm definition types -----------------------------------------------

/// Which fire actions a firearm exposes in the equip menu.
enum FireMode {
    Single,           // aim + fire shot
    Burst,            // aim + fire burst (no single shot)
    SingleAndBurst,   // aim + fire shot + fire burst
    Rocket,           // aim + fire rocket
    Fan,              // aim + fan fire
}

/// All varying parameters needed to construct a firearm Item.
struct FirearmDef {
    name: &'static str,
    glyph: char,
    fire_mode: FireMode,
    two_handed: bool,
    ammo: u32,
    damage: Damage,
    range: u32,
}

// ---- Firearm roster ---------------------------------------------------------

impl Item {
    pub fn revolver() -> Self {
        Item::make_firearm(FirearmDef { name: "Revolver",             glyph: 'p', fire_mode: FireMode::Single,         two_handed: false, ammo: 6,   damage: Damage::new(15,  0,  0, 0), range: 5  })
    }
    pub fn pistol() -> Self {
        Item::make_firearm(FirearmDef { name: "Pistol",               glyph: 'p', fire_mode: FireMode::Single,         two_handed: false, ammo: 12,  damage: Damage::new(10,  0,  0, 0), range: 5  })
    }
    pub fn flare_gun() -> Self {
        Item::make_firearm(FirearmDef { name: "Flare gun",            glyph: 'r', fire_mode: FireMode::Single,         two_handed: false, ammo: 1,   damage: Damage::new( 0,  0, 10, 0), range: 5  })
    }
    pub fn shock_pistol() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock pistol",         glyph: 'r', fire_mode: FireMode::Fan,            two_handed: false, ammo: 5,   damage: Damage::new( 0,  5,  0, 0), range: 3  })
    }
    pub fn submachine_gun() -> Self {
        Item::make_firearm(FirearmDef { name: "SMG",                  glyph: 'p', fire_mode: FireMode::SingleAndBurst, two_handed: false, ammo: 25,  damage: Damage::new(10,  0,  0, 0), range: 5  })
    }
    pub fn bolt_action_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Bolt action rifle",    glyph: 'p', fire_mode: FireMode::Single,         two_handed: true,  ammo: 5,   damage: Damage::new(25,  0,  0, 0), range: 15 })
    }
    pub fn semi_auto_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Semi-automatic rifle", glyph: 'p', fire_mode: FireMode::Single,         two_handed: true,  ammo: 10,  damage: Damage::new(20,  0,  0, 0), range: 15 })
    }
    pub fn assault_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Assault rifle",        glyph: 'm', fire_mode: FireMode::SingleAndBurst, two_handed: true,  ammo: 25,  damage: Damage::new(15,  0,  0, 0), range: 12 })
    }
    pub fn machinegun() -> Self {
        Item::make_firearm(FirearmDef { name: "Machine gun",          glyph: 'm', fire_mode: FireMode::Burst,          two_handed: true,  ammo: 30,  damage: Damage::new(15,  0,  0, 0), range: 10 })
    }
    pub fn rotary_machinegun() -> Self {
        Item::make_firearm(FirearmDef { name: "Rotary machine gun",   glyph: 'm', fire_mode: FireMode::Burst,          two_handed: true,  ammo: 100, damage: Damage::new(12,  0,  0, 0), range: 10 })
    }
    pub fn shock_carbine() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock carbine",        glyph: 'r', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 15,  damage: Damage::new( 0, 10,  0, 0), range: 6  })
    }
    pub fn shock_cannon() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock cannon",         glyph: 'r', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 2,   damage: Damage::new( 0, 25,  0, 0), range: 8  })
    }
    pub fn flamethrower() -> Self {
        Item::make_firearm(FirearmDef { name: "Flamethrower",         glyph: 'r', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 10,  damage: Damage::new( 0,  0, 10, 0), range: 5  })
    }
    pub fn rocket_launcher() -> Self {
        Item::make_firearm(FirearmDef { name: "Rocket launcher",      glyph: 'r', fire_mode: FireMode::Rocket,         two_handed: true,  ammo: 1,   damage: Damage::new(500, 0,  0, 0), range: 15 })
    }
    pub fn multi_rocket_launcher() -> Self {
        Item::make_firearm(FirearmDef { name: "Multi-rocket launcher", glyph: 'r', fire_mode: FireMode::Rocket,        two_handed: true,  ammo: 4,   damage: Damage::new(100, 0,  0, 0), range: 12 })
    }

    // TODO: grenade_launcher, frag_grenade, shock_grenade, fire_grenade, flashbang

    pub fn bulletproof_vest() -> Self {
        Item {
            id: 0,
            renderable: Renderable::new_char('v'),
            name: String::from("Bulletproof vest"),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions: vec!(),
            equip_slots: vec!(SlotType::Bodywear),
            kind: ItemKind::Wearable { armor: Armor::new(3, 0.25, 0, 0.1, 0, 0.0) },
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

// ---- Construction helpers ---------------------------------------------------

impl Item {
    fn make_firearm(def: FirearmDef) -> Item {
        let equip_actions = match def.fire_mode {
            FireMode::Single        => vec![Item::aim_action(), Item::fire_action()],
            FireMode::Burst         => vec![Item::aim_action(), Item::fire_burst_action()],
            FireMode::SingleAndBurst => vec![Item::aim_action(), Item::fire_action(), Item::fire_burst_action()],
            FireMode::Rocket        => vec![Item::aim_action(), Item::fire_rocket_action()],
            FireMode::Fan           => vec![Item::aim_action(), Item::fan_fire_action()],
        };
        let equip_slots = if def.two_handed {
            vec![SlotType::PrimaryHand, SlotType::SecondaryHand]
        } else {
            vec![SlotType::PrimaryHand]
        };
        Item {
            id: 0,
            renderable: Renderable::new_char(def.glyph),
            name: def.name.to_string(),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions,
            equip_slots,
            kind: ItemKind::Firearm { ammo: def.ammo, max_ammo: def.ammo, damage: def.damage, range: def.range },
            proxy: false,
        }
    }

    fn equip_action() -> ItemAction {
        ItemAction { name: "Equip".to_string(),            targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::equip_item_action    }
    }
    fn drop_action() -> ItemAction {
        ItemAction { name: "Drop".to_string(),             targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::drop_item_action     }
    }
    fn aim_action() -> ItemAction {
        ItemAction { name: "Aim at position".to_string(),  targeting: Targeting::Positional, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::aim_action           }
    }
    fn fire_action() -> ItemAction {
        ItemAction { name: "Fire shot".to_string(),        targeting: Targeting::Detailed,   phase: ExecutionPhase::Attack,    precondition: precondition_is_aiming, action: actions::single_fire_action   }
    }
    fn fire_burst_action() -> ItemAction {
        ItemAction { name: "Fire burst".to_string(),       targeting: Targeting::Detailed,   phase: ExecutionPhase::Attack,    precondition: precondition_is_aiming, action: actions::burst_fire_action    }
    }
    fn fire_rocket_action() -> ItemAction {
        ItemAction { name: "Fire rocket".to_string(),      targeting: Targeting::Positional, phase: ExecutionPhase::Attack,    precondition: precondition_is_aiming, action: actions::rocket_fire_action   }
    }
    fn fan_fire_action() -> ItemAction {
        ItemAction { name: "Fan fire".to_string(),         targeting: Targeting::Positional, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::fan_fire_action      }
    }
    fn throw_action() -> ItemAction {
        ItemAction { name: "Throw".to_string(),            targeting: Targeting::Positional, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::throw_grenade_action }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub fn precondition_is_aiming(self_ref: &Entity, _map: &Map, item: Option<&Item>) -> bool {
    let aiming = self_ref.body.get_status_effect(&StatusEffect::AimingAtGround(Point {x: 0, y: 0}, Item::pistol()));
    match aiming {
        Some(aim) => {
            match &aim {
                StatusEffect::AimingAtGround(_p, i) => {
                    match item {
                        Some(i2) => i.id == i2.id,
                        _ => false
                    }
                }
                _ => false
            }
        }
        None => false
    }
}
