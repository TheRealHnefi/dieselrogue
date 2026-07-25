use rltk::Point;
use crate::components::*;
use crate::item::*;
use crate::error::*;

#[derive(Clone)]
pub struct Body {
    pub position: Point,
    pub facing: Direction,
    pub inventory: Vec<Item>,
    pub parts: Vec<BodyPart>,
    pub item_slots: Vec<ItemSlot>
}

#[derive(Clone)]
pub struct BodyPart {
    pub name: String,
    pub slot_index: Vec<usize>
}

#[derive(Clone)]
pub struct ItemSlot {
    pub slot_type: SlotType,
    pub item: Option<Item>
}

/// Note: Slot types have to be unique.
impl Body {
    pub fn human_body(pos: Point, facing: Direction) -> Self {
        let mut body = Body { position: pos, facing: facing, inventory: vec!(), parts: vec!(), item_slots: vec!() };

        body.item_slots.push(ItemSlot {slot_type: SlotType::Headwear, item: None});
        body.parts.push(BodyPart {
            name: "Head".to_string(),
            slot_index: vec!(body.item_slots.len() - 1)
        });

        body.item_slots.push(ItemSlot {slot_type: SlotType::Bodywear, item: None});
        body.parts.push(BodyPart {
            name: "Torso".to_string(),
            slot_index: vec!(body.item_slots.len() - 1)
        });

        body.item_slots.push(ItemSlot {slot_type: SlotType::PrimaryHand, item: None});
        body.item_slots.push(ItemSlot {slot_type: SlotType::RightArmwear, item: None});
        body.parts.push(BodyPart {
            name: "Right arm".to_string(),
            slot_index: vec!(body.item_slots.len() - 2, body.item_slots.len() - 1)
        });

        body.item_slots.push(ItemSlot {slot_type: SlotType::SecondaryHand, item: None});
        body.item_slots.push(ItemSlot {slot_type: SlotType::LeftArmwear, item: None});
        body.parts.push(BodyPart {
            name: "Left arm".to_string(),
            slot_index: vec!(body.item_slots.len() - 2, body.item_slots.len() - 1)
        });

        body.item_slots.push(ItemSlot {slot_type: SlotType::Legwear, item: None});
        body.item_slots.push(ItemSlot {slot_type: SlotType::Footwear, item: None});
        body.parts.push(BodyPart {
            name: "Legs".to_string(),
            slot_index: vec!(body.item_slots.len() - 2, body.item_slots.len() - 1)
        });

        for i in 0 .. body.item_slots.len() - 1 {
            for j in i .. body.item_slots.len() - 1 {
                if j != i {
                    debug_assert!(body.item_slots[i].slot_type != body.item_slots[j].slot_type);
                }
            }
        }

        return body;
    }

    pub fn can_equip(&self, item: Item) -> bool {
        let mut unsatisfied_slots = item.equip_slots.len();
        for item_slot in item.equip_slots {
            for self_slot in &self.item_slots {
                if item_slot == self_slot.slot_type {
                    unsatisfied_slots = unsatisfied_slots - 1;
                }
                if unsatisfied_slots == 0 {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn equip(&mut self, item: Item) -> Result<Vec<Item>, GameError> {
        if !self.can_equip(item.clone()) {
            return Err(GameError{error: Error::UnsolvableSituation, message: String::from("Cannot equip item")});
        }
        let mut removed_items = vec!();

        for slot in &item.equip_slots {
            match self.unequip(*slot) {
                Some(removed) => removed_items.push(removed),
                None => ()
            }
        }

        for (slot_number, item_slot) in item.clone().equip_slots.iter().enumerate() {
            for self_slot in &mut self.item_slots {
                if self_slot.slot_type == *item_slot {
                    if slot_number == 0 {
                        self_slot.item = Some(item.clone());
                    }
                    else {
                        self_slot.item = Some(item.proxy());
                    }
                }
            }
        }

        return Ok(removed_items);
    }

    pub fn unequip(&mut self, slot: SlotType) -> Option<Item> {
        for self_slot in &mut self.item_slots {
            if self_slot.slot_type == slot {
                let removed_item = self_slot.item.take();
                match &removed_item {
                    Some(item) => {
                        for proxy_slot in &item.equip_slots {
                            self.clear_slot(*proxy_slot);
                        }
                    },
                    None => ()
                }
                return removed_item;
            }
        }
        None
    }

    pub fn get_item(&mut self, slot: SlotType) -> Option<&mut Item> {
        for self_slot in &mut self.item_slots {
            if self_slot.slot_type == slot {
                return self_slot.item.as_mut();
            }
        }
        None
    }

    fn clear_slot(&mut self, slot: SlotType) {
        for self_slot in &mut self.item_slots {
            if self_slot.slot_type == slot {
                self_slot.item = None;
                return;
            }
        }
    }
}