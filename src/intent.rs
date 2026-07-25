use rltk::Point;
use crate::Item;
use crate::Entity;
use crate::Map;
use crate::{EntityKind, DrivingState, Ability};
use crate::error::{Error, GameError};
use crate::components::*;
use crate::actions;
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
    GetItem,
    Unequip,
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

/// A plain move onto `target`.
pub fn move_intent(target: Point) -> Intent {
    Intent { phase: ExecutionPhase::Movement, data: IntentData::Target(target), action: actions::move_action }
}

/// Resolve a directional step for `entity` into a concrete intent: turn, move,
/// melee, open-door or embark depending on facing and the target tile. Shared by
/// the player hotkey and the AI. `Ok(None)` means no intent change (e.g. a vehicle
/// blocked by terrain).
pub fn resolve_step(entity: &Entity, direction: Direction, map: &Map, entities: &[Entity]) -> Result<Option<Intent>, GameError> {
    let driving = matches!(entity.driving, DrivingState::DrivenBy(_));
    let can_move = entity.has_ability(Ability::HumanMove) || entity.has_ability(Ability::VehicleMove);
    if !can_move {
        return Err(GameError { error: Error::BadPrecondition, message: "Player can not move".to_string() });
    }

    if entity.body.facing != direction {
        return Ok(Some(Intent {
            phase: ExecutionPhase::Movement,
            data: IntentData::Direction(direction),
            action: actions::turn_action,
        }));
    }

    let (dx, dy) = direction.delta_pos();
    let target = Point { x: entity.position.x + dx, y: entity.position.y + dy };

    if !driving {
        if target.x < 0 || target.y < 0 || target.x >= map.width as i32 || target.y >= map.height as i32 {
            return Err(GameError { error: Error::MapExit, message: String::new() });
        }
        let index = map.xy_idx(target.x, target.y);
        let pawn_entity_id = map.pawns[index].as_ref().map(|p| p.entity_id);
        match pawn_entity_id {
            Some(pid) => {
                let pawn = &entities[pid];
                if pawn.kind == EntityKind::Door {
                    Ok(Some(Intent { phase: ExecutionPhase::Movement, data: IntentData::Target(target), action: actions::open_door_action }))
                } else if pawn.driving == DrivingState::Drivable {
                    if !entity.has_ability(Ability::Embark) {
                        return Err(GameError { error: Error::BadPrecondition, message: "You don't know how to operate that vehicle.".to_string() });
                    }
                    Ok(Some(Intent { phase: ExecutionPhase::Movement, data: IntentData::Target(target), action: actions::embark_action }))
                } else {
                    Ok(Some(Intent { phase: ExecutionPhase::Attack, data: IntentData::Target(target), action: actions::melee_action }))
                }
            },
            None => {
                if !entity.check_fit(target, map) {
                    return Err(GameError { error: Error::BadPrecondition, message: "Bump!".to_string() });
                }
                Ok(Some(move_intent(target)))
            }
        }
    } else if entity.check_fit(target, map) {
        Ok(Some(move_intent(target)))
    } else {
        Ok(None)
    }
}

/// Hotkey-only catalog descriptor for picking up the item underfoot.
pub fn get_item_action_def() -> EntityAction {
    EntityAction { id: ActionId::GetItem, name: "Pick up".to_string(), targeting: Targeting::None, phase: ExecutionPhase::Inventory, precondition: precondition_ok, action: actions::get_item_action }
}

/// Descriptor for unequipping an item back to inventory.
pub fn unequip_action_def() -> EntityAction {
    EntityAction { id: ActionId::Unequip, name: "Unequip".to_string(), targeting: Targeting::None, phase: ExecutionPhase::Inventory, precondition: precondition_ok, action: actions::unequip_item_action }
}
