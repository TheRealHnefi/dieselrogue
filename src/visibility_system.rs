use legion::*;
use super::*;
use rltk::{field_of_view, Point};

#[system(for_each)]
pub fn update_visibility(pos: &Position, facing: &Facing, viewshed: &mut Viewshed, #[resource] map: &mut Map) {
    if viewshed.dirty {
        viewshed.dirty = false;
        viewshed.visible_tiles.clear();
        // let pos = match size {
        //     Some(s) => {
        //         Point::new(position.x + s.x/2, position.y + s.y/2)
        //     }
        //     None => Point::new(position.x, position.y)
        // };
        viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);

        match facing.direction {
            Direction::Up => {
                viewshed.visible_tiles.retain(|p| p.y <= pos.y &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::UpRight => {
                viewshed.visible_tiles.retain(|p| p.x - pos.x >= p.y - pos.y &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::Right => {
                viewshed.visible_tiles.retain(|p| p.x >= pos.x &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::DownRight => {
                viewshed.visible_tiles.retain(|p| p.x - pos.x >= -(p.y - pos.y) &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::Down => {
                viewshed.visible_tiles.retain(|p| p.y >= pos.y &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::DownLeft => {
                viewshed.visible_tiles.retain(|p| p.x - pos.x <= p.y - pos.y &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::Left => {
                viewshed.visible_tiles.retain(|p| p.x <= pos.x &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            },
            Direction::UpLeft => {
                viewshed.visible_tiles.retain(|p| p.x - pos.x <= -(p.y - pos.y) &&
                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
            }
        }

        for vis_flag in map.visible_tiles.iter_mut() {
            *vis_flag = false;
        }
        for vis in viewshed.visible_tiles.iter() {
            let idx = map.xy_idx(vis.x, vis.y);
            map.revealed_tiles[idx] = true;
            map.visible_tiles[idx] = true;
        }
    }
}

// pub struct VisibilitySystem {}

// impl<'a> System<'a> for VisibilitySystem {
//     type SystemData = (WriteExpect<'a, Map>,
//                         Entities<'a>,
//                         WriteStorage<'a, Viewshed>,
//                         WriteStorage<'a, Position>,
//                         ReadStorage<'a, Player>,
//                         ReadStorage<'a, Facing>,
//                         ReadStorage<'a, Size>);

//     fn run(&mut self, data: Self::SystemData) {
//         let (mut map, entities, mut viewsheds, positions, player, facings, sizes) = data;

//         for (ent, viewshed, position, size) in (&entities, &mut viewsheds, &positions, (&sizes).maybe()).join() {
//             if viewshed.dirty {
//                 viewshed.dirty = false;
//                 viewshed.visible_tiles.clear();
//                 let pos = match size {
//                     Some(s) => {
//                         Point::new(position.x + s.x/2, position.y + s.y/2)
//                     }
//                     None => Point::new(position.x, position.y)
//                 };
//                 viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);

//                 let maybe_facing: Option<&Facing> = facings.get(ent);
//                 match maybe_facing {
//                     None => {}
//                     Some(facing) => {
//                         match facing.direction {
//                             Direction::Up => {
//                                 viewshed.visible_tiles.retain(|p| p.y <= pos.y &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::UpRight => {
//                                 viewshed.visible_tiles.retain(|p| p.x - pos.x >= p.y - pos.y &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::Right => {
//                                 viewshed.visible_tiles.retain(|p| p.x >= pos.x &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::DownRight => {
//                                 viewshed.visible_tiles.retain(|p| p.x - pos.x >= -(p.y - pos.y) &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::Down => {
//                                 viewshed.visible_tiles.retain(|p| p.y >= pos.y &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::DownLeft => {
//                                 viewshed.visible_tiles.retain(|p| p.x - pos.x <= p.y - pos.y &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::Left => {
//                                 viewshed.visible_tiles.retain(|p| p.x <= pos.x &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             },
//                             Direction::UpLeft => {
//                                 viewshed.visible_tiles.retain(|p| p.x - pos.x <= -(p.y - pos.y) &&
//                                     p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
//                             }
//                         }
//                     }
//                 }

//                 let p: Option<&Player> = player.get(ent);
//                 if let Some(_p) = p {
//                     for vis_flag in map.visible_tiles.iter_mut() {
//                         *vis_flag = false;
//                     }
//                     for vis in viewshed.visible_tiles.iter() {
//                         let idx = map.xy_idx(vis.x, vis.y);
//                         map.revealed_tiles[idx] = true;
//                         map.visible_tiles[idx] = true;
//                     }
//                 }
//             }
//         }
//     }
// }
