use specs::prelude::*;
use super::{Viewshed, Position, Map, Player, Facing, Direction, Size};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player>,
                        ReadStorage<'a, Facing>,
                        ReadStorage<'a, Size>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewsheds, positions, player, facings, sizes) = data;

        for (ent, viewshed, position, size) in (&entities, &mut viewsheds, &positions, (&sizes).maybe()).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                let pos = match size {
                    Some(s) => {
                        Point::new(position.x + s.x/2, position.y + s.y/2)
                    }
                    None => Point::new(position.x, position.y)
                };
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);

                let maybe_facing: Option<&Facing> = facings.get(ent);
                match maybe_facing {
                    None => {}
                    Some(facing) => {
                        match facing.direction {
                            Direction::UP => {
                                viewshed.visible_tiles.retain(|p| p.y <= pos.y &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::UPRIGHT => {
                                viewshed.visible_tiles.retain(|p| p.x - pos.x >= p.y - pos.y &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::RIGHT => {
                                viewshed.visible_tiles.retain(|p| p.x >= pos.x &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::DOWNRIGHT => {
                                viewshed.visible_tiles.retain(|p| p.x - pos.x >= -(p.y - pos.y) &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::DOWN => {
                                viewshed.visible_tiles.retain(|p| p.y >= pos.y &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::DOWNLEFT => {
                                viewshed.visible_tiles.retain(|p| p.x - pos.x <= p.y - pos.y &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::LEFT => {
                                viewshed.visible_tiles.retain(|p| p.x <= pos.x &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            },
                            Direction::UPLEFT => {
                                viewshed.visible_tiles.retain(|p| p.x - pos.x <= -(p.y - pos.y) &&
                                    p.x >= 0 && p.x < map.width && p.y > 0 && p.y < map.height);
                            }
                        }
                    }
                }

                let p: Option<&Player> = player.get(ent);
                if let Some(_p) = p {
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
        }
    }
}
