use crate::components::*;
use rltk::Point;

#[derive(Clone)]
pub struct Entity {
    pub position: Point,
    pub renderable: Renderable,
    pub name: String,
    pub intent: Intent,
    pub facing: Facing
}
