use legion::*;
use super::*;
use rltk::{field_of_view, Point};

#[system(for_each)]
pub fn update_visibility(pos: &Position,
                         facing: &Facing,
                         viewshed: &mut Viewshed,
                         name: &Name,
                         #[resource] map: &mut Map) {
    if viewshed.dirty {
        viewshed.dirty = false;
        viewshed.visible_tiles.clear();
        // MIGRATION_TODO: Reintroduce size
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

        // MIGRATION_TODO: Do not rely on name text match.
        if name.value == "Player" {
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
