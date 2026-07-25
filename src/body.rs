use crate::components::*;
use crate::item::*;
use crate::error::*;

#[derive(Clone)]
pub struct Body {
    pub parts: Vec<BodyPart>
}

#[derive(Clone)]
pub struct BodyPart {
    pub name: String,
    pub slots: Vec<ItemSlot>
}

#[derive(Clone)]
pub struct ItemSlot {
    pub slot_type: SlotType,
    pub item: Option<Item>
}

/// Note: Slot types have to be unique.
impl Body {
    pub fn human_body() -> Self {
        let mut body = Body { parts: vec!() };
        body.parts.push(BodyPart {
            name: "Head".to_string(),
            slots: vec!()
        });

        body.parts.push(BodyPart {
            name: "Torso".to_string(),
            slots: vec!()
        });

        body.parts.push(BodyPart {
            name: "Left arm".to_string(),
            slots: vec!(ItemSlot {slot_type: SlotType::SecondaryHand, item: None})
        });

        body.parts.push(BodyPart {
            name: "Right arm".to_string(),
            slots: vec!(ItemSlot {slot_type: SlotType::PrimaryHand, item: None})
        });

        body.parts.push(BodyPart {
            name: "Left leg".to_string(),
            slots: vec!()
        });

        body.parts.push(BodyPart {
            name: "Right leg".to_string(),
            slots: vec!()
        });

        return body;
    }

    pub fn can_equip(&self, item: Item) -> bool {
        let mut unsatisfied_slots = item.equip_slots.len();
        for item_slot in item.equip_slots {
            for body_part in &self.parts {
                for part_slot in &body_part.slots {
                    if item_slot == part_slot.slot_type {
                        unsatisfied_slots = unsatisfied_slots - 1;
                    }
                    if unsatisfied_slots == 0 {
                        return true;
                    }
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
        for (slot_number, item_slot) in item.clone().equip_slots.iter().enumerate() {
            for body_part in &mut self.parts {
                for part_slot in &mut body_part.slots {
                    if part_slot.slot_type == *item_slot {
                        if part_slot.item.is_some() {
                            let taken = part_slot.item.take().unwrap();
                            if !taken.proxy {
                                removed_items.push(taken);
                            }
                        }
                        if slot_number == 0 {
                            part_slot.item = Some(item.clone());
                        }
                        else {
                            part_slot.item = Some(item.proxy());
                        }
                    }
                }
            }
        }

        return Ok(removed_items);
    }
}