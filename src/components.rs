use rltk::Point;
use std::hash::{Hash, Hasher};
use crate::{Animation, Item};

#[derive (PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft
}

impl Direction {
    pub fn counter_clockwise(&self) -> Direction {
        match self {
            Direction::Up => Direction::UpLeft,
            Direction::UpRight => Direction::Up,
            Direction::Right => Direction::UpRight,
            Direction::DownRight => Direction::Right,
            Direction::Down => Direction::DownRight,
            Direction::DownLeft => Direction::Down,
            Direction::Left => Direction::DownLeft,
            Direction::UpLeft => Direction::Left,
        }
    }

    pub fn clockwise(&self) -> Direction {
        match self {
            Direction::Up => Direction::UpRight,
            Direction::UpRight => Direction::Right,
            Direction::Right => Direction::DownRight,
            Direction::DownRight => Direction::Down,
            Direction::Down => Direction::DownLeft,
            Direction::DownLeft => Direction::Left,
            Direction::Left => Direction::UpLeft,
            Direction::UpLeft => Direction::Up,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Direction::Up        => "north",
            Direction::UpRight   => "northeast",
            Direction::Right     => "east",
            Direction::DownRight => "southeast",
            Direction::Down      => "south",
            Direction::DownLeft  => "southwest",
            Direction::Left      => "west",
            Direction::UpLeft    => "northwest",
        }
    }

    pub fn delta_pos(&self) -> (i32, i32) {
        match self {
            Direction::Up        => ( 0, -1),
            Direction::UpRight   => ( 1, -1),
            Direction::Right     => ( 1,  0),
            Direction::DownRight => ( 1,  1),
            Direction::Down      => ( 0,  1),
            Direction::DownLeft  => (-1,  1),
            Direction::Left      => (-1,  0),
            Direction::UpLeft    => (-1, -1),
        }
    }
}

#[derive (Copy, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

impl Renderable {
    pub fn new_glyph(character: rltk::FontCharType) -> Self {
        Self {
            glyph: character,
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }

    pub fn new_char(character: char) -> Self {
        Self {
            glyph: rltk::to_cp437(character),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }
}

#[derive(Clone)]
pub enum ItemLocation {
    OnMap(Point),
    InInventory(usize),
}

pub enum Effect {
    Damage {entity_id: usize, bodypart_index: usize, raw_damage: Damage},
    OpenDoor(Point),
    DestroyWall(Point),
    Animation(Animation),
    Embark{pilot_id: usize, vehicle_id: usize},
    Disembark{pilot_id: usize, vehicle_id: usize},
    ApplyStatus{target_id: usize, status: StatusEffect},
    BurnTick{entity_id: usize, bodypart_index: usize},
    SyncActiveItem{item_id: usize, location: ItemLocation},
}

// Status Effects are considered Eq if they have the same enum type, even if the value is
// different. This allows for storage of a particular kind of status effect in a hashmap with
// pretty quick lookup without a bunch of extra logic to avoid duplicates.
// Since the enum is a closed set, we simply make the hash an increasing integer via the to_index
// function. Equivalence testing uses the same function, ensuring coherence between Hash and Eq.
#[derive(Clone)]
pub enum StatusEffect {
    AimingAtGround(Point, Item),
    AimingAtEntity(usize, Item),
    Blind(u32),
    Burning(u32),
    Shocked(u32),
}

impl StatusEffect {
    fn to_index(&self) -> usize {
        match self {
            StatusEffect::AimingAtGround(_, _) => 0,
            StatusEffect::AimingAtEntity(_, _) => 0,
            StatusEffect::Blind(_)   => 1,
            StatusEffect::Burning(_) => 2,
            StatusEffect::Shocked(_) => 3,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            StatusEffect::AimingAtGround(_, _) => "Aiming",
            StatusEffect::AimingAtEntity(_, _) => "Aiming",
            StatusEffect::Blind(_)   => "Blind",
            StatusEffect::Burning(_) => "Burning",
            StatusEffect::Shocked(_) => "Shocked",
        }.to_string()
    }
}

impl Hash for StatusEffect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.to_index());
    }
}

impl PartialEq for StatusEffect {
    fn eq(&self, other: &Self) -> bool {
        self.to_index() == other.to_index()
    }
}

impl Eq for StatusEffect {}

impl StatusEffect {
    /// Returns the remaining duration for timed effects; `None` for persistent effects.
    pub fn duration(&self) -> Option<u32> {
        match self {
            StatusEffect::AimingAtGround(_, _) | StatusEffect::AimingAtEntity(_, _) => None,
            StatusEffect::Blind(n)   => Some(*n),
            StatusEffect::Burning(n) => Some(*n),
            StatusEffect::Shocked(n) => Some(*n),
        }
    }

