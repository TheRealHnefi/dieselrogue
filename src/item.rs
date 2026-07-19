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
    pub locked: bool,
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
    ammo_kind: AmmoKind,
    damage: Damage,
    range: u32,
    rarity: u8,
}

// ---- Firearm roster ---------------------------------------------------------

impl Item {
    pub fn revolver() -> Self {
        Item::make_firearm(FirearmDef { name: "Revolver",             glyph: 'P', fire_mode: FireMode::Single,         two_handed: false, ammo: 6,   ammo_kind: AmmoKind::Bullets,   damage: Damage::new(15,  0,  0, 0), range: 5,  rarity: 1 })
    }
    pub fn pistol() -> Self {
        Item::make_firearm(FirearmDef { name: "Pistol",               glyph: 'P', fire_mode: FireMode::Single,         two_handed: false, ammo: 12,  ammo_kind: AmmoKind::Bullets,   damage: Damage::new(10,  0,  0, 0), range: 5,  rarity: 0 })
    }
    pub fn flare_gun() -> Self {
        Item::make_firearm(FirearmDef { name: "Flare gun",            glyph: 'F', fire_mode: FireMode::Single,         two_handed: false, ammo: 1,   ammo_kind: AmmoKind::Fuel,      damage: Damage::new( 0,  0, 10, 0), range: 5,  rarity: 0 })
    }
    pub fn shock_pistol() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock pistol",         glyph: 'S', fire_mode: FireMode::Single,         two_handed: false, ammo: 5,   ammo_kind: AmmoKind::Batteries, damage: Damage::new( 0,  3,  0, 0), range: 3,  rarity: 1 })
    }
    pub fn submachine_gun() -> Self {
        Item::make_firearm(FirearmDef { name: "SMG",                  glyph: 'A', fire_mode: FireMode::SingleAndBurst, two_handed: false, ammo: 25,  ammo_kind: AmmoKind::Bullets,   damage: Damage::new(10,  0,  0, 0), range: 5,  rarity: 1 })
    }
    pub fn bolt_action_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Bolt action rifle",    glyph: 'B', fire_mode: FireMode::Single,         two_handed: true,  ammo: 5,   ammo_kind: AmmoKind::Bullets,   damage: Damage::new(25,  0,  0, 0), range: 15, rarity: 2 })
    }
    pub fn semi_auto_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Semi-automatic rifle", glyph: 'R', fire_mode: FireMode::Single,         two_handed: true,  ammo: 10,  ammo_kind: AmmoKind::Bullets,   damage: Damage::new(20,  0,  0, 0), range: 15, rarity: 2 })
    }
    pub fn assault_rifle() -> Self {
        Item::make_firearm(FirearmDef { name: "Assault rifle",        glyph: 'A', fire_mode: FireMode::SingleAndBurst, two_handed: true,  ammo: 25,  ammo_kind: AmmoKind::Bullets,   damage: Damage::new(15,  0,  0, 0), range: 12, rarity: 2 })
    }
    pub fn machinegun() -> Self {
        Item::make_firearm(FirearmDef { name: "Machine gun",          glyph: 'M', fire_mode: FireMode::Burst,          two_handed: true,  ammo: 30,  ammo_kind: AmmoKind::Bullets,   damage: Damage::new(15,  0,  0, 0), range: 10, rarity: 2 })
    }
    pub fn rotary_machinegun() -> Self {
        Item::make_firearm(FirearmDef { name: "Rotary machine gun",   glyph: 'M', fire_mode: FireMode::Burst,          two_handed: true,  ammo: 100, ammo_kind: AmmoKind::Bullets,   damage: Damage::new(12,  0,  0, 0), range: 10, rarity: 3 })
    }
    pub fn shock_carbine() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock carbine",        glyph: 'S', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 15,  ammo_kind: AmmoKind::Batteries, damage: Damage::new( 0,  3,  0, 0), range: 6,  rarity: 2 })
    }
    pub fn shock_cannon() -> Self {
        Item::make_firearm(FirearmDef { name: "Shock cannon",         glyph: 'S', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 2,   ammo_kind: AmmoKind::Batteries, damage: Damage::new( 0, 25,  0, 0), range: 8,  rarity: 3 })
    }
    pub fn flamethrower() -> Self {
        Item::make_firearm(FirearmDef { name: "Flamethrower",         glyph: 'F', fire_mode: FireMode::Fan,            two_handed: true,  ammo: 10,  ammo_kind: AmmoKind::Fuel,      damage: Damage::new( 0,  0,  3, 0), range: 10, rarity: 2 })
    }
    pub fn rocket_launcher() -> Self {
        Item::make_firearm(FirearmDef { name: "Rocket launcher",      glyph: 'R', fire_mode: FireMode::Rocket,         two_handed: true,  ammo: 1,   ammo_kind: AmmoKind::Rockets,   damage: Damage::new(500, 0,  0, 0), range: 15, rarity: 3 })
    }
    pub fn multi_rocket_launcher() -> Self {
        Item::make_firearm(FirearmDef { name: "Multi-rocket launcher", glyph: 'M', fire_mode: FireMode::Rocket,        two_handed: true,  ammo: 4,   ammo_kind: AmmoKind::Rockets,   damage: Damage::new(100, 0,  0, 0), range: 12, rarity: 3 })
    }

    // ---- Ammunition -------------------------------------------------------

    pub fn ammo_bullets() -> Self {
        Item::make_ammo(AmmoKind::Bullets,   "Bullets",   rltk::RGB::from_f32(0.85, 0.70, 0.20), 30)
    }
    pub fn ammo_rockets() -> Self {
        Item::make_ammo(AmmoKind::Rockets,   "Rockets",   rltk::RGB::from_f32(0.80, 0.20, 0.10),  3)
    }
    pub fn ammo_batteries() -> Self {
        Item::make_ammo(AmmoKind::Batteries, "Batteries", rltk::RGB::from_f32(0.20, 0.80, 0.90), 12)
    }
    pub fn ammo_fuel() -> Self {
        Item::make_ammo(AmmoKind::Fuel,      "Fuel",      rltk::RGB::from_f32(0.90, 0.50, 0.10), 5)
    }

    // ---- Healing ----------------------------------------------------------

    /// Regenerates one chosen body part for 10 turns.
    pub fn medkit() -> Self {
        Item::make_healing("Medkit", '+', rltk::RGB::from_f32(0.9, 0.2, 0.2), 10, false, 0)
    }
    /// Regenerates one chosen body part for 20 turns (double a Medkit).
    pub fn large_medkit() -> Self {
        Item::make_healing("Large medkit", '+', rltk::RGB::from_f32(1.0, 0.3, 0.3), 20, false, 1)
    }
    /// Regenerates every body part for 10 turns.
    pub fn elixir() -> Self {
        Item::make_healing("Elixir", '!', rltk::RGB::from_f32(0.8, 0.2, 0.9), 10, true, 2)
    }

    /// Instantly restores 50 energy.
    pub fn stimpack() -> Self {
        let rarity = 1;
        Item {
            id: 0, rarity,
            renderable: Renderable::new_colored_char('!', rltk::RGB::from_f32(0.9, 0.85, 0.2)),
            name: String::from("Stimpack"),
            inventory_actions: vec![Item::stim_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::Stimpack { energy: 50 },
            proxy: false,
            locked: false,
            active: false,
        }
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
            locked: false,
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
            locked: false,
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
            locked: false,
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
            locked: false,
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
            locked: false,
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
            locked: false,
            active: false,
        }
    }

    // Armor::new(phys_abs, phys_res, elec_abs, elec_res, fire_abs, fire_res)

    pub fn helmet() -> Self {
        Item::make_wearable("Helmet", '^', vec![SlotType::Headwear],
            Armor::new(2, 0.15, 0, 0.0, 0, 0.0), 1)
    }
    pub fn heavy_helmet() -> Self {
        Item::make_wearable("Heavy helmet", '^', vec![SlotType::Headwear],
            Armor::new(4, 0.25, 1, 0.10, 1, 0.10), 2)
    }
    pub fn riot_armor() -> Self {
        Item::make_wearable("Riot armor", ']',
            vec![SlotType::Bodywear, SlotType::LeftArmwear, SlotType::RightArmwear],
            Armor::new(3, 0.25, 1, 0.10, 0, 0.05), 2)
    }
    pub fn riot_pants() -> Self {
        Item::make_wearable("Riot pants", ']', vec![SlotType::Legwear],
            Armor::new(2, 0.20, 1, 0.10, 0, 0.05), 1)
    }
    pub fn heavy_combat_suit() -> Self {
        Item::make_wearable("Heavy combat suit", ']',
            vec![SlotType::Bodywear, SlotType::LeftArmwear, SlotType::RightArmwear, SlotType::Legwear, SlotType::Footwear],
            Armor::new(5, 0.35, 2, 0.20, 2, 0.20), 3)
    }
    pub fn light_kevlar_pants() -> Self {
        Item::make_wearable("Light kevlar pants", ']', vec![SlotType::Legwear],
            Armor::new(1, 0.15, 0, 0.0, 0, 0.0), 0)
    }

    /// Active headwear: light head protection plus the Recon action (a long-range
    /// directional vision cone that replaces normal sight while stationary).
    pub fn tactical_helmet() -> Self {
        let rarity = 2;
        Item {
            id: 0, rarity,
            renderable: Renderable::new_colored_char('^', Item::rarity_to_color(rarity)),
            name: String::from("Tactical helmet"),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions: vec![Item::recon_action()],
            equip_slots: vec![SlotType::Headwear],
            kind: ItemKind::Wearable { armor: Armor::new(2, 0.15, 0, 0.0, 0, 0.0) },
            proxy: false,
            locked: false,
            active: false,
        }
    }

    /// Active footwear: grants the Rocket Rush action (a loud 8-tile teleport),
    /// powered by 3 fuel charges.
    pub fn rocket_boots() -> Self {
        let rarity = 2;
        Item {
            id: 0, rarity,
            renderable: Renderable::new_colored_char('b', Item::rarity_to_color(rarity)),
            name: String::from("Rocket boots"),
            inventory_actions: vec![Item::equip_action(), Item::reload_action(), Item::drop_action()],
            equip_actions: vec![Item::rocket_rush_action(), Item::reload_action()],
            equip_slots: vec![SlotType::Footwear],
            kind: ItemKind::Powered { charges: 3, max_charges: 3, ammo_kind: AmmoKind::Fuel },
            proxy: false,
            locked: false,
            active: false,
        }
    }

    /// Active torso gear (worn instead of body armor, no protection): grants the
    /// Rocket Jump action (teleport to any revealed Ground/Road tile), 3 fuel charges.
    pub fn jetpack() -> Self {
        let rarity = 3;
        Item {
            id: 0, rarity,
            renderable: Renderable::new_colored_char('J', Item::rarity_to_color(rarity)),
            name: String::from("Jetpack"),
            inventory_actions: vec![Item::equip_action(), Item::reload_action(), Item::drop_action()],
            equip_actions: vec![Item::rocket_jump_action(), Item::reload_action()],
            equip_slots: vec![SlotType::Bodywear],
            kind: ItemKind::Powered { charges: 1, max_charges: 1, ammo_kind: AmmoKind::Fuel },
            proxy: false,
            locked: false,
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
            kind: ItemKind::Firearm { ammo: ammo, max_ammo: ammo, ammo_kind: AmmoKind::Rockets, damage: Damage::new(500, 0, 0, 0), range: range },
            proxy: false,
            locked: true,
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
            kind: ItemKind::Corpse,
            proxy: false,
            locked: false,
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
            kind: ItemKind::Corpse,
            proxy: false,
            locked: false,
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
            locked: false,
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
            locked: false,
            active: false,
        }
    }
}

