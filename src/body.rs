use crate::components::*;
use crate::item::*;

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

pub fn human_body() -> Body {
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
        slots: vec!(ItemSlot {slot_type: SlotType::LeftHand, item: None})
    });

    body.parts.push(BodyPart {
        name: "Right arm".to_string(),
        slots: vec!(ItemSlot {slot_type: SlotType::RightHand, item: None})
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