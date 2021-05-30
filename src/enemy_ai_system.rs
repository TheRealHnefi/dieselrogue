use legion::*;
use super::*;

#[system(for_each)]
pub fn enemy_ai(_pos: &mut Position,
                _facing: &mut Facing,
                _renderable: &mut Renderable,
                _viewshed: &mut Viewshed,
                _enemy: &Enemy,
                #[resource] _map: &mut Map) {
    
}

// impl<'a> System<'a> for EnemyAI {
//     type SystemData = ( WriteExpect<'a, Map>,
//                         ReadExpect<'a, Entity>,
//                         ReadExpect<'a, RunState>,
//                         WriteStorage<'a, Viewshed>,
//                         ReadStorage<'a, Enemy>,
//                         WriteStorage<'a, Position>,
//                         WriteStorage<'a, Facing>,
//                         WriteStorage<'a, Renderable>);

//     fn run(&mut self, data: Self::SystemData) {
//         let (mut map, player_entity, run_state, mut viewsheds, enemies, mut positions, mut facings, mut renderables) = data;

//         if *run_state != RunState::EnemyTurn {
//             return;
//         }

//         let player_pos;
//         {
//             let raw_player_pos = &positions.get(*player_entity).unwrap();            
//             player_pos = Point::new(raw_player_pos.x, raw_player_pos.y);
//         }

//         for (mut viewshed, _enemy, mut pos, mut facing, mut renderable) in 
//             (&mut viewsheds, &enemies, &mut positions, &mut facings, &mut renderables).join() {
        
//             let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), player_pos);
            
//             if viewshed.visible_tiles.contains(&player_pos) {
//                 // Path to the player
//                 let path = rltk::a_star_search(
//                     map.xy_idx(pos.x, pos.y),
//                     map.xy_idx(player_pos.x, player_pos.y),
//                     &mut *map
//                 );                
//                 if path.success {
//                     let step_x;
//                     let step_y;
//                     if path.steps.len() > 1 {
//                         step_x = path.steps[1] as i32 % map.width;
//                         step_y = path.steps[1] as i32 / map.width;
//                     } else {
//                         step_x = path.steps[0] as i32 % map.width;
//                         step_y = path.steps[0] as i32 / map.width;
//                     }

//                     let mut did_turn = false;

//                     let new_direction;
//                     if pos.x - step_x == 0 && pos.y - step_y == 1 { new_direction = Direction::Up }
//                     else if pos.x - step_x == -1 && pos.y - step_y == 1 { new_direction = Direction::UpRight }
//                     else if pos.x - step_x == -1 && pos.y - step_y == 0 { new_direction = Direction::Right }
//                     else if pos.x - step_x == -1 && pos.y - step_y == -1 { new_direction = Direction::DownRight }
//                     else if pos.x - step_x == 0 && pos.y - step_y == -1 { new_direction = Direction::Down }
//                     else if pos.x - step_x == 1 && pos.y - step_y == -1 { new_direction = Direction::DownLeft }
//                     else if pos.x - step_x == 1 && pos.y - step_y == 0 { new_direction = Direction::Left }
//                     else if pos.x - step_x == 1 && pos.y - step_y == 1 { new_direction = Direction::UpLeft }
//                     else { new_direction = Direction::Up }

//                     if new_direction != facing.direction {
//                         facing.direction = new_direction;
//                         viewshed.dirty = true;
//                         did_turn = true;
//                     }
//                     match new_direction {
//                         Direction::Up => {
//                             renderable.glyph = rltk::to_cp437('8');
//                         },
//                         Direction::UpRight => {
//                             renderable.glyph = rltk::to_cp437('9');
//                         },
//                         Direction::Right => {
//                             renderable.glyph = rltk::to_cp437('6');
//                         },
//                         Direction::DownRight => {
//                             renderable.glyph = rltk::to_cp437('3');
//                         },
//                         Direction::Down => {
//                             renderable.glyph = rltk::to_cp437('2');
//                         },
//                         Direction::DownLeft => {
//                             renderable.glyph = rltk::to_cp437('1');
//                         },
//                         Direction::Left => {
//                             renderable.glyph = rltk::to_cp437('4');
//                         },
//                         Direction::UpLeft => {
//                             renderable.glyph = rltk::to_cp437('7');
//                         }
//                     }
//                     if !did_turn {
//                         if distance < 1.5 {
//                             // Close enough to attack
//                         }
//                         else {
//                             let mut idx = map.xy_idx(pos.x, pos.y);
//                             map.blocked_tiles[idx] = false;

//                             pos.x = step_x;
//                             pos.y = step_y;

//                             idx = map.xy_idx(pos.x, pos.y);
//                             map.blocked_tiles[idx] = true;
//                             viewshed.dirty = true;
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }