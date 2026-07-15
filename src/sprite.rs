use crate::components::*;

#[derive(Clone)]
pub enum Sprite {
    Human,
    Tank,
    Door
}

impl Sprite {
    /// The solid base glyph for the sprite, drawn at full opacity on the map layer.
    pub fn glyph(&self, facing: Direction, index: u32) -> rltk::FontCharType {
        match self {
            Sprite::Human => rltk::to_cp437('☺'),
            Sprite::Tank => self.tank_sprite(index, facing),
            Sprite::Door => self.door_sprite(facing),
        }
    }

    /// The facing arrow to overlay translucently on top of the base glyph, if any.
    /// Sprites that already encode direction in their base glyph return None.
    pub fn direction_glyph(&self, facing: Direction) -> Option<rltk::FontCharType> {
        match self {
            Sprite::Human => Some(match facing {
                Direction::Up => rltk::to_cp437('▲'),
                Direction::UpRight => rltk::to_cp437('┐'),
                Direction::Right => rltk::to_cp437('►'),
                Direction::DownRight => rltk::to_cp437('┘'),
                Direction::Down => rltk::to_cp437('▼'),
                Direction::DownLeft => rltk::to_cp437('└'),
                Direction::Left => rltk::to_cp437('◄'),
                Direction::UpLeft => rltk::to_cp437('┌'),
            }),
            Sprite::Tank | Sprite::Door => None,
        }
    }

    fn tank_sprite(&self, index: u32, facing: Direction) -> rltk::FontCharType {
        if index > 8 {
            return rltk::to_cp437('?');
        }

        return self.tank_sprite_sheet(facing)[index as usize];
    }

    fn tank_sprite_sheet(&self, facing: Direction) -> [rltk::FontCharType; 9] {
        match facing {
            Direction::Up => {
                [
                    rltk::to_cp437('╒'),
                    rltk::to_cp437('│'),
                    rltk::to_cp437('╕'),
                    rltk::to_cp437('╞'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('╡'),
                    rltk::to_cp437('╘'),
                    rltk::to_cp437('═'),
                    rltk::to_cp437('╛'),
                ]
            },
            Direction::UpRight => {
                [
                    rltk::to_cp437('┌'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('┘'),
                ]
            },
            Direction::Right => {
                [
                    rltk::to_cp437('╓'),
                    rltk::to_cp437('╥'),
                    rltk::to_cp437('╖'),
                    rltk::to_cp437('║'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('─'),
                    rltk::to_cp437('╙'),
                    rltk::to_cp437('╨'),
                    rltk::to_cp437('╜'),
                ]
            },
            Direction::DownRight => {
                [
                    rltk::to_cp437('/'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('┐'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('└'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('\\'),
                ]
            },
            Direction::Down => {
                [
                    rltk::to_cp437('╒'),
                    rltk::to_cp437('═'),
                    rltk::to_cp437('╕'),
                    rltk::to_cp437('╞'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('╡'),
                    rltk::to_cp437('╘'),
                    rltk::to_cp437('│'),
                    rltk::to_cp437('╛'),
                ]
            },
            Direction::DownLeft => {
                [
                    rltk::to_cp437('┌'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('/'),
                    rltk::to_cp437('┘'),
                ]
            },
            Direction::Left => {
                [
                    rltk::to_cp437('╓'),
                    rltk::to_cp437('╥'),
                    rltk::to_cp437('╖'),
                    rltk::to_cp437('─'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('║'),
                    rltk::to_cp437('╙'),
                    rltk::to_cp437('╨'),
                    rltk::to_cp437('╜'),
                ]
            },
            Direction::UpLeft => {
                [
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('┐'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('█'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('└'),
                    rltk::to_cp437('\\'),
                    rltk::to_cp437('/'),
                ]
            }
        }
    }

    fn door_sprite(&self, facing: Direction) -> rltk::FontCharType {
        match facing {
            Direction::Up => rltk::to_cp437('║'),
            Direction::Right => rltk::to_cp437('═'),
            Direction::Down => rltk::to_cp437('║'),
            Direction::Left => rltk::to_cp437('═'),
            _ => rltk::to_cp437('?')
        }
    }
}


