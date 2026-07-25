use rltk::{Point, field_of_view};
use crate::components::*;
use crate::Map;

#[derive(Clone)]
pub struct Viewshed {
    pub range: i32,
    pub fov: FieldOfView,
    pub visible_tiles: Vec<Point>
}

#[derive(Clone)]
pub enum FieldOfView {
    Fov90,
    Fov180,
    Fov360
}

impl Viewshed {
    pub fn new(range: u32, fov: FieldOfView) -> Self {
        Self {
            range: range as i32,
            fov: fov,
            visible_tiles: vec!()
        }
    }

    pub fn update(&mut self, pos: Point, facing: Direction, map: &Map) {
        self.visible_tiles = field_of_view(pos, self.range, map);
        match self.fov {
            FieldOfView::Fov90 => self.cull_90(pos, facing, map),
            FieldOfView::Fov180 => self.cull_180(pos, facing, map),
            FieldOfView::Fov360 => ()
        }
    }

    fn cull_90(&mut self, pos: Point, facing: Direction, map: &Map) {
        match facing {
            Direction::Up => {
                self.visible_tiles.retain(|p|
                    p.x - pos.x >= p.y - pos.y
                    && p.x - pos.x <= -(p.y - pos.y)
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::UpRight => {
                self.visible_tiles.retain(|p|
                    p.x >= pos.x
                    && p.y <= pos.y
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::Right => {
                self.visible_tiles.retain(|p|
                    p.x - pos.x >= -(p.y - pos.y)
                    && p.x - pos.x >= p.y - pos.y
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::DownRight => {
                self.visible_tiles.retain(|p|
                    p.x >= pos.x
                    && p.y >= pos.y
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::Down => {
                self.visible_tiles.retain(|p|
                    p.x - pos.x <= p.y - pos.y
                    && p.x - pos.x >= -(p.y - pos.y)
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::DownLeft => {
                self.visible_tiles.retain(|p|
                    p.x <= pos.x
                    && p.y >= pos.y
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::Left => {
                self.visible_tiles.retain(|p|
                    p.x - pos.x <= -(p.y - pos.y)
                    && p.x - pos.x <= p.y - pos.y
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::UpLeft => {
                self.visible_tiles.retain(|p|
                    p.x <= pos.x
                    && p.y <= pos.y
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            }                
        }
    }

    fn cull_180(&mut self, pos: Point, facing: Direction, map: &Map) {
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