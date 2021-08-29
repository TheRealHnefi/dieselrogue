use crate::components::*;
use rltk::Point;
use crate::Map;

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

    pub fn resolve(&mut self, map: &mut Map) {
        let old_index = map.xy_idx(self.position.x, self.position.y);
        let mut new_index = old_index;

        match self.intent.action {
            Action::Idle => {},
            Action::Move(pos) => {
                if !map.blocked(pos.x, pos.y) {
                    new_index = map.xy_idx(pos.x, pos.y);
                    self.position = pos;
                }
            },
            Action::Turn(direction) => {
                self.facing.direction = direction;

                match direction {
                    Direction::Up => {self.renderable.glyph = rltk::to_cp437('8')},
                    Direction::UpRight => {self.renderable.glyph = rltk::to_cp437('9')},
                    Direction::Right => {self.renderable.glyph = rltk::to_cp437('6')},
                    Direction::DownRight => {self.renderable.glyph = rltk::to_cp437('3')},
                    Direction::Down => {self.renderable.glyph = rltk::to_cp437('2')},
                    Direction::DownLeft => {self.renderable.glyph = rltk::to_cp437('1')},
                    Direction::Left => {self.renderable.glyph = rltk::to_cp437('4')},
                    Direction::UpLeft => {self.renderable.glyph = rltk::to_cp437('7')},
                }
            }
        }

        map.pawns[old_index] = None;
        map.pawns[new_index] = Some(self.create_pawn());
        self.intent = Intent {action: Action::Idle};
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

