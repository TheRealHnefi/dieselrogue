use rltk::Point;
use crate::Item;
use crate::Entity;
use crate::Map;
use crate::components::*;
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
    Positional { max_range: Option<u32> },
    Detailed,
    /// Fire at the player's current aim position (set by a prior `aim_action`).
    /// If `ask_bodypart` is true and an entity occupies the aimed tile, opens
    /// the bodypart menu before resolving (used by single/burst fire).
    /// If false, fires directly at the position (used by rockets, fan fire).
    UseExistingAim { ask_bodypart: bool },
    /// Cycle the cursor among visible entities within range; confirms as AimingAtEntity.
    EntityAim { max_range: Option<u32> },
    /// Resolve against an adjacent direction chosen by keypress.
    Direction,
}

/// Stable identity for a catalog action. Lets keybindings and the AI reference
/// an action without a fn-pointer.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ActionId {
    Shout,
    IronBody,
    Rush,
    Twist,
    Distract,
    Equip,
    Drop,
    AimAtPosition,
    AimAtEntity,
    FireShot,
    FireBurst,
    FireRocket,
    FanFire,
    Prime,
    Throw,
}

#[derive(Clone)]
pub struct EntityAction {
    // Read by keybindings/AI from stage 5 onward.
    #[allow(dead_code)]
    pub id: ActionId,
    pub name: String,
    pub targeting: Targeting,
    pub phase: ExecutionPhase,
    pub precondition: fn (self_ref: &Entity, map: &Map, affected_item: Option<&Item>) -> bool,
    pub action: Action
}

fn noop_action(_entity: &Entity, _map: &Map, _entities: &[Entity]) -> Vec<Effect> {
    vec!()
}

pub fn precondition_ok(_self_ref: &Entity, _map: &Map, _affected_item: Option<&Item>) -> bool {
    true
}

impl Intent {
    pub fn description(&self) -> String {
        match &self.data {
            IntentData::Void => match self.phase {
                ExecutionPhase::Idle => "Idle".to_string(),
                _                   => "Acting".to_string(),
            },
            IntentData::Direction(dir) => format!("Turning {}", dir.name()),
            IntentData::Target(pos) => match self.phase {
                ExecutionPhase::Attack   => format!("Attacking ({},{})", pos.x, pos.y),
                ExecutionPhase::Movement => format!("Moving to ({},{})", pos.x, pos.y),
                _                        => format!("Acting at ({},{})", pos.x, pos.y),
            },
            IntentData::EquippedItem(_)                              => "Using item".to_string(),
            IntentData::InventoryItem(item)                          => format!("Using {}", item.name),
            IntentData::TargetWithEquipment { target, .. }           => format!("Firing at ({},{})", target.x, target.y),
            IntentData::TargetWithInventory { item, target }         => format!("{} at ({},{})", item.name, target.x, target.y),
            IntentData::TargetBodypartWithEquipment { target, .. }   => format!("Firing at ({},{})", target.x, target.y),
            IntentData::TargetBodypartWithInventory { item, target, .. } => format!("{} at ({},{})", item.name, target.x, target.y),
        }
    }
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
    ActiveItems,
    Misc
}

impl ExecutionPhase {
    pub fn next(&self) -> Option<ExecutionPhase> {
        match self {
            ExecutionPhase::Idle => Some(ExecutionPhase::Instant),
            ExecutionPhase::Instant => Some(ExecutionPhase::Inventory),
            ExecutionPhase::Inventory => Some(ExecutionPhase::Attack),
            ExecutionPhase::Attack => Some(ExecutionPhase::Movement),
            ExecutionPhase::Movement => Some(ExecutionPhase::ActiveItems),
            ExecutionPhase::ActiveItems => Some(ExecutionPhase::Misc),
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

/// The item/slot context an action operates through, if any.
#[derive(Clone)]
pub enum ActionSource {
    InventoryItem(Item),
    EquippedSlot(SlotType),
}

/// A resolved target for an action, chosen by the picker (menu/hotkey/AI).
pub enum Resolution {
    None,
    Direction(Direction),
    Position(Point),
    Bodypart { target: Point, bodypart_index: usize },
}

/// The single constructor for a concrete `Intent` from a catalog action plus a
/// resolved source and target.
pub fn build_intent(action: &EntityAction, source: Option<ActionSource>, resolution: Resolution) -> Intent {
    let data = match resolution {
        Resolution::None => match source {
            None                                     => IntentData::Void,
            Some(ActionSource::EquippedSlot(slot))   => IntentData::EquippedItem(slot),
            Some(ActionSource::InventoryItem(item))  => IntentData::InventoryItem(item),
        },
        Resolution::Direction(dir) => IntentData::Direction(dir),
        Resolution::Position(target) => match source {
            None                                     => IntentData::Target(target),
            Some(ActionSource::EquippedSlot(slot))   => IntentData::TargetWithEquipment { slot, target },
            Some(ActionSource::InventoryItem(item))  => IntentData::TargetWithInventory { item, target },
        },
        Resolution::Bodypart { target, bodypart_index } => match source {
            Some(ActionSource::EquippedSlot(slot))   => IntentData::TargetBodypartWithEquipment { slot, target, bodypart_index },
            Some(ActionSource::InventoryItem(item))  => IntentData::TargetBodypartWithInventory { item, target, bodypart_index },
            None => unreachable!("bodypart targeting requires an item or slot source"),
        },
    };
    Intent { phase: action.phase, data, action: action.action }
}
