use specs::prelude::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use super::serde_collections::*;
use super::Map;

pub struct SerializeMarker;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: Map
}

#[derive (Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive (Component, Serialize, Deserialize, Clone)]
pub struct Enemy {}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Size {
    pub x: i32,
    pub y: i32
}

#[derive (Component, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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

#[derive (Component, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum ItemSlot {
    MainWeapon,
    OffhandWeapon,
    Head,
    Torso,
    Legs
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Facing {
    pub direction: Direction
}

#[derive (Component, Serialize, Deserialize, Clone)]
pub struct Vehicle {
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct LargeRenderable {
    pub glyphs: Vec<rltk::FontCharType>,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Name {
    pub value: String
}

#[derive (Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Firearm {
    pub range: i32
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Equippable {
    pub equipped: bool,
    pub slot: ItemSlot
}

#[derive (Component, Serialize, Deserialize, Clone)]
pub struct Gettable {}

#[derive (Component, Serialize, Deserialize, Clone)]
pub struct GettingItem {}

#[derive (Component, ConvertSaveload, Clone)]
pub struct DroppingItem {
    pub item: Entity
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct EquippingItem {
    pub item: Entity
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct Inventory {
    pub items: EntityVec<Entity>
}

#[derive (ConvertSaveload, Clone)]
pub struct BodyPart {
    pub item_slot: ItemSlot,
    pub max_hitpoints: i32,
    pub hitpoints: i32,
    pub equipped_item: EntityOption<Entity>
}

#[derive (Component, ConvertSaveload, Clone)]
pub struct HumanoidBody {
    pub max_hitpoints: i32,
    pub hitpoints: i32,

    pub head: BodyPart,
    pub torso: BodyPart,
    pub left_arm: BodyPart,
    pub right_arm: BodyPart,
    pub legs: BodyPart
}
impl HumanoidBody {
    pub fn new(max_hp: i32) -> HumanoidBody {
        HumanoidBody {
            max_hitpoints: max_hp,
            hitpoints: max_hp,
            head: BodyPart {
                item_slot: ItemSlot::Head,
                max_hitpoints: max_hp / 4,
                hitpoints: max_hp / 4,
                equipped_item: EntityOption::from(None)
            },
            torso: BodyPart {
                item_slot: ItemSlot::Torso,
                max_hitpoints: max_hp / 2,
                hitpoints: max_hp / 2,
                equipped_item: EntityOption::from(None)
            },
            left_arm: BodyPart {
                item_slot: ItemSlot::OffhandWeapon,
                max_hitpoints: max_hp / 5,
                hitpoints: max_hp / 5,
                equipped_item: EntityOption::from(None)
            },
            right_arm: BodyPart {
                item_slot: ItemSlot::MainWeapon,
                max_hitpoints: max_hp / 5,
                hitpoints: max_hp / 5,
                equipped_item: EntityOption::from(None)
            },
            legs: BodyPart {
                item_slot: ItemSlot::Legs,
                max_hitpoints: max_hp / 3,
                hitpoints: max_hp / 3,
                equipped_item: EntityOption::from(None)
            }
        }
    }
}
