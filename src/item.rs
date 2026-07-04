use rltk::Point;
use crate::components::*;
use crate::entity::Entity;
use crate::Map;
use crate::intent::*;
use crate::actions::{self, Action};

#[derive(Clone)]
pub struct Item {
    pub id: usize,
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<EntityAction>,
    pub equip_actions: Vec<EntityAction>,
    pub equip_slots: Vec<SlotType>,
    pub kind: ItemKind,
    pub proxy: bool,
    pub active: bool,
    /// Minimum zone depth at which this item may spawn.
    /// 0 = any zone, 1 = depth ≥ 1, 2 = depth ≥ 2, etc.
    pub rarity: u8,
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
    rarity: u8,
}

// ---- Firearm roster ---------------------------------------------------------

impl Item {
    pub fn revolver() -> Self {
        Item::make_firearm(FirearmDef { name: "Revolver",             glyph: 'P', fire_mode: FireMode::Single,         two_handed: false, ammo: 6,   damage: Damage::new(15,  0,  0, 0), range: 5,  rarity: 1 })
    }
    pub fn pistol() -> Self {
        Item::make_firearm(FirearmDef { name: "Pistol",               glyph: 'P', fire_mode: FireMode::Single,         two_handed: false, ammo: 12,  damage: Damage::new(10,  0,  0, 0), range: 5,  rarity: 0 })
    }
    pub fn flare_gun() -> Self {
        Item::make_firearm(FirearmDef { name: "Flare gun",            glyph: 'F', fire_mode: FireMode::Single,         two_handed: false, ammo: 1,   damage: Damage::new( 0,  0, 10, 0), range: 5,  rarity: 0 })
    }
    pub fn shock_pistol() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock pistol",         glyph: 'S', fire_mode: FireMode::Single,         two_handed: false, ammo: 5,   damage: Damage::new( 0,  3,  0, 0), range: 3,  rarity: 1 })
    }
    pub fn submachine_gun() -> Self {
        Item::make_firearm(FirearmDef { name: "SMG",                  glyph: 'A', fire_mode: FireMode::SingleAndBurst, two_handed: false, ammo: 25,  damage: Damage::new(10,  0,  0, 0), range: 5,  rarity: 1 })
    }
    pub fn bolt_action_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Bolt action rifle",    glyph: 'B', fire_mode: FireMode::Single,         two_handed: true,  ammo: 5,   damage: Damage::new(25,  0,  0, 0), range: 15, rarity: 2 })
    }
    pub fn semi_auto_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Semi-automatic rifle", glyph: 'R', fire_mode: FireMode::Single,         two_handed: true,  ammo: 10,  damage: Damage::new(20,  0,  0, 0), range: 15, rarity: 2 })
    }
    pub fn assault_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Assault rifle",        glyph: 'A', fire_mode: FireMode::SingleAndBurst, two_handed: true,  ammo: 25,  damage: Damage::new(15,  0,  0, 0), range: 12, rarity: 2 })
    }
    pub fn machinegun() -> Self {
        Item::make_firearm(FirearmDef { name: "Machine gun",          glyph: 'M', fire_mode: FireMode::Burst,          two_handed: true,  ammo: 30,  damage: Damage::new(15,  0,  0, 0), range: 10, rarity: 2 })
    }
    pub fn rotary_machinegun() -> Self {
        Item::make_firearm(FirearmDef { name: "Rotary machine gun",   glyph: 'M', fire_mode: FireMode::Burst,          two_handed: true,  ammo: 100, damage: Damage::new(12,  0,  0, 0), range: 10, rarity: 3 })
    }
    pub fn shock_carbine() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock carbine",        glyph: 'S', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 15,  damage: Damage::new( 0,  3,  0, 0), range: 6,  rarity: 2 })
    }
    pub fn shock_cannon() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock cannon",         glyph: 'S', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 2,   damage: Damage::new( 0, 25,  0, 0), range: 8,  rarity: 3 })
    }
    pub fn flamethrower() -> Self {
        Item::make_firearm(FirearmDef { name: "Flamethrower",         glyph: 'F', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 10,  damage: Damage::new( 0,  0,  3, 0), range: 10, rarity: 2 })
    }
    pub fn rocket_launcher() -> Self {
        Item::make_firearm(FirearmDef { name: "Rocket launcher",      glyph: 'R', fire_mode: FireMode::Rocket,         two_handed: true,  ammo: 1,   damage: Damage::new(500, 0,  0, 0), range: 15, rarity: 3 })
    }
    pub fn multi_rocket_launcher() -> Self {
        Item::make_firearm(FirearmDef { name: "Multi-rocket launcher", glyph: 'M', fire_mode: FireMode::Rocket,        two_handed: true,  ammo: 4,   damage: Damage::new(100, 0,  0, 0), range: 12, rarity: 3 })
    }

    // ---- Grenades ---------------------------------------------------------

    pub fn grenade() -> Self {
        let rarity = 1;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_colored_char('g', Item::rarity_to_color(rarity)),
            name: String::from("Grenade"),
            inventory_actions: vec![Item::prime_action(), Item::throw_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::FusedExplosive { damage: Damage::new(10, 0, 0, 0), timeout: 4, radius: 3, flash: false },
            proxy: false,
            active: false,
        }
    }

    pub fn fire_grenade() -> Self {
        let rarity = 2;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_colored_char('f', Item::rarity_to_color(rarity)),
            name: String::from("Fire grenade"),
            inventory_actions: vec![Item::prime_action(), Item::throw_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::FusedExplosive { damage: Damage::new(0, 0, 3, 0), timeout: 4, radius: 3, flash: false },
            proxy: false,
            active: false,
        }
    }

    pub fn shock_grenade() -> Self {
        let rarity = 2;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_colored_char('g', Item::rarity_to_color(rarity)),
            name: String::from("Shock grenade"),
            inventory_actions: vec![Item::prime_action(), Item::throw_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::FusedExplosive { damage: Damage::new(0, 7, 0, 0), timeout: 4, radius: 3, flash: false },
            proxy: false,
            active: false,
        }
    }

    pub fn flashbang() -> Self {
        let rarity = 1;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_char('f'),
            name: String::from("Flashbang"),
            inventory_actions: vec![Item::prime_action(), Item::throw_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::FusedExplosive { damage: Damage::new(0, 0, 0, 0), timeout: 4, radius: 10, flash: true },
            proxy: false,
            active: false,
        }
    }

