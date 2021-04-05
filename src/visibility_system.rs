use specs::prelude::*;
use super::{Viewshed, Position, Map, Player, Facing, Direction};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (WriteExpect<'a, Map>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Player>,
                        ReadStorage<'a, Facing>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewsheds, positions, player, facings) = data;

        // This means viewshed REQUIRES a facing. May need revisiting.
        for (ent, viewshed, pos, facing) in (&entities, &mut viewsheds, &positions, &facings).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);

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
