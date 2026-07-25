use rltk::{RandomNumberGenerator, BaseMap, Algorithm2D, Point};
use std::cmp::{max, min};
use crate::Rect;
use crate::entity::Pawn;
use crate::item::Item;
use super::{GameError, Error};

const BLOCK_SIZE: usize = 100;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    OpenDoor,
    ClosedDoor
}

pub struct Block {
    pub dimensions: Rect
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub blocks: Vec<Block>,
    pub width: usize,
    pub height: usize,
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

    pub fn blocked_idx(&self, index: usize) -> bool {
        match self.tiles[index] {
            TileType::Floor => self.pawns[index].is_some(),
            TileType::Wall => true,
            TileType::OpenDoor => self.pawns[index].is_some(),
            TileType::ClosedDoor => true
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
            return map.tiles[idx] == TileType::Floor && map.items[idx].is_none();
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

/***
 * Scratchpad for map creation algorithm
 * One function for indoor maps, one for outdoor
 * Or - one function for building buildings, another for conjoining buildings into one large indoor map?
 * Indoor maps:
 * Data that might be needed:
 *  exits
 *  size
 *  list of types of rooms, with amounts for each
 *  list of possible items
 *  list of possible enemies
 * Rooms may need the following data:
 *  types of occupants (civilians, guards, vehicles, etc) - to group similar usage together
 *  name (hangar, bedroom, control room etc)
 *  value, to set up guard routes
 *  special items or enemies
 *  dimensions and shape
 *  impassable areas
 * Idea for algorithm:
 *  pick room type at random and place one
 *  find rooms with similar occupants and place adjacent, create doors between
 *  if high value, surround with corridors for guarding, possibly create one long corridor with roomcluster at end
 *  create room cluster along corridor
 *  repeat
 */
    pub fn new_game_map(size_in_blocks: usize) -> Map {
        let map_width = size_in_blocks * BLOCK_SIZE;
        let map_height = size_in_blocks * BLOCK_SIZE;
        let tile_count = map_width * map_height;
        let mut map = Map {
            tiles: vec![TileType::Floor; tile_count],
            rooms: vec!(),
            blocks: vec!(),
            width: map_width,
            height: map_height,
            revealed_tiles: vec![true; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        };

        for x in 0..size_in_blocks {
            for y in 0..size_in_blocks {
                map.create_block(x, y);
            }
        }

        return map;
    }

    pub fn new_empty_map(map_width: usize, map_height: usize) -> Map {
        let tile_count = map_width * map_height;
        Map {
            tiles: vec![TileType::Floor; tile_count],
            rooms: vec!(),
            blocks: vec!(),
            width: map_width,
            height: map_height,
            revealed_tiles: vec![true; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        }
    }

    fn create_block(&mut self, index_x: usize, index_y: usize) {
        let x1 = index_x * BLOCK_SIZE;
        let x2 = (index_x + 1) * BLOCK_SIZE - 1;
        let y1 = index_y * BLOCK_SIZE;
        let y2 = (index_y + 1) * BLOCK_SIZE - 1;

        for x in x1..=x2 {
            for y in y1..=y2 {
                let idx = self.xy_idx(x as i32, y as i32);
                if x == x1
                    || x == x2
                    || y == y1
                    || y == y2 {
                        self.tiles[idx] = TileType::Wall;
                } else {
                    self.tiles[idx] = TileType::Floor;
                }
            }
        }

        let mut rng = RandomNumberGenerator::new();

        let max_buildings = 10;
        let room_min_size = 4;
        let building_max_size = 30;

        let mut buildings = vec!();

        for _ in 0 .. max_buildings {
            let building_width = rng.range(room_min_size, building_max_size);
            let building_height = rng.range(room_min_size, building_max_size);
            let building_left = rng.range(x1, x2 - building_max_size - 1);
            let building_top = rng.range(y1, y2 - building_max_size - 1);

            let building = Rect::new(building_left as i32, building_top as i32, building_width as i32, building_height as i32);
            let mut ok = true;
            {
                for j in 0..buildings.len() {
                    if building.intersect(&buildings[j]) {
                        ok = false;
                    }
                }
                if ok {
                    buildings.push(building);
                }
                else {
                    continue;
                }
            }

            let mut rooms = self.split_room(building, room_min_size as i32, &mut rng);

            if rooms.len() == 1 {
                self.create_random_door(rooms[0], &mut rng, vec!(Side::Top, Side::Bottom, Side::Left, Side::Right));
            }

            self.rooms.append(&mut rooms);
        }
    }

    fn create_room_walls(&mut self, room: Rect) {
        for x in room.x1..=room.x2 {
            for y in room.y1..=room.y2 {
                let idx = self.xy_idx(x, y);
                if x == room.x1
                || x == room.x2
                || y == room.y1
                || y == room.y2 {
                    self.tiles[idx] = TileType::Wall;
                } else {
                    self.tiles[idx] = TileType::Floor;
                }
            }
        }
    }
    
    fn create_random_door(&mut self, room: Rect, rng: &mut rltk::RandomNumberGenerator, allowed_dirs: Vec<Side>) {
        let chosen_dir = rng.range(0, allowed_dirs.len());
        match allowed_dirs[chosen_dir] {
            Side::Top => {
                let x = rng.range(room.x1 + 1, room.x2);
                let index = self.xy_idx(x, room.y1);
                self.tiles[index] = TileType::ClosedDoor;
            },
            Side::Bottom => {
                let x = rng.range(room.x1 + 1, room.x2);
                let index = self.xy_idx(x, room.y2);
                self.tiles[index] = TileType::ClosedDoor;
            },
            Side::Left => {
                let y = rng.range(room.y1 + 1, room.y2);
                let index = self.xy_idx(room.x1, y);
                self.tiles[index] = TileType::ClosedDoor;
            },
            Side::Right => {
                let y = rng.range(room.y1 + 1, room.y2);
                let index = self.xy_idx(room.x2, y);
                self.tiles[index] = TileType::ClosedDoor;
            },
        }
    }

    fn split_room(&mut self, room: Rect, min_size: i32, rng: &mut rltk::RandomNumberGenerator) -> Vec<Rect> {
        let mut result = vec!();

        let mut allowed_dirs = vec!();
        if room.x2 - room.x1 > 2 * min_size {
            allowed_dirs.push(Dirs::Horizontal);
        }
        if room.y2 - room.y1 > 2 * min_size {
            allowed_dirs.push(Dirs::Vertical);
        }

        let chosen_dir = rng.range(0, allowed_dirs.len() + 1);
        if chosen_dir == allowed_dirs.len() {
            self.create_room_walls(room);
            result.push(room);
            return result;
        }
        match allowed_dirs[chosen_dir] {
            Dirs::Vertical => {
                let split_point = rng.range(room.y1 + min_size, room.y2 - min_size);
                let top_room = Rect::new(room.x1, room.y1, room.x2 - room.x1, split_point - room.y1);
                let bottom_room = Rect::new(room.x1, top_room.y2, room.x2 - room.x1, room.y2 - top_room.y2);

                result.append(&mut self.split_room(top_room, min_size, rng));
                result.append(&mut self.split_room(bottom_room, min_size, rng));
                self.create_random_door(top_room, rng, vec!(Side::Bottom));
                self.create_random_door(top_room, rng, vec!(Side::Top, Side::Left, Side::Right));
                self.create_random_door(bottom_room, rng, vec!(Side::Bottom, Side::Left, Side::Right));
            },
            Dirs::Horizontal => {
                let split_point = rng.range(room.x1 + min_size, room.x2 - min_size);
                let left_room = Rect::new(room.x1, room.y1, split_point - room.x1, room.y2 - room.y1);
                let right_room = Rect::new(left_room.x2, room.y1, room.x2 - left_room.x2, room.y2 - room.y1);

                result.append(&mut self.split_room(left_room, min_size, rng));
                result.append(&mut self.split_room(right_room, min_size, rng));
                self.create_random_door(left_room, rng, vec!(Side::Right));
                self.create_random_door(left_room, rng, vec!(Side::Top, Side::Left, Side::Bottom));
                self.create_random_door(right_room, rng, vec!(Side::Bottom, Side::Bottom, Side::Right));
            }
        }

        result
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
    fn is_opaque(&self, idx: usize) -> bool {
        match self.tiles[idx as usize] {
            TileType::Wall => true,
            TileType::Floor => false,
            TileType::OpenDoor => false,
            TileType::ClosedDoor => true,
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