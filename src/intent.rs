use rltk::Point;
use crate::Item;
use crate::Entity;
use crate::Map;
use crate::components::*;
use crate::GameLog;
use crate::actions::Action;

#[derive (Clone)]
pub struct Intent {
    pub phase: ExecutionPhase,
    pub data: IntentData,
    pub action: Action
}

#[derive(Copy, Clone, PartialEq)]
pub enum Targeting {
    None,
    Positional,
    Detailed,
    /// Fire at the player's current aim position (set by a prior `aim_action`).
    /// If `ask_bodypart` is true and an entity occupies the aimed tile, opens
    /// the bodypart menu before resolving (used by single/burst fire).
    /// If false, fires directly at the position (used by rockets, fan fire).
    UseExistingAim { ask_bodypart: bool },
}

#[derive(Clone)]
pub struct ItemAction {
    pub name: String,
    pub targeting: Targeting,
    pub phase: ExecutionPhase,
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
        phase: ExecutionPhase::Idle,
        data: IntentData::Void,
        action: noop_action
    }
}

#[derive (PartialEq, Eq, Copy, Clone)]
pub enum ExecutionPhase {
    Idle,
    Instant,
    Inventory,
    Attack,
    Movement,
    Misc
}

impl ExecutionPhase {
    pub fn next(&self) -> Option<ExecutionPhase> {
        match self {
            ExecutionPhase::Idle => Some(ExecutionPhase::Instant),
            ExecutionPhase::Instant => Some(ExecutionPhase::Inventory),
            ExecutionPhase::Inventory => Some(ExecutionPhase::Attack),
            ExecutionPhase::Attack => Some(ExecutionPhase::Movement),
            ExecutionPhase::Movement => Some(ExecutionPhase::Misc),
            ExecutionPhase::Misc => None
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
