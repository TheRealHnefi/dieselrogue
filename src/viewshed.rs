use rltk::{Point, BaseMap};
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
        compute_fov(pos, range, facing, effective_fov, map, &mut self.visible_tiles);
    }
}

// Per-octant coordinate transforms: (dx_row, dx_col, dy_row, dy_col)
// Tile offset from origin: dx = dx_row*row + dx_col*col, dy = dy_row*row + dy_col*col
// row ∈ [1, range] = distance; col ∈ [0, row]; slope = col/row ∈ [0, 1]
// col=0 is the octant's primary axis, col=row is the 45° boundary.
// Y increases downward (screen coordinates).
const OCTANT_TRANSFORMS: [(i32, i32, i32, i32); 8] = [
    ( 1,  0,  0, -1), // 0: E→NE  (right, trending up)
    ( 0,  1, -1,  0), // 1: N→NE  (up, trending right)
    ( 0, -1, -1,  0), // 2: N→NW  (up, trending left)
    (-1,  0,  0, -1), // 3: W→NW  (left, trending up)
    (-1,  0,  0,  1), // 4: W→SW  (left, trending down)
    ( 0, -1,  1,  0), // 5: S→SW  (down, trending left)
    ( 0,  1,  1,  0), // 6: S→SE  (down, trending right)
    ( 1,  0,  0,  1), // 7: E→SE  (right, trending down)
];

// Maps each (facing, fov) pair to the octants that fall entirely inside the viewing cone.
// Cardinal facings align with octant boundaries, so the cone is covered by exactly 2/4/6 full
// octants with no per-tile trimming needed.  Diagonal facings use 4/6 octants for the same reason.
fn octants_for(facing: Direction, fov: &FieldOfView) -> &'static [usize] {
    match fov {
        FieldOfView::Fov360 => &[0, 1, 2, 3, 4, 5, 6, 7],
        FieldOfView::Fov90 => match facing {
            Direction::Right     => &[0, 7],
            Direction::UpRight   => &[0, 1],
            Direction::Up        => &[1, 2],
            Direction::UpLeft    => &[2, 3],
            Direction::Left      => &[3, 4],
            Direction::DownLeft  => &[4, 5],
            Direction::Down      => &[5, 6],
            Direction::DownRight => &[6, 7],
        },
        FieldOfView::Fov180 => match facing {
            Direction::Right     => &[0, 1, 6, 7],
            Direction::UpRight   => &[0, 1, 2, 7],
            Direction::Up        => &[0, 1, 2, 3],
            Direction::UpLeft    => &[1, 2, 3, 4],
            Direction::Left      => &[2, 3, 4, 5],
            Direction::DownLeft  => &[3, 4, 5, 6],
            Direction::Down      => &[4, 5, 6, 7],
            Direction::DownRight => &[0, 5, 6, 7],
        },
        FieldOfView::Fov270 => match facing {
            Direction::Right     => &[0, 1, 2, 5, 6, 7],
            Direction::UpRight   => &[0, 1, 2, 3, 6, 7],
            Direction::Up        => &[0, 1, 2, 3, 4, 7],
            Direction::UpLeft    => &[0, 1, 2, 3, 4, 5],
            Direction::Left      => &[1, 2, 3, 4, 5, 6],
            Direction::DownLeft  => &[2, 3, 4, 5, 6, 7],
            Direction::Down      => &[0, 3, 4, 5, 6, 7],
            Direction::DownRight => &[0, 1, 4, 5, 6, 7],
        },
    }
}

fn compute_fov(
    origin: Point,
    range: i32,
    facing: Direction,
    effective_fov: &FieldOfView,
    map: &Map,
    out: &mut Vec<Point>,
) {
    out.clear();
    out.push(origin);
    let range_sq = range * range;
    for &oct in octants_for(facing, effective_fov) {
        cast_light_octant(origin, range, range_sq, OCTANT_TRANSFORMS[oct], map, out);
    }
}

