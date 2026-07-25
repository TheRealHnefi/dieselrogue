use crate::components::*;
use rltk::Point;

/// Concrete type containing all data of something that acts and moves.
#[derive(Clone)]
pub struct Entity {
    pub position: Point,
    pub renderable: Renderable,
    pub name: String,
    pub intent: Intent,
    pub facing: Facing
}

impl Entity {
    pub fn create_pawn(&self) -> Pawn {
        Pawn {
            renderable: self.renderable,
            name: self.name.clone(),
            intent: self.intent,
            facing: self.facing
        }
    }
}

/// Contains information typically needed to be referenced by others. Placed on the map for quick
/// indexing.
#[derive(Clone)]
pub struct Pawn {
    pub renderable: Renderable,
    pub name: String,
    pub intent: Intent,
    pub facing: Facing
}

