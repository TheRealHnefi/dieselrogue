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
                if pos.x - player_pos.x == 0 && pos.y - player_pos.y == 1 { new_direction = Direction::UP }
                else if pos.x - player_pos.x <= -1 && pos.y - player_pos.y >= 1 { new_direction = Direction::UPRIGHT }
                else if pos.x - player_pos.x <= -1 && pos.y - player_pos.y == 0 { new_direction = Direction::RIGHT }
                else if pos.x - player_pos.x <= -1 && pos.y - player_pos.y <= -1 { new_direction = Direction::DOWNRIGHT }
                else if pos.x - player_pos.x == 0 && pos.y - player_pos.y <= -1 { new_direction = Direction::DOWN }
                else if pos.x - player_pos.x >= 1 && pos.y - player_pos.y <= -1 { new_direction = Direction::DOWNLEFT }
                else if pos.x - player_pos.x >= 1 && pos.y - player_pos.y == 0 { new_direction = Direction::LEFT }
                else if pos.x - player_pos.x >= 1 && pos.y - player_pos.y >= 1 { new_direction = Direction::UPLEFT }
                else { new_direction = Direction::UP }

                if new_direction != facing.direction {
                    facing.direction = new_direction;
                    viewshed.dirty = true;
                }
                match new_direction {
                    Direction::UP => {
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
                    Direction::UPRIGHT => {
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
                    Direction::RIGHT => {
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
                    Direction::DOWNRIGHT => {
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
                    Direction::DOWN => {
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
                    Direction::DOWNLEFT => {
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
                    Direction::LEFT => {
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
                    Direction::UPLEFT => {
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