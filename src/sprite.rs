use crate::components::*;

#[derive(Clone)]
pub enum Sprite {
    Human,
    Tank
}

impl Sprite {
    pub fn glyph(&self, facing: Direction, index: u32) -> rltk::FontCharType {
        match self {
            Sprite::Human => self.human_sprite(facing),
            Sprite::Tank => self.tank_sprite(index, facing),
        }
    }

    fn human_sprite(&self, facing: Direction) -> rltk::FontCharType {
        match facing {
            Direction::Up => rltk::to_cp437('8'),
            Direction::UpRight => rltk::to_cp437('9'),
            Direction::Right => rltk::to_cp437('6'),
            Direction::DownRight => rltk::to_cp437('3'),
            Direction::Down => rltk::to_cp437('2'),
            Direction::DownLeft => rltk::to_cp437('1'),
            Direction::Left => rltk::to_cp437('4'),
            Direction::UpLeft => rltk::to_cp437('7'),
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
}


