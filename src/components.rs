use rltk::Point;
use std::hash::{Hash, Hasher};
use crate::{Animation, Item};

pub const INVENTORY_MAX: usize = 20;

pub const KEY_COLORS: [(u8, u8, u8); 16] = [
    (220,  50,  50),  // 0  Red
    (230, 140,  30),  // 1  Orange
    (220, 220,  50),  // 2  Yellow
    (130, 220,  50),  // 3  Lime
    ( 50, 180,  50),  // 4  Green
    ( 50, 220, 180),  // 5  Teal
    ( 50, 180, 230),  // 6  Sky
    ( 60,  60, 220),  // 7  Blue
    (150,  60, 220),  // 8  Purple
    (220,  60, 220),  // 9  Magenta
    (230, 120, 180),  // 10 Pink
    (150,  90,  30),  // 11 Brown
    (240, 240, 240),  // 12 White
    (180, 180, 180),  // 13 Silver
    (110, 110, 110),  // 14 Gray
    (230, 190,  40),  // 15 Gold
];

pub const KEY_COLOR_NAMES: [&str; 16] = [
    "Red", "Orange", "Yellow", "Lime", "Green", "Teal", "Sky", "Blue",
    "Purple", "Magenta", "Pink", "Brown", "White", "Silver", "Gray", "Gold",
];

