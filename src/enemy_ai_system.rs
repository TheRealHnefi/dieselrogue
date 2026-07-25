use specs::prelude::*;
use super::{Viewshed, Position, Map, Enemy, RunState, Facing, Direction, Renderable};
use rltk::{Point};

pub struct EnemyAI {}

impl<'a> System<'a> for EnemyAI {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Enemy>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, Facing>,
                        WriteStorage<'a, Renderable>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_entity, run_state, mut viewsheds, enemies, mut positions, mut facings, mut renderables) = data;

        if *run_state != RunState::EnemyTurn {
            return;
        }

        let player_pos;
        {
            let raw_player_pos = &mut positions.get(*player_entity).unwrap();            
            player_pos = Point::new(raw_player_pos.x, raw_player_pos.y);
        }

        for (mut viewshed, _enemy, mut pos, mut facing, mut renderable) in 
            (&mut viewsheds, &enemies, &mut positions, &mut facings, &mut renderables).join() {
        
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
            
            if viewshed.visible_tiles.contains(&player_pos) {
                // Path to the player
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(player_pos.x, player_pos.y),
                    &mut *map
                );                
                if path.success {
                    let step_x = path.steps[1] as i32 % map.width;
                    let step_y = path.steps[1] as i32 / map.width;

                    let mut did_turn = false;

                    let new_direction;
                    if pos.x - step_x == 0 && pos.y - step_y == 1 { new_direction = Direction::UP }
                    else if pos.x - step_x == -1 && pos.y - step_y == 1 { new_direction = Direction::UPRIGHT }
                    else if pos.x - step_x == -1 && pos.y - step_y == 0 { new_direction = Direction::RIGHT }
                    else if pos.x - step_x == -1 && pos.y - step_y == -1 { new_direction = Direction::DOWNRIGHT }
                    else if pos.x - step_x == 0 && pos.y - step_y == -1 { new_direction = Direction::DOWN }
                    else if pos.x - step_x == 1 && pos.y - step_y == -1 { new_direction = Direction::DOWNLEFT }
                    else if pos.x - step_x == 1 && pos.y - step_y == 0 { new_direction = Direction::LEFT }
                    else if pos.x - step_x == 1 && pos.y - step_y == 1 { new_direction = Direction::UPLEFT }
                    else { new_direction = Direction::UP }

                    if new_direction != facing.direction {
                        facing.direction = new_direction;
                        viewshed.dirty = true;
                        did_turn = true;
                    }
                    match new_direction {
                        Direction::UP => {
                            renderable.glyph = rltk::to_cp437('8');
                        },
                        Direction::UPRIGHT => {
                            renderable.glyph = rltk::to_cp437('9');
                        },
                        Direction::RIGHT => {
                            renderable.glyph = rltk::to_cp437('6');
                        },
                        Direction::DOWNRIGHT => {
                            renderable.glyph = rltk::to_cp437('3');
                        },
                        Direction::DOWN => {
                            renderable.glyph = rltk::to_cp437('2');
                        },
                        Direction::DOWNLEFT => {
                            renderable.glyph = rltk::to_cp437('1');
                        },
                        Direction::LEFT => {
                            renderable.glyph = rltk::to_cp437('4');
                        },
                        Direction::UPLEFT => {
                            renderable.glyph = rltk::to_cp437('7');
                        }
                    }
                    if !did_turn {
                        if distance < 1.5 {
                            // Close enough to attack
                        }
                        else {
                            let mut idx = map.xy_idx(pos.x, pos.y);
                            map.blocked_tiles[idx] = false;

                            pos.x = step_x;
                            pos.y = step_y;

                            idx = map.xy_idx(pos.x, pos.y);
                            map.blocked_tiles[idx] = true;
                            viewshed.dirty = true;
                        }
                    }
                }
            }
        }
    }
}