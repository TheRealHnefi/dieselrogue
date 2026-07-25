use rltk::{Point, field_of_view};
use crate::components::*;
use crate::Map;

#[derive(Clone)]
pub struct Viewshed {
    pub dirty: bool,
    pub range: i32,
    pub visible_tiles: Vec<Point>
}

impl Viewshed {
    pub fn new() -> Self {
        Self {
            dirty: true,
            range: 10,
            visible_tiles: vec!()
        }
    }

    pub fn update(&mut self, pos: Point, facing: Direction, map: &Map) {
        if self.dirty {
            self.dirty = false;

            self.visible_tiles = field_of_view(pos, self.range, map);
            match facing {
                Direction::Up => {
                    self.visible_tiles.retain(|p| p.y <= pos.y &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::UpRight => {
                    self.visible_tiles.retain(|p| p.x - pos.x >= p.y - pos.y &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::Right => {
                    self.visible_tiles.retain(|p| p.x >= pos.x &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::DownRight => {
                    self.visible_tiles.retain(|p| p.x - pos.x >= -(p.y - pos.y) &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::Down => {
                    self.visible_tiles.retain(|p| p.y >= pos.y &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::DownLeft => {
                    self.visible_tiles.retain(|p| p.x - pos.x <= p.y - pos.y &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::Left => {
                    self.visible_tiles.retain(|p| p.x <= pos.x &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                },
                Direction::UpLeft => {
                    self.visible_tiles.retain(|p| p.x - pos.x <= -(p.y - pos.y) &&
                        p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
                }                
            }
        }
    }
}