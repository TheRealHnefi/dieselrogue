use specs::prelude::*;
use specs_derive::*;

#[derive (Component)]
pub struct Player {}

#[derive (Component)]
pub struct Enemy {}

#[derive (Component)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive (Component)]
pub struct Size {
    pub x: i32,
    pub y: i32
}

#[derive (Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    UP,
    UPRIGHT,
    RIGHT,
    DOWNRIGHT,
    DOWN,
    DOWNLEFT,
    LEFT,
    UPLEFT
}

#[derive (Component, Debug)]
pub struct Facing {
    pub direction: Direction
}

#[derive (Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

#[derive (Component)]
pub struct LargeRenderable {
    pub glyphs: Vec<rltk::FontCharType>,
    pub color: rltk::RGB,
    pub background: rltk::RGB
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive (Component)]
pub struct Name {
    pub value: String
}

#[derive (Component)]
pub struct BlocksTile {}

#[derive (Component)]
pub struct GettableItem {}

#[derive (Component)]
pub struct GettingItem {}

#[derive (Component)]
pub struct Inventory {
    pub items: Vec<Entity>
}

pub struct BodyPart {
    pub name: String,
    pub max_hitpoints: i32,
    pub hitpoints: i32
}

#[derive (Component)]
pub struct HumanoidBody {
    pub max_hitpoints: i32,
    pub hitpoints: i32,

    pub head: BodyPart,
    pub torso: BodyPart,
    pub left_arm: BodyPart,
    pub right_arm: BodyPart,
    pub left_leg: BodyPart,
    pub right_leg: BodyPart
}
impl HumanoidBody {
    pub fn new(max_hp: i32) -> HumanoidBody {
        HumanoidBody {
            max_hitpoints: max_hp,
            hitpoints: max_hp,
            head: BodyPart {
                name: "head".to_string(),
                max_hitpoints: max_hp / 4,
                hitpoints: max_hp / 4
            },
            torso: BodyPart {
                name: "torso".to_string(),
                max_hitpoints: max_hp / 2,
                hitpoints: max_hp / 2
            },
            left_arm: BodyPart {
                name: "left arm".to_string(),
                max_hitpoints: max_hp / 5,
                hitpoints: max_hp / 5
            },
            right_arm: BodyPart {
                name: "right arm".to_string(),
                max_hitpoints: max_hp / 5,
                hitpoints: max_hp / 5
            },
            left_leg: BodyPart {
                name: "left leg".to_string(),
                max_hitpoints: max_hp / 3,
                hitpoints: max_hp / 3
            },
            right_leg: BodyPart {
                name: "right leg".to_string(),
                max_hitpoints: max_hp / 3,
                hitpoints: max_hp / 3
            }
        }
    }
}