/// Which paper-doll image to show when inspecting an entity.
/// Add a variant here whenever a new sprite sheet is added to RexAssets.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaperDoll {
    Player,
    MaleSilhouette,
    Tank,
}

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

    pub const ALL: [Direction; 8] = [
        Direction::Up, Direction::UpRight, Direction::Right, Direction::DownRight,
        Direction::Down, Direction::DownLeft, Direction::Left, Direction::UpLeft,
    ];

    /// The compass direction whose heading best matches the vector `(dx, dy)`.
    /// Returns `None` for the zero vector.
    pub fn nearest(dx: i32, dy: i32) -> Option<Direction> {
        if dx == 0 && dy == 0 { return None; }
        let len = ((dx * dx + dy * dy) as f32).sqrt();
        let (nx, ny) = (dx as f32 / len, dy as f32 / len);
        let mut best = Direction::Up;
        let mut best_dot = f32::MIN;
        for d in Direction::ALL {
            let (ex, ey) = d.delta_pos();
            let el = ((ex * ex + ey * ey) as f32).sqrt();
            let dot = (nx * ex as f32 + ny * ey as f32) / el;
            if dot > best_dot { best_dot = dot; best = d; }
        }
        Some(best)
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

    pub fn new_colored_char(character: char, color: rltk::RGB) -> Self {
        Self {
            glyph: rltk::to_cp437(character),
            color: color,
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
    /// Apply damage to entity
    Damage      { entity_id: usize, bodypart_index: usize, raw_damage: Damage },
    /// Open door at position
    OpenDoor    { pos: Point, actor_id: usize },
    /// Remove wall
    DestroyWall(Point),
    /// Run animation
    Animation(Animation),
    /// Enter vehicle
    Embark      { pilot_id: usize, vehicle_id: usize },
    /// Hop out of vehicle
    Disembark   { pilot_id: usize, vehicle_id: usize },
    /// Apply status effect
    ApplyStatus { target_id: usize, status: StatusEffect },
    /// Handle burning status effect
    BurnTick    { entity_id: usize, bodypart_index: usize },
    /// Handle active items such as ticking grenades
    SyncActiveItem { item_id: usize, location: ItemLocation },
    /// Fire sound
    Sound(SoundEvent),
    /// Turn target entity in given direction
    Twist       { entity_id: usize, direction: Direction },
    /// Remove the aim of target entity
    Distract    { entity_id: usize },
    /// Move entity to pos (updates pawns + clears aiming).
    Move        { entity_id: usize, pos: Point },
    /// Change entity facing and refresh pawns (clears aiming).
    SetFacing   { entity_id: usize, direction: Direction },
    /// Decrement ammo in the weapon held in `slot` by `shots`.
    ConsumeAmmo { entity_id: usize, slot: SlotType, shots: u32 },
    /// Refill the firearm with the given id (equipped or in inventory) up to capacity,
    /// draining matching ammo boxes from the entity's inventory.
    ReloadWeapon { entity_id: usize, weapon_id: usize },
    /// Apply Regenerating for `turns` turns. `bodypart_index` = `Some(i)` targets one
    /// part; `None` targets every part. Merges with any existing regeneration.
    ApplyRegeneration { entity_id: usize, bodypart_index: Option<usize>, turns: u32 },
    /// Set (or replace) the recon Scanning status aimed at `target`.
    ApplyScan { entity_id: usize, target: Point },
    /// Heal one HP on the given body part (drives the Regenerating tick).
    RegenTick { entity_id: usize, bodypart_index: usize },
    /// Remove and discard the inventory item with the given id (consumables).
    ConsumeItem { entity_id: usize, item_id: usize },
    /// Spend one charge from the powered item with the given id (equipped or carried).
    ConsumeCharge { entity_id: usize, item_id: usize },
    /// Subtract energy from entity (ability cost).
    SpendEnergy { entity_id: usize, amount: u32 },
    /// Restore energy to entity, clamped to its maximum (stimpack).
    RestoreEnergy { entity_id: usize, amount: u32 },
    /// Pick up the item at entity's current map tile.
    PickUpItem  { entity_id: usize },
    /// Drop the inventory item with the given id to the nearest free tile.
    DropItem    { entity_id: usize, item_id: usize },
    /// Take the inventory item with the given id and place it near `target_pos`.
    ThrowItem   { entity_id: usize, item_id: usize, target_pos: Point },
    /// Activate (prime) the explosive item with the given id in entity's inventory.
    PrimeItem   { entity_id: usize, item_id: usize },
    /// Equip the inventory item with the given id.
    EquipItem   { entity_id: usize, item_id: usize },
    /// Unequip the item with the given id back to inventory.
    UnequipItem { entity_id: usize, item_id: usize },
    /// Append a message to the game log.
    Log(String),
}

#[derive(Clone)]
pub enum SoundKind {
    Gunshot,
    Burst,
    Explosion,
    Footstep,
    Engine,
    Shout,
}

#[derive(Clone)]
pub struct SoundEvent {
    pub kind: SoundKind,
    pub pos: Point,
    pub volume: u32,
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
    Deaf(u32),
    IronBody(u32),
    /// Heals one HP per turn on each body part with turns remaining.
    /// Indexed by bodypart index; each entry is that part's remaining regen turns.
    Regenerating(Vec<u32>),
    /// Recon vision: a long-range cone aimed at the stored tile replaces normal sight.
    /// Persists until the wearer moves or turns.
    Scanning(Point),
}

impl StatusEffect {
    fn to_index(&self) -> usize {
        match self {
            StatusEffect::AimingAtGround(_, _) => 0,
            StatusEffect::AimingAtEntity(_, _) => 0,
            StatusEffect::Blind(_)   => 1,
            StatusEffect::Burning(_) => 2,
            StatusEffect::Shocked(_) => 3,
            StatusEffect::Deaf(_)    => 4,
            StatusEffect::IronBody(_) => 5,
            StatusEffect::Regenerating(_) => 6,
            StatusEffect::Scanning(_) => 7,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            StatusEffect::AimingAtGround(_, _) => "Aiming",
            StatusEffect::AimingAtEntity(_, _) => "Aiming",
            StatusEffect::Blind(_)   => "Blind",
            StatusEffect::Burning(_) => "Burning",
            StatusEffect::Shocked(_) => "Shocked",
            StatusEffect::Deaf(_)    => "Deaf",
            StatusEffect::IronBody(_) => "Iron Body",
            StatusEffect::Regenerating(_) => "Regenerating",
            StatusEffect::Scanning(_) => "Recon",
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
            StatusEffect::Deaf(n)     => Some(*n),
            StatusEffect::IronBody(n) => Some(*n),
            // Show the longest-running part's countdown.
            StatusEffect::Regenerating(v) => Some(v.iter().copied().max().unwrap_or(0)),
            StatusEffect::Scanning(_) => None,
        }
    }

    /// Decrement the duration by one tick. Returns `None` when the effect expires.
    /// Aiming effects are not duration-based and are returned unchanged.
    pub fn tick(&self) -> Option<StatusEffect> {
        match self {
            StatusEffect::AimingAtGround(_, _) | StatusEffect::AimingAtEntity(_, _) => Some(self.clone()),
            StatusEffect::Blind(n)    => if *n > 1 { Some(StatusEffect::Blind(*n - 1))    } else { None },
            StatusEffect::Burning(n)  => if *n > 1 { Some(StatusEffect::Burning(*n - 1))  } else { None },
            StatusEffect::Shocked(n)  => if *n > 1 { Some(StatusEffect::Shocked(*n - 1))  } else { None },
            StatusEffect::Deaf(n)     => if *n > 1 { Some(StatusEffect::Deaf(*n - 1))     } else { None },
            StatusEffect::IronBody(n) => if *n > 1 { Some(StatusEffect::IronBody(*n - 1)) } else { None },
            StatusEffect::Regenerating(v) => {
                let next: Vec<u32> = v.iter().map(|n| n.saturating_sub(1)).collect();
                if next.iter().any(|&n| n > 0) { Some(StatusEffect::Regenerating(next)) } else { None }
            },
            // Persistent; cleared explicitly on move/turn, not by ticking.
            StatusEffect::Scanning(_) => Some(self.clone()),
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
    RightArmwear,
    TurretMount
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
            SlotType::RightArmwear => "On r. arm",
            SlotType::TurretMount => "Mounted"
        }.to_string()
    }
}

/// Category of ammunition. A firearm consumes exactly one kind; an ammo box
/// supplies exactly one kind.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AmmoKind {
    Bullets,
    Rockets,
    Batteries,
    Fuel,
}

impl AmmoKind {
    pub fn name(&self) -> &'static str {
        match self {
            AmmoKind::Bullets   => "bullets",
            AmmoKind::Rockets   => "rockets",
            AmmoKind::Batteries => "batteries",
            AmmoKind::Fuel      => "fuel",
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ItemKind {
    Firearm {ammo: u32, max_ammo: u32, ammo_kind: AmmoKind, damage: Damage, range: u32},
    MeleeWeapon {damage: Damage},
    Wearable {armor: Armor},
    FusedExplosive {damage: Damage, timeout: u32, radius: u32, flash: bool},
    Key {color: usize},
    /// A box of ammunition holding `charges` rounds of `kind`, used to reload firearms.
    Ammo {kind: AmmoKind, charges: u32},
    /// A consumable that applies Regenerating for `turns` turns (to one body part
    /// or all, depending on the item's targeting).
    Healing {turns: u32},
    /// A consumable that instantly restores `energy` points (clamped to the max).
    Stimpack {energy: u32},
    /// Active gear (jetpack, rocket boots) powered by rechargeable charges,
    /// reloaded from ammo boxes of `ammo_kind`.
    Powered {charges: u32, max_charges: u32, ammo_kind: AmmoKind},
    Corpse,
    Misc
}

impl ItemKind {
    /// For kinds that hold a rechargeable count, returns `(current, max, ammo_kind)`.
    /// Shared by the reload system so firearms and powered gear reload the same way.
    pub fn reloadable(&self) -> Option<(u32, u32, AmmoKind)> {
        match self {
            ItemKind::Firearm { ammo, max_ammo, ammo_kind, .. } => Some((*ammo, *max_ammo, *ammo_kind)),
            ItemKind::Powered { charges, max_charges, ammo_kind } => Some((*charges, *max_charges, *ammo_kind)),
            _ => None,
        }
    }

    /// Adds `amount` to the current count, clamped to the maximum. No-op for other kinds.
    pub fn add_charges(&mut self, amount: u32) {
        match self {
            ItemKind::Firearm { ammo, max_ammo, .. } => *ammo = (*ammo + amount).min(*max_ammo),
            ItemKind::Powered { charges, max_charges, .. } => *charges = (*charges + amount).min(*max_charges),
            _ => {},
        }
    }

    /// Spends one charge from powered gear if available; returns whether one was spent.
    pub fn spend_charge(&mut self) -> bool {
        match self {
            ItemKind::Powered { charges, .. } if *charges > 0 => { *charges -= 1; true },
            _ => false,
        }
    }
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