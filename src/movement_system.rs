use legion::*;
use super::*;
use std::cmp::{min, max};

// TODO: How to handle collisions between moving objects?
#[system(for_each)]
pub fn movement(pos: &mut Position,
                viewshed: &mut Viewshed,
                intent: &mut Intent,
                entity: &Entity,
                #[resource] map: &mut Map) {
    match intent.action {
        Action::Walk(direction) => {
            let (delta_x, delta_y);
            match direction {
                Direction::Up => {delta_x = 0; delta_y = -1;},
                Direction::UpRight => {delta_x = 1; delta_y = -1;},
                Direction::Right => {delta_x = 1; delta_y = 0;},
                Direction::DownRight => {delta_x = 1; delta_y = 1;},
                Direction::Down => {delta_x = 0; delta_y = 1;},
                Direction::DownLeft => {delta_x = -1; delta_y = 1;},
                Direction::Left => {delta_x = -1; delta_y = 0;},
                Direction::UpLeft => {delta_x = -1; delta_y = -1;},
            }
            let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
            let orig_idx = map.xy_idx(pos.x, pos.y);
            if !map.blocked_tiles[dest_idx] {
                pos.x = min(map.width - 1, max(0, pos.x + delta_x));
                pos.y = min(map.height - 1, max(0, pos.y + delta_y));

                viewshed.dirty = true;

                map.blocked_tiles[dest_idx] = true;
                map.tile_blockers[dest_idx] = Some(*entity);

                map.blocked_tiles[orig_idx] = false;
                map.tile_blockers[orig_idx] = None;
            }

            intent.action = Action::Idle;
        },
        _ => ()
    }
}

#[system(for_each)]
pub fn turning(facing: &mut Facing,
               viewshed: &mut Viewshed,
               renderable: &mut Renderable,
               intent: &mut Intent) {
    match intent.action {
        Action::Turn(direction) => {
            if direction != facing.direction {
                viewshed.dirty = true;
                facing.direction = direction;
                match direction {
                    Direction::Up => {renderable.glyph = rltk::to_cp437('▲')},
                    Direction::UpRight => {renderable.glyph = rltk::to_cp437('┐')},
                    Direction::Right => {renderable.glyph = rltk::to_cp437('►')},
                    Direction::DownRight => {renderable.glyph = rltk::to_cp437('┘')},
                    Direction::Down => {renderable.glyph = rltk::to_cp437('▼')},
                    Direction::DownLeft => {renderable.glyph = rltk::to_cp437('└')},
                    Direction::Left => {renderable.glyph = rltk::to_cp437('◄')},
                    Direction::UpLeft => {renderable.glyph = rltk::to_cp437('┌')},
                }
            }

            intent.action = Action::Idle;
        },
        _ => ()
    }
}