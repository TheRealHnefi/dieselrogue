use rltk::Point;
use crate::Item;
use crate::Entity;
use crate::Map;
use crate::components::*;
use crate::GameLog;

#[derive (Clone)]
pub struct Intent {
    pub phase: IntentPhase,
    pub data: IntentData,
    pub action: fn (self_ref: &mut Entity, map: &mut Map, log: &mut GameLog) -> Vec<Effect>
}

fn intent_noop(_entity: &mut Entity, _map: &mut Map, _log: &mut GameLog) -> Vec<Effect> {
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
    TargetWithInventory{item: Item, target: Point},
    TargetBodypartWithEquipment{slot: SlotType, target: Point, bodypart_index: usize},
    TargetBodypartWithInventory{item: Item, target: Point, bodypart_index: usize}
}