// ---- Construction helpers ---------------------------------------------------

impl Item {
    fn make_firearm(def: FirearmDef) -> Item {
        // Reload appears first so the equipped-weapon action menu lists it ahead of fire actions.
        let mut equip_actions = vec![Item::reload_action()];
        equip_actions.extend(match def.fire_mode {
            FireMode::Single         => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_action()],
            FireMode::Burst          => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_burst_action()],
            FireMode::SingleAndBurst => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_action(), Item::fire_burst_action()],
            FireMode::Rocket         => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fire_rocket_action()],
            FireMode::Fan            => vec![Item::aim_action(def.range), Item::aim_at_entity_action(def.range), Item::fan_fire_action()],
        });
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
            inventory_actions: vec![Item::equip_action(), Item::reload_action(), Item::drop_action()],
            equip_actions,
            equip_slots,
            kind: ItemKind::Firearm { ammo: def.ammo, max_ammo: def.ammo, ammo_kind: def.ammo_kind, damage: def.damage, range: def.range },
            proxy: false,
            locked: false,
            active: false,
        }
    }

    fn make_ammo(kind: AmmoKind, name: &str, color: rltk::RGB, charges: u32) -> Item {
        Item {
            id: 0,
            rarity: 0,
            renderable: Renderable::new_colored_char('=', color),
            name: name.to_string(),
            inventory_actions: vec![Item::reload_from_ammo_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::Ammo { kind, charges },
            proxy: false,
            locked: false,
            active: false,
        }
    }

    /// Build a wearable armor piece covering one or more slots. Multi-slot pieces
    /// (e.g. riot armor) are placed via the proxy mechanism at equip time; `update_armor`
    /// resolves those proxies so every covered part is protected.
    fn make_wearable(name: &str, glyph: char, slots: Vec<SlotType>, armor: Armor, rarity: u8) -> Item {
        Item {
            id: 0,
            rarity,
            renderable: Renderable::new_colored_char(glyph, Item::rarity_to_color(rarity)),
            name: name.to_string(),
            inventory_actions: vec![Item::equip_action(), Item::drop_action()],
            equip_actions: vec![],
            equip_slots: slots,
            kind: ItemKind::Wearable { armor },
            proxy: false,
            locked: false,
            active: false,
        }
    }

    /// Build a healing consumable. `all_parts` picks the targeting: `false` prompts for
    /// one body part (medkits), `true` regenerates the whole body at once (elixir).
    fn make_healing(name: &str, glyph: char, color: rltk::RGB, turns: u32, all_parts: bool, rarity: u8) -> Item {
        let use_action = if all_parts { Item::heal_all_action() } else { Item::heal_part_action() };
        Item {
            id: 0,
            rarity,
            renderable: Renderable::new_colored_char(glyph, color),
            name: name.to_string(),
            inventory_actions: vec![use_action, Item::drop_action()],
            equip_actions: vec![],
            equip_slots: vec![],
            kind: ItemKind::Healing { turns },
            proxy: false,
            locked: false,
            active: false,
        }
    }

    fn equip_action() -> EntityAction {
        EntityAction { id: ActionId::Equip,         name: "Equip".to_string(),            targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::equip_item_action    }
    }
    fn drop_action() -> EntityAction {
        EntityAction { id: ActionId::Drop,          name: "Drop".to_string(),             targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::drop_item_action     }
    }
    fn aim_action(range: u32) -> EntityAction {
        EntityAction { id: ActionId::AimAtPosition, name: "Aim at position".to_string(),  targeting: Targeting::Positional { max_range: Some(range) }, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::aim_action           }
    }
    fn fire_action() -> EntityAction {
        EntityAction { id: ActionId::FireShot,      name: "Fire shot".to_string(),        targeting: Targeting::UseExistingAim { ask_bodypart: true  }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::single_fire_action   }
    }
    fn fire_burst_action() -> EntityAction {
        EntityAction { id: ActionId::FireBurst,     name: "Fire burst".to_string(),       targeting: Targeting::UseExistingAim { ask_bodypart: true  }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::burst_fire_action    }
    }
    fn fire_rocket_action() -> EntityAction {
        EntityAction { id: ActionId::FireRocket,    name: "Fire rocket".to_string(),      targeting: Targeting::UseExistingAim { ask_bodypart: false }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::rocket_fire_action   }
    }
    fn fan_fire_action() -> EntityAction {
        EntityAction { id: ActionId::FanFire,       name: "Fan fire".to_string(),         targeting: Targeting::UseExistingAim { ask_bodypart: false }, phase: ExecutionPhase::Attack, precondition: precondition_is_aiming, action: actions::fan_fire_action      }
    }
    fn aim_at_entity_action(range: u32) -> EntityAction {
        EntityAction { id: ActionId::AimAtEntity,   name: "Aim at entity".to_string(),    targeting: Targeting::EntityAim { max_range: Some(range) },   phase: ExecutionPhase::Attack, precondition: precondition_ok,        action: actions::aim_action           }
    }
    fn prime_action() -> EntityAction {
        EntityAction { id: ActionId::Prime,         name: "Prime".to_string(),            targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ok,        action: actions::prime_grenade_action }
    }
    /// Reload initiated from the firearm itself (equipped or in inventory).
    fn reload_action() -> EntityAction {
        EntityAction { id: ActionId::Reload,        name: "Reload".to_string(),           targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_can_reload, action: actions::reload_weapon_action }
    }
    /// Reload initiated from an ammo box, targeting a matching firearm.
    fn reload_from_ammo_action() -> EntityAction {
        EntityAction { id: ActionId::Reload,        name: "Reload weapon".to_string(),    targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_ammo_has_target, action: actions::reload_from_ammo_action }
    }
    /// Use a healing item on a single chosen body part.
    fn heal_part_action() -> EntityAction {
        EntityAction { id: ActionId::Heal,          name: "Use".to_string(),              targeting: Targeting::SelfBodypart, phase: ExecutionPhase::Inventory, precondition: precondition_can_heal, action: actions::use_healing_item_action }
    }
    /// Use a healing item on the whole body at once.
    fn heal_all_action() -> EntityAction {
        EntityAction { id: ActionId::Heal,          name: "Use".to_string(),              targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_can_heal, action: actions::use_healing_item_action }
    }
    /// Use a stimpack to restore energy.
    fn stim_action() -> EntityAction {
        EntityAction { id: ActionId::Stim,          name: "Use".to_string(),              targeting: Targeting::None,       phase: ExecutionPhase::Inventory, precondition: precondition_can_stim, action: actions::use_stimpack_action }
    }
    /// Rocket boots: a loud instant teleport onto a visible tile up to 8 away.
    fn rocket_rush_action() -> EntityAction {
        EntityAction { id: ActionId::RocketRush,    name: "Rocket Rush".to_string(),      targeting: Targeting::Positional { max_range: Some(8) }, phase: ExecutionPhase::Movement, precondition: precondition_has_charge, action: actions::rocket_boots_action }
    }
    /// Jetpack: a loud instant jump to any revealed Ground/Road tile.
    fn rocket_jump_action() -> EntityAction {
        EntityAction { id: ActionId::RocketJump,    name: "Rocket Jump".to_string(),      targeting: Targeting::JumpTile, phase: ExecutionPhase::Movement, precondition: precondition_can_rocket_jump, action: actions::rocket_jump_action }
    }
    /// Tactical helmet: aim a long-range recon vision cone at a visible tile.
    fn recon_action() -> EntityAction {
        EntityAction { id: ActionId::Recon,         name: "Recon".to_string(),            targeting: Targeting::Positional { max_range: None }, phase: ExecutionPhase::Instant, precondition: precondition_ok, action: actions::recon_action }
    }
    fn throw_action() -> EntityAction {
        EntityAction { id: ActionId::Throw,         name: "Throw".to_string(),            targeting: Targeting::Positional { max_range: Some(5) }, phase: ExecutionPhase::Attack,    precondition: precondition_ok,        action: actions::throw_grenade_action }
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

/// True if `item` reloads from `kind` and is below capacity (a valid reload target).
/// Covers both firearms and powered gear.
fn is_reloadable(item: &Item, kind: AmmoKind) -> bool {
    matches!(item.kind.reloadable(), Some((cur, max, k)) if k == kind && cur < max)
}

/// True if the entity carries at least one ammo box of `kind` with charges left.
fn has_ammo_of_kind(entity: &Entity, kind: AmmoKind) -> bool {
    entity.body.inventory.iter().any(|i| matches!(&i.kind, ItemKind::Ammo { kind: k, charges } if *k == kind && *charges > 0))
}

/// Reload precondition for a reloadable item (firearm or powered gear): it must be below
/// capacity and the entity must carry matching ammo. `item` is the item the action belongs to.
pub fn precondition_can_reload(self_ref: &Entity, _map: &Map, item: Option<&Item>) -> bool {
    match item.and_then(|i| i.kind.reloadable()) {
        Some((cur, max, kind)) => cur < max && has_ammo_of_kind(self_ref, kind),
        None => false,
    }
}

/// Reload precondition for an ammo box: some carried or equipped item of the box's
/// kind must be below capacity. `item` is the ammo box the action belongs to.
pub fn precondition_ammo_has_target(self_ref: &Entity, _map: &Map, item: Option<&Item>) -> bool {
    let kind = match item {
        Some(i) => match &i.kind {
            ItemKind::Ammo { kind, charges } if *charges > 0 => *kind,
            _ => return false,
        },
        None => return false,
    };
    find_reloadable_weapon_id(self_ref, kind).is_some()
}

/// Pick a reloadable item for an ammo box of `kind`: prefer equipped, then inventory.
/// Returns its item id.
pub fn find_reloadable_weapon_id(entity: &Entity, kind: AmmoKind) -> Option<usize> {
    entity.body.item_slots.iter()
        .filter_map(|s| s.item.as_ref())
        .find(|i| is_reloadable(i, kind))
        .or_else(|| entity.body.inventory.iter().find(|i| is_reloadable(i, kind)))
        .map(|i| i.id)
}

/// True if `item` is powered gear with at least one charge left.
fn has_charge(item: &Item) -> bool {
    matches!(&item.kind, ItemKind::Powered { charges, .. } if *charges > 0)
}

/// Precondition for a powered-gear action: the item must have a charge to spend.
pub fn precondition_has_charge(_self_ref: &Entity, _map: &Map, item: Option<&Item>) -> bool {
    item.map_or(false, has_charge)
}

/// Rocket Jump precondition: a charge is available AND the wearer stands on Ground/Road.
pub fn precondition_can_rocket_jump(self_ref: &Entity, map: &Map, item: Option<&Item>) -> bool {
    if !item.map_or(false, has_charge) { return false; }
    matches!(map.tiles[map.pos_idx(self_ref.position)], crate::TileType::Ground | crate::TileType::Road)
}

/// Healing items are usable only when at least one body part is damaged.
pub fn precondition_can_heal(self_ref: &Entity, _map: &Map, _item: Option<&Item>) -> bool {
    self_ref.body.parts.iter().any(|p| p.damage > 0)
}

/// Stimpacks are usable only when energy is below the maximum.
pub fn precondition_can_stim(self_ref: &Entity, _map: &Map, _item: Option<&Item>) -> bool {
    self_ref.body.energy < self_ref.body.max_energy
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
