use rltk::Point;
use std::hash::{Hash, Hasher};
use crate::Animation;

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

pub enum Effect {
    Damage {entity_id: usize, bodypart_index: usize, raw_damage: Damage},
    OpenDoor(Point),
    DestroyWall(Point),
    Animation(Animation),
    Embark{pilot_id: usize, vehicle_id: usize},
    Disembark{pilot_id: usize, vehicle_id: usize},
    ApplyStatus{target_id: usize, status: StatusEffect}
}

// Status Effects are considered Eq if they have the same enum type, even if the value is
// different. This allows for storage of a particular kind of status effect in a hashmap with
// pretty quick lookup without a bunch of extra logic to avoid duplicates.
// Since the enum is a closed set, we simply make the hash an increasing integer via the to_index
// function. Equivalence testing uses the same function, ensuring coherence between Hash and Eq.
#[derive(Clone)]
pub enum StatusEffect {
    AimingAtGround(Point)
}

impl StatusEffect {
    fn to_index(&self) -> usize {
        match self {
            StatusEffect::AimingAtGround(_) => 0
        }
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

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum SlotType {
    PrimaryHand,
    SecondaryHand,
    Headwear,
    Legwear,
    Footwear,
    Bodywear,
    LeftArmwear,
    RightArmwear
}

#[derive(Clone)]
pub enum ItemKind {
    Firearm {ammo: u32, max_ammo: u32, damage: Damage},
    Wearable {armor: Armor},
    Misc
}

#[derive(Clone, Copy, PartialEq)]
pub struct Damage {
    pub physical: u32,
    pub fire: u32,
    pub electrical: u32
}

impl Damage {
    pub fn new(phys: u32, fire: u32, elec: u32) -> Self {
        Self {
            physical: phys,
            fire: fire,
            electrical: elec
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
    pub fn new(phys_abs: u32, phys_res: f32, fire_abs: u32, fire_res: f32, elec_abs: u32, elec_res: f32) -> Self {
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

        return physical + fire + electrical;
    }
}