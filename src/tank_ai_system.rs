use specs::prelude::*;
use super::{Viewshed, Position, Vehicle, RunState, Facing, Direction, LargeRenderable, Size};
use rltk::{Point};

pub struct TankAI {}

impl<'a> System<'a> for TankAI {
    type SystemData = ( ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Facing>,
                        WriteStorage<'a, LargeRenderable>,
                        ReadStorage<'a, Vehicle>,
                        ReadStorage<'a, Size>);

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, run_state, mut viewsheds, mut positions, mut facings, mut renderables, vehicles, sizes) = data;
        if *run_state != RunState::EnemyTurn {
            return;
        }

        let player_pos;
        {
            let raw_player_pos = &positions.get(*player_entity).unwrap();            
            player_pos = Point::new(raw_player_pos.x, raw_player_pos.y);
        }

        for (mut viewshed, _vehicle, position, mut facing, renderable, size) in 
            (&mut viewsheds, &vehicles, &mut positions, &mut facings, &mut renderables, (&sizes).maybe()).join() {

            let pos = match size {
                Some(s) => {
                    Point::new(position.x + s.x/2, position.y + s.y/2)
                }
                None => Point::new(position.x, position.y)
            };
                    
            if viewshed.visible_tiles.contains(&player_pos) {
                let new_direction;
                if pos.x - player_pos.x == 0 && pos.y - player_pos.y == 1 { new_direction = Direction::Up }
                else if pos.x - player_pos.x <= -1 && pos.y - player_pos.y >= 1 { new_direction = Direction::UpRight }
                else if pos.x - player_pos.x <= -1 && pos.y - player_pos.y == 0 { new_direction = Direction::Right }
                else if pos.x - player_pos.x <= -1 && pos.y - player_pos.y <= -1 { new_direction = Direction::DownRight }
                else if pos.x - player_pos.x == 0 && pos.y - player_pos.y <= -1 { new_direction = Direction::Down }
                else if pos.x - player_pos.x >= 1 && pos.y - player_pos.y <= -1 { new_direction = Direction::DownLeft }
                else if pos.x - player_pos.x >= 1 && pos.y - player_pos.y == 0 { new_direction = Direction::Left }
                else if pos.x - player_pos.x >= 1 && pos.y - player_pos.y >= 1 { new_direction = Direction::UpLeft }
                else { new_direction = Direction::Up }

                if new_direction != facing.direction {
                    facing.direction = new_direction;
                    viewshed.dirty = true;
                }
                match new_direction {
                    Direction::Up => {
                        renderable.glyphs[0] = rltk::to_cp437('╒');
                        renderable.glyphs[1] = rltk::to_cp437('│');
                        renderable.glyphs[2] = rltk::to_cp437('╕');
                        renderable.glyphs[3] = rltk::to_cp437('╞');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('╡');
                        renderable.glyphs[6] = rltk::to_cp437('╘');
                        renderable.glyphs[7] = rltk::to_cp437('═');
                        renderable.glyphs[8] = rltk::to_cp437('╛');
                    },
                    Direction::UpRight => {
                        renderable.glyphs[0] = rltk::to_cp437('┌');
                        renderable.glyphs[1] = rltk::to_cp437('/');
                        renderable.glyphs[2] = rltk::to_cp437('/');
                        renderable.glyphs[3] = rltk::to_cp437('/');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('/');
                        renderable.glyphs[6] = rltk::to_cp437('\\');
                        renderable.glyphs[7] = rltk::to_cp437('/');
                        renderable.glyphs[8] = rltk::to_cp437('┘');
                    },
                    Direction::Right => {
                        renderable.glyphs[0] = rltk::to_cp437('╓');
                        renderable.glyphs[1] = rltk::to_cp437('╥');
                        renderable.glyphs[2] = rltk::to_cp437('╖');
                        renderable.glyphs[3] = rltk::to_cp437('║');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('─');
                        renderable.glyphs[6] = rltk::to_cp437('╙');
                        renderable.glyphs[7] = rltk::to_cp437('╨');
                        renderable.glyphs[8] = rltk::to_cp437('╜');
                    },
                    Direction::DownRight => {
                        renderable.glyphs[0] = rltk::to_cp437('/');
                        renderable.glyphs[1] = rltk::to_cp437('\\');
                        renderable.glyphs[2] = rltk::to_cp437('┐');
                        renderable.glyphs[3] = rltk::to_cp437('\\');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('\\');
                        renderable.glyphs[6] = rltk::to_cp437('└');
                        renderable.glyphs[7] = rltk::to_cp437('\\');
                        renderable.glyphs[8] = rltk::to_cp437('\\');
                    },
                    Direction::Down => {
                        renderable.glyphs[0] = rltk::to_cp437('╒');
                        renderable.glyphs[1] = rltk::to_cp437('═');
                        renderable.glyphs[2] = rltk::to_cp437('╕');
                        renderable.glyphs[3] = rltk::to_cp437('╞');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('╡');
                        renderable.glyphs[6] = rltk::to_cp437('╘');
                        renderable.glyphs[7] = rltk::to_cp437('│');
                        renderable.glyphs[8] = rltk::to_cp437('╛');
                    },
                    Direction::DownLeft => {
                        renderable.glyphs[0] = rltk::to_cp437('┌');
                        renderable.glyphs[1] = rltk::to_cp437('/');
                        renderable.glyphs[2] = rltk::to_cp437('\\');
                        renderable.glyphs[3] = rltk::to_cp437('/');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('/');
                        renderable.glyphs[6] = rltk::to_cp437('/');
                        renderable.glyphs[7] = rltk::to_cp437('/');
                        renderable.glyphs[8] = rltk::to_cp437('┘');
                    },
                    Direction::Left => {
                        renderable.glyphs[0] = rltk::to_cp437('╓');
                        renderable.glyphs[1] = rltk::to_cp437('╥');
                        renderable.glyphs[2] = rltk::to_cp437('╖');
                        renderable.glyphs[3] = rltk::to_cp437('─');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('║');
                        renderable.glyphs[6] = rltk::to_cp437('╙');
                        renderable.glyphs[7] = rltk::to_cp437('╨');
                        renderable.glyphs[8] = rltk::to_cp437('╜');
                    },
                    Direction::UpLeft => {
                        renderable.glyphs[0] = rltk::to_cp437('\\');
                        renderable.glyphs[1] = rltk::to_cp437('\\');
                        renderable.glyphs[2] = rltk::to_cp437('┐');
                        renderable.glyphs[3] = rltk::to_cp437('\\');
                        renderable.glyphs[4] = rltk::to_cp437('█');
                        renderable.glyphs[5] = rltk::to_cp437('\\');
                        renderable.glyphs[6] = rltk::to_cp437('└');
                        renderable.glyphs[7] = rltk::to_cp437('\\');
                        renderable.glyphs[8] = rltk::to_cp437('/');
                    }
                }
            }
        }
    }
}