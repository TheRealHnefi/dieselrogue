use rltk::Point;
use crate::Map;
use crate::item::Item;
use crate::entity::Entity;

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

#[derive (Copy, Clone)]
pub struct Facing {
    pub direction: Direction
}

#[derive (Copy, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

impl Renderable {
    pub fn new() -> Self {
        Self {
            glyph: rltk::to_cp437('?'),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }

    pub fn new_glyph(character: char) -> Self {
        Self {
            glyph: rltk::to_cp437(character),
            color: rltk::RGB::named(rltk::YELLOW),
            background: rltk::RGB::named(rltk::BLACK)
        }
    }
}

#[derive (Clone)]
pub struct Intent {
    pub phase: IntentPhase,
    pub data: IntentData,
    pub action: fn (self_ref: &mut Entity, map: &mut Map) -> Vec<Effect>
}

fn intent_noop(_entity: &mut Entity, _map: &mut Map) -> Vec<Effect> {
    vec!()
}

pub fn idle_intent() -> Intent {
    Intent {
        phase: IntentPhase::Idle,
        data: IntentData::Void,
        action: intent_noop
    }
}

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum IntentPhase {
    Idle,
    Instant,
    Inventory,
    Attack,
    Movement,
    Misc
}

#[derive (Clone)]
pub enum IntentData {
    Void,
    InventoryItem(Item),
    EquippedItem(SlotType),
    Target(Point),
    Direction(Direction),
    TargetWithEquipment{slot: SlotType, target: Point},
    TargetWithInventory{item: Item, target: Point}
}

pub enum Effect {
    Damage {entity_id: usize},
}

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
    Firearm {ammo: u32, max_ammo: u32},
    Misc
}
