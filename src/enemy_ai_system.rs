use specs::prelude::*;
use super::{Viewshed, Position, Map, Enemy, RunState};
use rltk::{Point};

pub struct EnemyAI {}

impl<'a> System<'a> for EnemyAI {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Enemy>,
                        WriteStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_entity, run_state, mut viewsheds, enemies, mut positions) = data;

        if *run_state != RunState::EnemyTurn {
            return;
        }

        let player_pos;
        {
            let raw_player_pos = &mut positions.get(*player_entity).unwrap();            
            player_pos = Point::new(raw_player_pos.x, raw_player_pos.y);
        }

        for (mut viewshed, _enemy, mut pos) in (&mut viewsheds, &enemies, &mut positions).join() {
        
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
            if distance < 1.5 {
                // Close enough to attack
            }
            else if viewshed.visible_tiles.contains(&player_pos) {
                // Path to the player
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y),
                    map.xy_idx(player_pos.x, player_pos.y),
                    &mut *map
                );
                if path.success && path.steps.len() > 1 {
                    let mut idx = map.xy_idx(pos.x, pos.y);
                    map.blocked_tiles[idx] = false;
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    idx = map.xy_idx(pos.x, pos.y);
                    map.blocked_tiles[idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}