use crate::components::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Sprite {
    Human,
    Tank,
    Door
}

impl Sprite {
    /// The solid base glyph for the sprite, drawn at full opacity on the map layer.
    pub fn glyph(&self, facing: Direction, index: u32) -> crate::FontCharType {
        match self {
            Sprite::Human => crate::to_cp437('☺'),
            Sprite::Tank => self.tank_sprite(index, facing),
            Sprite::Door => self.door_sprite(facing),
        }
    }

    /// The facing arrow to overlay translucently on top of the base glyph, if any.
    /// Sprites that already encode direction in their base glyph return None.
    pub fn direction_glyph(&self, facing: Direction) -> Option<crate::FontCharType> {
        match self {
            Sprite::Human => Some(match facing {
                Direction::Up => crate::to_cp437('▲'),
                Direction::UpRight => crate::to_cp437('┐'),
                Direction::Right => crate::to_cp437('►'),
                Direction::DownRight => crate::to_cp437('┘'),
                Direction::Down => crate::to_cp437('▼'),
                Direction::DownLeft => crate::to_cp437('└'),
                Direction::Left => crate::to_cp437('◄'),
                Direction::UpLeft => crate::to_cp437('┌'),
            }),
            Sprite::Tank | Sprite::Door => None,
        }
    }

    fn tank_sprite(&self, index: u32, facing: Direction) -> crate::FontCharType {
        if index > 8 {
            return crate::to_cp437('?');
        }

        return self.tank_sprite_sheet(facing)[index as usize];
    }

    fn tank_sprite_sheet(&self, facing: Direction) -> [crate::FontCharType; 9] {
        match facing {
            Direction::Up => {
                [
                    crate::to_cp437('╒'),
                    crate::to_cp437('│'),
                    crate::to_cp437('╕'),
                    crate::to_cp437('╞'),
                    crate::to_cp437('█'),
                    crate::to_cp437('╡'),
                    crate::to_cp437('╘'),
                    crate::to_cp437('═'),
                    crate::to_cp437('╛'),
                ]
            },
            Direction::UpRight => {
                [
                    crate::to_cp437('┌'),
                    crate::to_cp437('/'),
                    crate::to_cp437('/'),
                    crate::to_cp437('/'),
                    crate::to_cp437('█'),
                    crate::to_cp437('/'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('/'),
                    crate::to_cp437('┘'),
                ]
            },
            Direction::Right => {
                [
                    crate::to_cp437('╓'),
                    crate::to_cp437('╥'),
                    crate::to_cp437('╖'),
                    crate::to_cp437('║'),
                    crate::to_cp437('█'),
                    crate::to_cp437('─'),
                    crate::to_cp437('╙'),
                    crate::to_cp437('╨'),
                    crate::to_cp437('╜'),
                ]
            },
            Direction::DownRight => {
                [
                    crate::to_cp437('/'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('┐'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('█'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('└'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('\\'),
                ]
            },
            Direction::Down => {
                [
                    crate::to_cp437('╒'),
                    crate::to_cp437('═'),
                    crate::to_cp437('╕'),
                    crate::to_cp437('╞'),
                    crate::to_cp437('█'),
                    crate::to_cp437('╡'),
                    crate::to_cp437('╘'),
                    crate::to_cp437('│'),
                    crate::to_cp437('╛'),
                ]
            },
            Direction::DownLeft => {
                [
                    crate::to_cp437('┌'),
                    crate::to_cp437('/'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('/'),
                    crate::to_cp437('█'),
                    crate::to_cp437('/'),
                    crate::to_cp437('/'),
                    crate::to_cp437('/'),
                    crate::to_cp437('┘'),
                ]
            },
            Direction::Left => {
                [
                    crate::to_cp437('╓'),
                    crate::to_cp437('╥'),
                    crate::to_cp437('╖'),
                    crate::to_cp437('─'),
                    crate::to_cp437('█'),
                    crate::to_cp437('║'),
                    crate::to_cp437('╙'),
                    crate::to_cp437('╨'),
                    crate::to_cp437('╜'),
                ]
            },
            Direction::UpLeft => {
                [
                    crate::to_cp437('\\'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('┐'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('█'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('└'),
                    crate::to_cp437('\\'),
                    crate::to_cp437('/'),
                ]
            }
        }
    }

    fn door_sprite(&self, facing: Direction) -> crate::FontCharType {
        match facing {
            Direction::Up => crate::to_cp437('║'),
            Direction::Right => crate::to_cp437('═'),
            Direction::Down => crate::to_cp437('║'),
            Direction::Left => crate::to_cp437('═'),
            _ => crate::to_cp437('?')
        }
    }
}


