use rltk::{RandomNumberGenerator, BaseMap, Algorithm2D, Point};
use std::cmp::{max, min};
use crate::Rect;
use crate::entity::Pawn;

const MAPWIDTH: usize = 80;
const MAPHEIGHT: usize = 43;
const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor
}

//#[derive(Default, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub pawns: Vec<Option<Pawn>>
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * MAPWIDTH) + x as usize
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        let index = self.xy_idx(x, y);
        return
            self.tiles[index] != TileType::Floor || 
            self.pawns[index].is_some()
    }

    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            pawns: vec![None; MAPCOUNT]
        };

        let mut rng = RandomNumberGenerator::new();

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x =  rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
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
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2) ..= max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
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
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
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