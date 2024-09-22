use rltk::{BaseMap, Algorithm2D, Point};
use std::cmp::{max, min};
use crate::entity::*;
use crate::item::Item;
use crate::tile::TileType;
use super::{GameError, Error};

const BLOCK_SIZE: usize = 32;

pub struct Map {
    pub width: usize,
    pub height: usize,

    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub pawns: Vec<Option<Pawn>>,
    pub items: Vec<Option<Item>>,
}

enum Side {
    Top,
    Bottom,
    Left,
    Right
}

enum Dirs {
    Vertical,
    Horizontal
}

impl Map {
    pub fn pos_idx(&self, pos: Point) -> usize {
        self.xy_idx(pos.x, pos.y)
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width) + x as usize
    }

    pub fn idx_pos(&self, idx: usize) -> Point {
        let y = idx / self.width;
        let x = idx % self.width;
        
        Point {x: x as i32, y: y as i32}
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        let index = self.xy_idx(x, y);
        self.blocked_idx(index)
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        let index = self.xy_idx(x, y);
        return self.tiles[index];
    }

    pub fn blocked_idx(&self, index: usize) -> bool {
        match self.tiles[index] {
            TileType::Floor => self.pawns[index].is_some(),
            TileType::Ground => self.pawns[index].is_some(),
            TileType::Wall => true,
            TileType::Doorway => self.pawns[index].is_some(),
        }
    }

    pub fn get_entities_in_vicinity(&self, center: Point, radius: i32) -> Vec<usize> {
        let min_x = max(center.x - radius, 0);
        let max_x = min(center.x + radius, self.width as i32);
        let min_y = max(center.y - radius, 0);
        let max_y = min(center.y + radius, self.height as i32);
        let mut result = vec!();
        for x in min_x..max_x {
            for y in min_y..max_y {
                let index = self.xy_idx(x, y);
                match &self.pawns[index] {
                    Some(pawn) => result.push(pawn.entity_id),
                    None => ()
                }
            }
        }

        return result;
    }

    pub fn nearest_free_item_position(&self, pos: Point) -> Result<Point, GameError> {

        fn is_free(map: &Map, idx: usize) -> bool {
            return (map.tiles[idx] == TileType::Floor || map.tiles[idx] == TileType::Ground)
            && map.items[idx].is_none();
        }

        return self.find_nearest_tile(pos, 5, is_free);
    }

    pub fn nearest_free_pawn_position(&self, pos: Point) -> Result<Point, GameError> {

        fn is_free(map: &Map, idx: usize) -> bool {
            return !map.blocked_idx(idx);
        }

        return self.find_nearest_tile(pos, 5, is_free);
    }

    fn find_nearest_tile(&self, pos: Point, radius: usize, good_tile: fn (&Map, usize) -> bool) -> Result<Point, GameError> {
        let mut index = self.xy_idx(pos.x, pos.y);

        if good_tile(&self, index) {
            return Ok(pos);
        }

        // This should be replaced by a spiral search for efficiency. But meh.
        for distance in 1..=radius as i32 {
            for dx in -distance..=distance {
                if dx + pos.x > self.width as i32 || pos.x - dx < 0 {
                    continue;
                }
                for dy in -distance..=distance {
                    if dy + pos.y > self.height as i32 || pos.y - dy < 0 {
                        continue;
                    }
                    index = self.xy_idx(pos.x + dx, pos.y + dy);
                    if good_tile(&self, index) {
                        return Ok(Point {x: pos.x + dx, y: pos.y + dy});
                    }
                }
            }
        }

        return Err(
            GameError {
                message: String::from("Could not find open spot"),
                error: Error::UnsolvableSituation
        });
    }

    pub fn new_game_map(size_in_blocks: usize) -> Map {
        let map_width = size_in_blocks * BLOCK_SIZE;
        let map_height = size_in_blocks * BLOCK_SIZE;
        let tile_count = map_width * map_height;
        let mut map = Map {
            tiles: vec![TileType::Ground; tile_count],
            width: map_width,
            height: map_height,
            revealed_tiles: vec![true; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        };

        return map;
    }

    pub fn new_empty_map(map_width: usize, map_height: usize) -> Map {
        let tile_count = map_width * map_height;
        Map {
            tiles: vec![TileType::Ground; tile_count],
            width: map_width,
            height: map_height,
            revealed_tiles: vec![true; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width as i32 - 1 || y < 1 || y > self.height as i32 - 1 {
            return false;
        }
        !self.blocked(x, y)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, index: usize) -> bool {
        match self.tiles[index] {
            TileType::Wall => true,
            TileType::Floor => false,
            TileType::Ground => false,
            TileType::Doorway => {
                match &self.pawns[index] {
                    Some(pawn) => pawn.kind == EntityKind::Door,
                    None => false
                }
            },
        }
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width as i32;
        let y = idx as i32 / self.width as i32;
        let w = self.width as usize;

        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };
    
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }
    
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}