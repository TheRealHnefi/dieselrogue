use rltk::{RandomNumberGenerator, BaseMap, Algorithm2D, Point};
use std::cmp::{max, min};
use crate::Rect;
use crate::entity::Pawn;
use crate::item::Item;
use super::{GameError, Error};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    OpenDoor,
    ClosedDoor
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: usize,
    pub height: usize,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub pawns: Vec<Option<Pawn>>,
    pub items: Vec<Option<Item>>,
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

    pub fn nearest_free_item_position(&self, pos: Point) -> Result<Point, GameError> {
        let mut index = self.xy_idx(pos.x, pos.y);

        fn is_free(map: &Map, idx: usize) -> bool {
            return map.tiles[idx] == TileType::Floor && map.items[idx].is_none();
        }

        if is_free(self, index) {
            return Ok(pos);
        }

        // This should be replaced by a spiral search for efficiency. But meh.
        for distance in 1..=5 {
            for dx in -distance..=distance {
                if dx + pos.x > self.width as i32 || pos.x - dx < 0 {
                    continue;
                }
                for dy in -distance..=distance {
                    if dy + pos.y > self.height as i32 || pos.y - dy < 0 {
                        continue;
                    }
                    index = self.xy_idx(pos.x + dx, pos.y + dy);
                    if is_free(self, index) {
                        return Ok(Point {x: pos.x + dx, y: pos.y + dy});
                    }
                }
            }
        }

        return Err(
            GameError {
                message: String::from("Could not find open spot for item"),
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
    pub fn new_empty_map(map_width: usize, map_height: usize) -> Map {
        let tile_count = map_width * map_height;
        Map {
            tiles: vec![TileType::Floor; tile_count],
            rooms: Vec::new(),
            width: map_width,
            height: map_height,
            revealed_tiles: vec![true; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        }
    }

    pub fn new_map_buildings_outdoors(map_width: usize, map_height: usize) -> Map {
        let tile_count = map_width * map_height;
        let mut map = Map {
            tiles: vec![TileType::Floor; tile_count],
            rooms: Vec::new(),
            width: map_width,
            height: map_height,
            revealed_tiles: vec![true; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        };

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

        fn create_room(map: &mut Map, room: Rect) -> Rect {
            for x in room.x1..=room.x2 {
                for y in room.y1..=room.y2 {
                    let idx = map.xy_idx(x, y);
                    if x == room.x1
                        || x == room.x2
                        || y == room.y1
                        || y == room.y2 {
                        map.tiles[idx] = TileType::Wall;
                    } else {
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            room
        }
        
        fn create_door(map: &mut Map, room: Rect, rng: &mut rltk::RandomNumberGenerator, allowed_dirs: Vec<Side>) {
            let chosen_dir = rng.range(0, allowed_dirs.len());
            match allowed_dirs[chosen_dir] {
                Side::Top => {
                    let x = rng.range(room.x1 + 1, room.x2);
                    let index = map.xy_idx(x, room.y1);
                    map.tiles[index] = TileType::OpenDoor;
                },
                Side::Bottom => {
                    let x = rng.range(room.x1 + 1, room.x2);
                    let index = map.xy_idx(x, room.y2);
                    map.tiles[index] = TileType::OpenDoor;
                },
                Side::Left => {
                    let y = rng.range(room.y1 + 1, room.y2);
                    let index = map.xy_idx(room.x1, y);
                    map.tiles[index] = TileType::OpenDoor;
                },
                Side::Right => {
                    let y = rng.range(room.y1 + 1, room.y2);
                    let index = map.xy_idx(room.x2, y);
                    map.tiles[index] = TileType::OpenDoor;
                },
            }
        }

        fn split_room(map: &mut Map, room: Rect, min_size: i32, rng: &mut rltk::RandomNumberGenerator) -> Vec<Rect> {
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
                result.push(create_room(map, room));
                return result;
            }
            match allowed_dirs[chosen_dir] {
                Dirs::Vertical => {
                    let split_point = rng.range(room.y1 + min_size, room.y2 - min_size);
                    let top_room = Rect::new(room.x1, room.y1, room.x2 - room.x1, split_point - room.y1);
                    let bottom_room = Rect::new(room.x1, top_room.y2, room.x2 - room.x1, room.y2 - top_room.y2);

                    result.append(&mut split_room(map, top_room, min_size, rng));
                    result.append(&mut split_room(map, bottom_room, min_size, rng));
                    create_door(map, top_room, rng, vec!(Side::Bottom));
                    create_door(map, top_room, rng, vec!(Side::Top, Side::Left, Side::Right));
                    create_door(map, bottom_room, rng, vec!(Side::Bottom, Side::Left, Side::Right));
                },
                Dirs::Horizontal => {
                    let split_point = rng.range(room.x1 + min_size, room.x2 - min_size);
                    let left_room = Rect::new(room.x1, room.y1, split_point - room.x1, room.y2 - room.y1);
                    let right_room = Rect::new(left_room.x2, room.y1, room.x2 - left_room.x2, room.y2 - room.y1);

                    result.append(&mut split_room(map, left_room, min_size, rng));
                    result.append(&mut split_room(map, right_room, min_size, rng));
                    create_door(map, left_room, rng, vec!(Side::Right));
                    create_door(map, left_room, rng, vec!(Side::Top, Side::Left, Side::Bottom));
                    create_door(map, right_room, rng, vec!(Side::Bottom, Side::Bottom, Side::Right));
                }
            }

            result
        }

        let mut rng = RandomNumberGenerator::new();

        let max_buildings = 10;
        let room_min_size = 4;
        let building_max_size = 30;

        let mut buildings = vec!();

        for _ in 0 .. max_buildings {
            let building_width = rng.range(building_max_size - room_min_size, building_max_size);
            let building_height = rng.range(building_max_size - room_min_size, building_max_size);
            let building_left = rng.range(1, map_width as i32 - building_max_size - 1);
            let building_top = rng.range(1, map_height as i32 - building_max_size - 1);

            let building = Rect::new(building_left, building_top, building_width, building_height);
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

            let mut rooms = split_room(&mut map, building, room_min_size, &mut rng);

            if rooms.len() == 1 {
                create_door(&mut map, rooms[0], &mut rng, vec!(Side::Top, Side::Bottom, Side::Left, Side::Right));
            }

            map.rooms.append(&mut rooms);
        }

        map
    }

    pub fn new_map_rooms_and_corridors(width: usize, height: usize) -> Map {
        let tile_count = width * height;
        let mut map = Map {
            tiles: vec![TileType::Wall; tile_count],
            rooms: Vec::new(),
            width: width,
            height: height,
            revealed_tiles: vec![false; tile_count],
            visible_tiles: vec![false; tile_count],
            pawns: vec![None; tile_count],
            items: vec![None; tile_count]
        };

        let mut rng = RandomNumberGenerator::new();

        const MAX_ROOMS: i32 = 60;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 20;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x =  rng.roll_dice(1, map.width as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, map.height as i32 - h - 1) - 1;
            let new_room = Rect::new(x, y , w, h);

            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                    break;
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();

                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2) ..= max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.tiles.len() {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..= max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.tiles.len() {
                self.tiles[idx as usize] = TileType::Floor;
            }
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