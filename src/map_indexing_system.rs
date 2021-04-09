use specs::prelude::*;
use super::{Map, Position, BlocksTile, GettableItem};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        ReadStorage<'a, GettableItem>,
                        Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, gettables, entities) = data;

        map.populate_blocked();
        map.clear_contents_index();
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            let actor: Option<&BlocksTile> = blockers.get(entity);
            let item: Option<&GettableItem> = gettables.get(entity);
            match (actor, item) {
                (Some(_), None) => {
                    map.blocked_tiles[idx] = true;
                    map.tile_blockers[idx] = Some(entity);
                },
                (None, Some(_)) => {
                    map.tile_items[idx] = Some(entity);
                }
                _ => {}
            }
        }
    }
}