    /// Decrement the duration by one tick. Returns `None` when the effect expires.
    /// Aiming effects are not duration-based and are returned unchanged.
    pub fn tick(&self) -> Option<StatusEffect> {
        match self {
            StatusEffect::AimingAtGround(_, _) | StatusEffect::AimingAtEntity(_, _) => Some(self.clone()),
            StatusEffect::Blind(n)   => if *n > 1 { Some(StatusEffect::Blind(*n - 1))   } else { None },
            StatusEffect::Burning(n) => if *n > 1 { Some(StatusEffect::Burning(*n - 1)) } else { None },
            StatusEffect::Shocked(n) => if *n > 1 { Some(StatusEffect::Shocked(*n - 1)) } else { None },
        }
    }
}

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum SlotType {
    PrimaryHand,
    SecondaryHand,
    Headwear,
    Facewear,
    Legwear,
    Footwear,
    Bodywear,
    Backwear,
    LeftArmwear,
    RightArmwear
}

impl SlotType {
    pub fn to_string(&self) -> String {
        match self {
            SlotType::PrimaryHand => "In r. hand",
            SlotType::SecondaryHand => "In l. hand",
            SlotType::Headwear => "On head",
            SlotType::Facewear => "On Face",
            SlotType::Legwear => "On legs",
            SlotType::Footwear => "On feet",
            SlotType::Bodywear => "On body",
            SlotType::Backwear => "On back",
            SlotType::LeftArmwear => "On l. arm",
            SlotType::RightArmwear => "On r. arm"
        }.to_string()
    }
}

#[derive(Clone)]
pub enum ItemKind {
    Firearm {ammo: u32, max_ammo: u32, damage: Damage, range: u32},
    Wearable {armor: Armor},
    FusedExplosive {damage: Damage, timeout: u32, flash: bool},
    Misc
}

#[derive(Clone, Copy, PartialEq)]
pub struct Damage {
    pub physical: u32,
    pub fire: u32,
    pub electrical: u32,
    pub piercing: u32
}

impl Damage {
    pub fn new(phys: u32, elec: u32, fire: u32, pierce: u32) -> Self {
        Self {
            physical: phys,
            fire: fire,
            electrical: elec,
            piercing: pierce
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Armor {
    pub phys_absorption: u32,
    pub phys_resistance: f32,

    pub fire_absorption: u32,
    pub fire_resistance: f32,

    pub elec_absorption: u32,
    pub elec_resistance: f32
}

impl Armor {
    pub fn new(phys_abs: u32, phys_res: f32, elec_abs: u32, elec_res: f32, fire_abs: u32, fire_res: f32) -> Self {
        Self {
            phys_absorption: phys_abs,
            phys_resistance: phys_res,
            fire_absorption: fire_abs,
            fire_resistance: fire_res,
            elec_absorption: elec_abs,
            elec_resistance: elec_res,
        }
    }

    pub fn zero() -> Self {
        Self {
            phys_absorption: 0,
            phys_resistance: 0.0,
            fire_absorption: 0,
            fire_resistance: 0.0,
            elec_absorption: 0,
            elec_resistance: 0.0
        }
    }

    pub fn add(&self, other: &Armor) -> Armor {
        Armor {
            phys_absorption: self.phys_absorption + other.phys_absorption,
            phys_resistance: self.phys_resistance + other.phys_resistance,
            fire_absorption: self.fire_absorption + other.fire_absorption,
            fire_resistance: self.fire_resistance + other.fire_resistance,
            elec_absorption: self.elec_absorption + other.elec_absorption,
            elec_resistance: self.elec_resistance + other.elec_resistance
        }
    }

    pub fn electrical_penetrates(&self, damage: Damage) -> bool {
        if self.elec_absorption >= damage.electrical || self.elec_resistance >= 1.0 {
            return false;
        }
        ((damage.electrical - self.elec_absorption) as f32 * (1.0 - self.elec_resistance)) as u32 > 0
    }

    pub fn modify_damage(&self, damage: Damage) -> u32 {
        fn mod_dmg(dmg: u32, abs: u32, res: f32) -> u32 {
            if abs >= dmg
            || res >= 1.0 {
                return 0;
            }
            return ((dmg - abs) as f32 * (1.0 - res)) as u32;
        }

        let physical = mod_dmg(damage.physical, self.phys_absorption, self.phys_resistance);
        let fire = mod_dmg(damage.fire, self.fire_absorption, self.fire_resistance);
        let electrical = mod_dmg(damage.electrical, self.elec_absorption, self.elec_resistance);

        return physical + fire + electrical + damage.piercing;
    }
}