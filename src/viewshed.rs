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
    Fov270,
    Fov360,
}

impl FieldOfView {
    /// Minimum dot product of (facing, normalised_dir_to_point) for the point to be visible.
    /// Points with a lower value are in the blind spot.
    pub fn min_visible_dot(&self) -> f32 {
        match self {
            FieldOfView::Fov90  =>  0.707, // cos 45°
            FieldOfView::Fov180 =>  0.0,   // cos 90°
            FieldOfView::Fov270 => -0.707, // cos 135°
            FieldOfView::Fov360 => -1.0,   // always visible
        }
    }
}

impl Viewshed {
    pub fn new(range: u32, fov: FieldOfView) -> Self {
        Self {
            range: range as i32,
            fov: fov,
            visible_tiles: vec!()
        }
    }

    pub fn update(&mut self, pos: Point, facing: Direction, range: i32, effective_fov: &FieldOfView, map: &Map) {
        self.visible_tiles = field_of_view(pos, range, map);
        match effective_fov {
            FieldOfView::Fov90  => self.cull_90(pos, facing, map),
            FieldOfView::Fov180 => self.cull_180(pos, facing, map),
            FieldOfView::Fov270 => self.cull_270(pos, facing, map),
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

    // Retains everything except the 90° blind spot directly behind the entity.
    // Each arm of the retain condition is the negation of the opposite direction's cull_90.
    // The entity's own tile (*p == pos) is always kept because both sides of the OR evaluate
    // to false at dx=0, dy=0.
    fn cull_270(&mut self, pos: Point, facing: Direction, map: &Map) {
        match facing {
            Direction::Up => {
                // blind spot = Down-cone: dx <= dy && dx >= -dy
                self.visible_tiles.retain(|p| {
                    let (dx, dy) = (p.x - pos.x, p.y - pos.y);
                    (dx > dy || dx < -dy || (dx == 0 && dy == 0))
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32
                });
            },
            Direction::UpRight => {
                // blind spot = DownLeft-cone: px <= pos.x && py >= pos.y
                self.visible_tiles.retain(|p|
                    (p.x > pos.x || p.y < pos.y || *p == pos)
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::Right => {
                // blind spot = Left-cone: dx <= -dy && dx <= dy
                self.visible_tiles.retain(|p| {
                    let (dx, dy) = (p.x - pos.x, p.y - pos.y);
                    (dx > -dy || dx > dy || (dx == 0 && dy == 0))
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32
                });
            },
            Direction::DownRight => {
                // blind spot = UpLeft-cone: px <= pos.x && py <= pos.y
                self.visible_tiles.retain(|p|
                    (p.x > pos.x || p.y > pos.y || *p == pos)
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::Down => {
                // blind spot = Up-cone: dx >= dy && dx <= -dy
                self.visible_tiles.retain(|p| {
                    let (dx, dy) = (p.x - pos.x, p.y - pos.y);
                    (dx < dy || dx > -dy || (dx == 0 && dy == 0))
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32
                });
            },
            Direction::DownLeft => {
                // blind spot = UpRight-cone: px >= pos.x && py <= pos.y
                self.visible_tiles.retain(|p|
                    (p.x < pos.x || p.y > pos.y || *p == pos)
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
            Direction::Left => {
                // blind spot = Right-cone: dx >= -dy && dx >= dy
                self.visible_tiles.retain(|p| {
                    let (dx, dy) = (p.x - pos.x, p.y - pos.y);
                    (dx < -dy || dx < dy || (dx == 0 && dy == 0))
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32
                });
            },
            Direction::UpLeft => {
                // blind spot = DownRight-cone: px >= pos.x && py >= pos.y
                self.visible_tiles.retain(|p|
                    (p.x < pos.x || p.y < pos.y || *p == pos)
                    && p.x >= 0 && p.x < map.width as i32 && p.y > 0 && p.y < map.height as i32);
            },
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