// Scans one octant using recursive shadowcasting.
// Tracks shadow intervals in slope-space [0, 1] accumulated from opaque tiles.
// A tile's shadow spans [(col-0.5)/row, (col+0.5)/row]; visibility is tested at center slope col/row.
//
// Key invariant: opaque tiles ALWAYS emit their shadow, even when they are themselves inside an
// existing shadow interval.  Skipping shadow emission for in-shadow walls causes "holes".
fn cast_light_octant(
    origin: Point,
    range: i32,
    range_sq: i32,
    t: (i32, i32, i32, i32),
    map: &Map,
    out: &mut Vec<Point>,
) {
    let (dx_row, dx_col, dy_row, dy_col) = t;
    let map_w = map.width as i32;
    let map_h = map.height as i32;
    let mut shadows: Vec<(f32, f32)> = Vec::with_capacity(8);

    for row in 1..=range {
        if shadows.len() == 1 && shadows[0].0 <= 0.0 && shadows[0].1 >= 1.0 {
            break; // entire octant is in shadow
        }
        for col in 0..=row {
            let dx = dx_row * row + dx_col * col;
            let dy = dy_row * row + dy_col * col;
            let x = origin.x + dx;
            let y = origin.y + dy;
            if x < 0 || y < 0 || x >= map_w || y >= map_h {
                continue;
            }

            let slope     = col as f32 / row as f32;
            let in_shadow = shadows.iter().any(|&(lo, hi)| lo <= slope && slope <= hi);
            let opaque    = map.is_opaque(map.xy_idx(x, y));

            if opaque {
                // Always emit shadow from opaque tiles, regardless of whether the tile itself
                // is in shadow.  This keeps the shadow cascade intact through thick walls.
                let slo = ((col as f32 - 0.5) / row as f32).max(0.0);
                let shi =  (col as f32 + 0.5) / row as f32;
                add_shadow(&mut shadows, slo, shi);
                if !in_shadow && dx * dx + dy * dy <= range_sq {
                    out.push(Point::new(x, y)); // visible wall face
                }
            } else {
                if in_shadow {
                    continue;
                }
                // Block transparent tiles at the octant boundary (col==row==1) when both
                // immediately-adjacent axis-aligned tiles are opaque.  Each wall casts shadow
                // [0, 0.5] in its own octant but never reaches slope 1.0, so without this check
                // a transparent tile wedged in the inside corner of two walls stays visible.
                if col == row && row == 1 {
                    let perp_x = origin.x + dx_row;
                    let perp_y = origin.y + dy_row;
                    let adj_x  = origin.x + dx_col;
                    let adj_y  = origin.y + dy_col;
                    let perp_opaque = perp_x >= 0 && perp_y >= 0 && perp_x < map_w && perp_y < map_h
                        && map.is_opaque(map.xy_idx(perp_x, perp_y));
                    let adj_opaque  = adj_x  >= 0 && adj_y  >= 0 && adj_x  < map_w && adj_y  < map_h
                        && map.is_opaque(map.xy_idx(adj_x, adj_y));
                    if perp_opaque && adj_opaque {
                        add_shadow(&mut shadows, 0.5, 1.5);
                        continue;
                    }
                }
                if dx * dx + dy * dy <= range_sq {
                    out.push(Point::new(x, y));
                }
            }
        }
    }
}

// Inserts a new shadow interval and merges any overlapping existing ones.
fn add_shadow(shadows: &mut Vec<(f32, f32)>, new_lo: f32, new_hi: f32) {
    let mut lo = new_lo;
    let mut hi = new_hi;
    let mut i = 0;
    while i < shadows.len() {
        let (slo, shi) = shadows[i];
        if slo <= hi && shi >= lo {
            lo = lo.min(slo);
            hi = hi.max(shi);
            shadows.swap_remove(i);
        } else {
            i += 1;
        }
    }
    shadows.push((lo, hi));
}
