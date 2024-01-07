use rltk::Point;
use crate::Item;
use crate::Entity;
use crate::Map;
use crate::components::*;
use crate::GameLog;
use crate::actions::Action;

#[derive (Clone)]
pub struct Intent {
    pub phase: IntentPhase,
    pub data: IntentData,
    pub action: Action
}

#[derive(Copy, Clone, PartialEq)]
pub enum Targeting {
    None,
    Positional,
    Detailed
}

#[derive(Clone)]
pub struct IntentAction {
    pub name: String,
    pub targeting: Targeting,
    pub phase: IntentPhase,
    pub precondition: fn (self_ref: &Entity, map: &Map, affected_item: Option<&Item>) -> bool,
    pub action: Action
}

fn noop_action(_entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
    vec!()
}

pub fn precondition_ok(_self_ref: &Entity, _map: &Map, _affected_item: Option<&Item>) -> bool {
    true
}

pub fn idle_intent() -> Intent {
    Intent {
        phase: IntentPhase::Idle,
        data: IntentData::Void,
        action: noop_action
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

impl IntentPhase {
    pub fn next(&self) -> Option<IntentPhase> {
        match self {
            IntentPhase::Idle => Some(IntentPhase::Instant),
            IntentPhase::Instant => Some(IntentPhase::Inventory),
            IntentPhase::Inventory => Some(IntentPhase::Attack),
            IntentPhase::Attack => Some(IntentPhase::Movement),
            IntentPhase::Movement => Some(IntentPhase::Misc),
            IntentPhase::Misc => None
        }
    }
}

#[derive (Clone)]
pub enum IntentData {
    Void,
    InventoryItem(Item),
    EquippedItem(SlotType),
    Target(Point),
    Direction(Direction),
    TargetWithEquipment{slot: SlotType, target: Point},
    TargetWithInventory{item: Item, target: Point},
    TargetBodypartWithEquipment{slot: SlotType, target: Point, bodypart_index: usize},
    TargetBodypartWithInventory{item: Item, target: Point, bodypart_index: usize}
}
