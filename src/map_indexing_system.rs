use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_contents_index();
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked_tiles[idx] = true;
            }

            map.tile_contents[idx].push(entity);
        }
    }
}