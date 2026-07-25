use specs::prelude::*;
use super::{Map, Position, BlocksTile, Gettable, Size, Point};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        ReadStorage<'a, Gettable>,
                        ReadStorage<'a, Size>,
                        Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, gettables, sizes, entities) = data;

        map.populate_blocked();
        map.clear_contents_index();
        for (entity, position, blocker, size) in (&entities, &position, (&blockers).maybe(), (&sizes).maybe()).join() {
            let item: Option<&Gettable> = gettables.get(entity);
            let dimensions;
            match size {
                Some(s) => {
                    dimensions = Point::new(s.x, s.y);
                },
                None => {
                    dimensions = Point::new(1, 1);
                }
            }
            match (blocker, item) {
                (Some(_), None) => {
                    for y in 0..dimensions.y {
                        for x in 0..dimensions.x {
                            let index = map.xy_idx(position.x + x, position.y + y);
                            map.blocked_tiles[index] = true;
                            map.tile_blockers[index] = Some(entity);
                        }
                    }
                },
                (None, Some(_)) => {
                    let index = map.xy_idx(position.x, position.y);
                    map.tile_items[index] = Some(entity);
                }
                _ => {}
            }
        }
    }
}