// ---- Misc ---------------------------------------------------------

    pub fn knife() -> Self {
        let rarity = 0;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_colored_char('/', Item::rarity_to_color(rarity)),
            name: String::from("Knife"),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![SlotType::SecondaryHand],
            kind: ItemKind::MeleeWeapon { damage: Damage::new(5, 0, 0, 0) },
            proxy: false,
            active: false,
        }
    }

// ---- Armor ---------------------------------------------------------

    pub fn bulletproof_vest() -> Self {
        let rarity = 1;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_colored_char('V', Item::rarity_to_color(rarity)),
            name: String::from("Bulletproof vest"),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions: vec!(),
            equip_slots: vec!(SlotType::Bodywear),
            kind: ItemKind::Wearable { armor: Armor::new(3, 0.25, 0, 0.1, 0, 0.0) },
            proxy: false,
            active: false,
        }
    }

// ---- System items ---------------------------------------------------------

    pub fn mounted_cannon() -> Self {
        let range = 10;
        let ammo = 10;
        let rarity = 0;
        Item {
            id: 0, rarity: rarity,
            renderable: Renderable::new_colored_char('C', Item::rarity_to_color(rarity)),
            name: String::from("Mounted cannon"),
            inventory_actions: vec![],
            equip_actions: vec![Item::aim_action(range), Item::aim_at_entity_action(range), Item::fire_action()],
            equip_slots: vec![SlotType::TurretMount],
            kind: ItemKind::Firearm { ammo: ammo, max_ammo: ammo, damage: Damage::new(500, 0, 0, 0), range: range },
            proxy: false,
            active: false,
        }
    }

    pub fn corpse() -> Self {
        Item {
            id: 0, rarity: 0,
            renderable: Renderable {
                glyph: 1,
                color: rltk::RGB::from_f32(0.5, 0.2, 0.0),
                background: rltk::RGB::named(rltk::BLACK),
            },
            name: String::from("Corpse"),
            inventory_actions: vec![Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::Misc,
            proxy: false,
            active: false,
        }
    }

    pub fn rubble() -> Self {
        Item {
            id: 0, rarity: 0,
            renderable: Renderable {
                glyph: rltk::to_cp437('≈'),
                color: rltk::RGB::from_f32(0.5, 0.5, 0.5),
                background: rltk::RGB::named(rltk::BLACK),
            },
            name: String::from("Rubble"),
            inventory_actions: vec![],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::Misc,
            proxy: false,
            active: false,
        }
    }

    pub fn key(color: usize) -> Self {
        let (r, g, b) = crate::components::KEY_COLORS[color];
        Item {
            id: 0, rarity: 0,
            renderable: Renderable::new_colored_char('k', rltk::RGB::from_u8(r, g, b)),
            name: format!("{} key", crate::components::KEY_COLOR_NAMES[color]),
            inventory_actions: vec![Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::Key { color },
            proxy: false,
            active: false,
        }
    }


    pub fn is_droppable(&self) -> bool {
        self.inventory_actions.iter().any(|a| {
            std::ptr::fn_addr_eq(a.action, actions::drop_item_action as Action)
        })
    }

    pub fn proxy(&self) -> Self {
        Item {
            id: self.id,
            rarity: self.rarity,
            renderable: self.renderable.clone(),
            name: self.name.clone(),
            inventory_actions: vec!(),
            equip_actions: vec!(),
            equip_slots: self.equip_slots.clone(),
            kind: ItemKind::Misc,
            proxy: true,
            active: false,
        }
    }
}

// ---- Construction helpers ---------------------------------------------------

impl Item {
    fn make_firearm(def: FirearmDef) -> Item {
        let equip_actions = match def.fire_mode {
            FireMode::Single         => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_action()],
            FireMode::Burst          => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_burst_action()],
            FireMode::SingleAndBurst => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_action(), Item::fire_burst_action()],
            FireMode::Rocket         => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_rocket_action()],
            FireMode::Fan            => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fan_fire_action()],
        };
        let equip_slots = if def.two_handed {
            vec![SlotType::PrimaryHand, SlotType::SecondaryHand]
        } else {
            vec![SlotType::PrimaryHand]
        };
        
        Item {
            id: 0,
            rarity: def.rarity,
            renderable: Renderable::new_colored_char(def.glyph, Item::rarity_to_color(def.rarity)),
            name: def.name.to_string(),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions,
            equip_slots,
            kind: ItemKind::Firearm { ammo: def.ammo, max_ammo: def.ammo, damage: def.damage, range: def.range },
            proxy: false,
            active: false,
        }
    }

    fn equip_action() -> EntityAction {
        EntityAction { name: "Equip".to_string(),            targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::equip_item_action    }
    }
    fn drop_action() -> EntityAction {
        EntityAction { name: "Drop".to_string(),             targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::drop_item_action     }
    }
    fn aim_action(range: u32) -> EntityAction {
        EntityAction { name: "Aim at position".to_string(),  targeting: Targeting::Positional { max_range: Some(range) }, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::aim_action           }
    }
    fn fire_action() -> EntityAction {
        EntityAction { name: "Fire shot".to_string(),        targeting: Targeting::UseExistingAim { ask_bodypart: true  }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::single_fire_action   }
    }
    fn fire_burst_action() -> EntityAction {
        EntityAction { name: "Fire burst".to_string(),       targeting: Targeting::UseExistingAim { ask_bodypart: true  }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::burst_fire_action    }
    }
    fn fire_rocket_action() -> EntityAction {
        EntityAction { name: "Fire rocket".to_string(),      targeting: Targeting::UseExistingAim { ask_bodypart: false }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::rocket_fire_action   }
    }
    fn fan_fire_action() -> EntityAction {
        EntityAction { name: "Fan fire".to_string(),         targeting: Targeting::UseExistingAim { ask_bodypart: false }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::fan_fire_action      }
    }
    fn aim_at_entity_action(range: u32) -> EntityAction {
        EntityAction { name: "Aim at entity".to_string(),    targeting: Targeting::EntityAim { max_range: Some(range) },   phase: ExecutionPhase::Attack, precondition: precondition_ok,        action: actions::aim_action           }
    }
    fn prime_action() -> EntityAction {
        EntityAction { name: "Prime".to_string(),            targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::prime_grenade_action }
    }
    fn throw_action() -> EntityAction {
        EntityAction { name: "Throw".to_string(),            targeting: Targeting::Positional { max_range: Some(5) }, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::throw_grenade_action }
    }

    fn rarity_to_color(rarity: u8) -> rltk::RGB {
        match rarity {
            0 => rltk::RGB::named(rltk::GRAY),
            1 => rltk::RGB::named(rltk::GREEN),
            2 => rltk::RGB::named(rltk::BLUE),
            3 => rltk::RGB::named(rltk::PURPLE),
            _ => rltk::RGB::named(rltk::WHITE)
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub fn precondition_is_aiming(self_ref: &Entity, _map: &Map, item: Option<&Item>) -> bool {
    let key = StatusEffect::AimingAtGround(Point { x: 0, y: 0 }, Item::pistol());
    let aimed_item_id = match self_ref.body.get_status_effect(&key) {
        Some(StatusEffect::AimingAtGround(_, i)) => i.id,
        Some(StatusEffect::AimingAtEntity(_, i)) => i.id,
        _ => return false,
    };
    match item {
        Some(i2) => aimed_item_id == i2.id,
        _ => false,
    }
}
