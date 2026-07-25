use crate::components::*;
use crate::Map;
use rltk::Point;

#[derive(Clone)]
pub struct Item {
    pub renderable: Renderable,
    pub name: String,
    pub inventory_actions: Vec<ItemAction>
}

impl Item {
    pub fn grenade() -> Self {
        let throw_action = ItemAction {
            label: String::from("Throw"),
            targeting: TargetingType::Position,
            cost: UsageCost::Consume,
            effect: throw_grenade
        };
        Item {
            renderable: Renderable::new_glyph('g'),
            name: String::from("Grenade"),
            inventory_actions: vec![throw_action]
        }
    }
}

fn throw_grenade(source_position: Point, target_position: Point, map: &Map) -> Option<Effect> {
    None
}