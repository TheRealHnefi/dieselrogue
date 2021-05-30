use legion::*;
use super::*;

// MIGRATION_TODO: Add size support.
// This is also rather inefficient. Can we update indexing while handling entities instead?
#[system(for_each)]
pub fn map_index_blockables(pos: &Position,
                            _blocker: &BlocksTile,
                            entity: &Entity,
                            #[resource] map: &mut Map) {
    
    let index = map.xy_idx(pos.x, pos.y);
    map.blocked_tiles[index] = true;
    map.tile_blockers[index] = Some(*entity);
}

#[system(for_each)]
pub fn map_index_items(pos: &Position,
                           _gettable: &Gettable,
                           entity: &Entity,
                           #[resource] map: &mut Map) {
                               
    let index = map.xy_idx(pos.x, pos.y);
    map.tile_items[index] = Some(*entity);
}
