/// Offline block generator — writes .txt block files to resources/blocks/.
///
/// Usage:
///   cargo run --bin gen_blocks              (generate all blocks)
///   cargo run --bin gen_blocks -- --clean   (delete previously generated files first)
///   cargo run --bin gen_blocks -- --dir path/to/blocks
///
/// Each generated file produces 16 WFC variants at load time (4 rotations × 4 mirrors),
/// so one canonical form covers all symmetric orientations automatically.

use std::fs;
use std::path::PathBuf;

// ─── Tile ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum T {
    Ground,  // '.'
    Road,    // '_'
    Floor,   // '-'
    Wall,    // 'W'
    Doorway, // 'D'
    Window,  // 'x'
    Fence,   // '#'
}

impl T {
    fn ch(self) -> char {
        match self {
            T::Ground  => '.',
            T::Road    => '_',
            T::Floor   => '-',
            T::Wall    => 'W',
            T::Doorway => 'D',
            T::Window  => 'x',
            T::Fence   => '#',
        }
    }
}

// ─── Grid ────────────────────────────────────────────────────────────────────

const N: usize = 32;
type Grid = [[T; N]; N];

fn empty() -> Grid { [[T::Ground; N]; N] }

fn gs(g: &mut Grid, x: usize, y: usize, t: T) {
    if x < N && y < N { g[y][x] = t; }
}

fn gg(g: &Grid, x: usize, y: usize) -> T {
    if x < N && y < N { g[y][x] } else { T::Ground }
}

fn to_txt(g: &Grid) -> String {
    let mut s = String::with_capacity(N * (N + 1));
    for row in g.iter() {
        for &t in row.iter() { s.push(t.ch()); }
        s.push('\n');
    }
    s
}

// ─── Road constants ──────────────────────────────────────────────────────────
// Road is 6 tiles wide at cols 13–18. Note to self: add different road types?

const RL: usize = 13;          // road left edge (inclusive, 0-indexed)
const RW: usize = 6;           // road width in tiles
const RH: usize = RL + RW;     // road right edge (exclusive) = 19

// Positions relative to road for building placement
const WX1: usize = 1;          // west building left  (1-tile margin from block edge)
const WX2: usize = RL - 2;     // west building right exclusive = 11  (gap: 11,12 → road at 13)
const EX1: usize = RH + 2;     // east building left = 21  (gap: 19,20 → road ends at 18)
const EX2: usize = N - 1;      // east building right exclusive = 31

// Standard building y extents and door y (center of building).
const BY1: usize = 5;
const BY2: usize = 27;
const BDY: usize = (BY1 + BY2) / 2; // = 16

// ─── Thin path constants ─────────────────────────────────────────────────────
// 2-tile alley at x=8-9 (off-center left). The loader's 180° rotation maps
// this to x=22-23 (off-center right), so one canonical orientation is enough.
const PL:  usize = 8;            // path left edge (inclusive)
const PW:  usize = 2;            // path width in tiles
const PH:  usize = PL + PW;     // path right edge (exclusive) = 10
const PCX: usize = PL + PW / 2; // path centre x = 9

// ─── Road draw helpers ───────────────────────────────────────────────────────

/// Full vertical (N-S) road strip.
fn v_road(g: &mut Grid, cx: usize, w: usize) {
    v_road_seg(g, cx, w, 0, N);
}

/// Full horizontal (E-W) road strip.
fn h_road(g: &mut Grid, cy: usize, w: usize) {
    h_road_seg(g, cy, w, 0, N);
}

/// Vertical road segment spanning rows y0..y1.
fn v_road_seg(g: &mut Grid, cx: usize, w: usize, y0: usize, y1: usize) {
    let lo = cx.saturating_sub(w / 2);
    let hi = (lo + w).min(N);
    for x in lo..hi { for y in y0..y1.min(N) { g[y][x] = T::Road; } }
}

/// Horizontal road segment spanning cols x0..x1.
fn h_road_seg(g: &mut Grid, cy: usize, w: usize, x0: usize, x1: usize) {
    let lo = cy.saturating_sub(w / 2);
    let hi = (lo + w).min(N);
    for y in lo..hi { for x in x0..x1.min(N) { g[y][x] = T::Road; } }
}

// ─── Building helpers ────────────────────────────────────────────────────────

/// Rectangular building: Wall perimeter, Floor interior.
fn bld(g: &mut Grid, x1: usize, y1: usize, x2: usize, y2: usize) {
    for y in y1..y2.min(N) {
        for x in x1..x2.min(N) {
            let on_wall = x == x1 || x + 1 == x2 || y == y1 || y + 1 == y2;
            g[y][x] = if on_wall { T::Wall } else { T::Floor };
        }
    }
}

/// Doorway at (x, y) — overwrites any existing tile.
fn door(g: &mut Grid, x: usize, y: usize) { gs(g, x, y, T::Doorway); }

/// Windows along a vertical wall (x fixed) from ya..yb every 3 tiles.
fn win_v(g: &mut Grid, x: usize, ya: usize, yb: usize) {
    let mut y = ya + 1;
    while y + 1 < yb {
        if gg(g, x, y) == T::Wall { gs(g, x, y, T::Window); }
        y += 3;
    }
}

/// Windows along a horizontal wall (y fixed) from xa..xb every 3 tiles.
fn win_h(g: &mut Grid, y: usize, xa: usize, xb: usize) {
    let mut x = xa + 1;
    while x + 1 < xb {
        if gg(g, x, y) == T::Wall { gs(g, x, y, T::Window); }
        x += 3;
    }
}

/// Horizontal wall partition inside a building (full Wall row, door in the middle).
fn partition_h(g: &mut Grid, y: usize, x1: usize, x2: usize, door_x: usize) {
    for x in x1..x2 { gs(g, x, y, T::Wall); }
    door(g, door_x, y);
}

// ─── West/East building shorthands ───────────────────────────────────────────

/// Standard west-side building with door facing road, windows on other walls.
fn west_bld(g: &mut Grid, x1: usize, x2: usize, y1: usize, y2: usize) {
    bld(g, x1, y1, x2, y2);
    door(g, x2 - 1, (y1 + y2) / 2);   // east wall door faces road
    win_v(g, x1, y1, y2);              // west wall (facing block edge)
    win_h(g, y1, x1, x2);             // north wall
    win_h(g, y2 - 1, x1, x2);         // south wall
}

/// Standard east-side building with door facing road, windows on other walls.
fn east_bld(g: &mut Grid, x1: usize, x2: usize, y1: usize, y2: usize) {
    bld(g, x1, y1, x2, y2);
    door(g, x1, (y1 + y2) / 2);       // west wall door faces road
    win_v(g, x2 - 1, y1, y2);         // east wall (facing block edge)
    win_h(g, y1, x1, x2);             // north wall
    win_h(g, y2 - 1, x1, x2);         // south wall
}

// ─── Straight N-S road ───────────────────────────────────────────────────────

fn road_straight_v1() -> Grid {
    // Open street, no buildings.
    let mut g = empty();
    v_road(&mut g, RL + RW / 2, RW);
    g
}

fn road_straight_v2() -> Grid {
    // Standard building on the west side.
    let mut g = road_straight_v1();
    west_bld(&mut g, WX1, WX2, BY1, BY2);
    g
}

fn road_straight_v3() -> Grid {
    // Buildings on both sides.
    let mut g = road_straight_v1();
    west_bld(&mut g, WX1, WX2, BY1, BY2);
    east_bld(&mut g, EX1, EX2, BY1, BY2);
    g
}

fn road_straight_v4() -> Grid {
    // Shorter building on the west, open east side.
    let mut g = road_straight_v1();
    west_bld(&mut g, WX1, WX2, 10, 22);
    g
}

fn road_straight_v5() -> Grid {
    // Buildings on both sides with internal partition (two-room buildings).
    let mut g = road_straight_v1();
    west_bld(&mut g, WX1, WX2, BY1, BY2);
    partition_h(&mut g, (BY1 + BY2) / 2, WX1 + 1, WX2 - 1, (WX1 + WX2) / 2);
    east_bld(&mut g, EX1, EX2, BY1, BY2);
    partition_h(&mut g, (BY1 + BY2) / 2, EX1 + 1, EX2 - 1, (EX1 + EX2) / 2);
    g
}

fn road_straight_wide_v1() -> Grid {
    // 10-tile-wide road, building on west side.
    const WRL: usize = 11;
    const WRW: usize = 10;
    const WRH: usize = WRL + WRW;
    let mut g = empty();
    v_road(&mut g, WRL + WRW / 2, WRW);
    // Buildings are narrower because the road takes more space.
    if WRL > 3 {
        bld(&mut g, 1, BY1, WRL - 1, BY2);
        door(&mut g, WRL - 2, BDY);
        win_v(&mut g, 1, BY1, BY2);
        win_h(&mut g, BY1, 1, WRL - 1);
        win_h(&mut g, BY2 - 1, 1, WRL - 1);
    }
    if WRH + 2 < N {
        bld(&mut g, WRH + 1, BY1, N - 1, BY2);
        door(&mut g, WRH + 1, BDY);
        win_v(&mut g, N - 2, BY1, BY2);
        win_h(&mut g, BY1, WRH + 1, N - 1);
        win_h(&mut g, BY2 - 1, WRH + 1, N - 1);
    }
    g
}

// ─── Turn: N + E exits ───────────────────────────────────────────────────────
// Canonical NE turn; loader rotation gives NW / SE / SW variants.

fn road_turn_v1() -> Grid {
    // Turn N→E, no buildings.
    let mut g = empty();
    v_road_seg(&mut g, RL + RW / 2, RW, 0, RH); // vertical arm N to corner
    h_road_seg(&mut g, RL + RW / 2, RW, RL, N); // horizontal arm corner to E
    g
}

fn road_turn_v2() -> Grid {
    // Turn N→E, building in the SE open area.
    let mut g = road_turn_v1();
    // SE area: x=[EX1..N-1], y=[RH+1..N-1]
    let bx1 = EX1;
    let bx2 = N - 1;
    let by1 = RH + 2;
    let by2 = N - 1;
    if bx2 > bx1 + 4 && by2 > by1 + 4 {
        bld(&mut g, bx1, by1, bx2, by2);
        door(&mut g, bx1, (by1 + by2) / 2); // west wall door
        win_v(&mut g, bx2 - 1, by1, by2);
        win_h(&mut g, by1, bx1, bx2);
        win_h(&mut g, by2 - 1, bx1, bx2);
    }
    g
}

fn road_turn_v3() -> Grid {
    // Turn N→E, buildings in SE and NW open areas.
    let mut g = road_turn_v1();
    // SE area
    {
        let (bx1, bx2, by1, by2) = (EX1, N - 1, RH + 2, N - 1);
        if bx2 > bx1 + 4 && by2 > by1 + 4 {
            bld(&mut g, bx1, by1, bx2, by2);
            door(&mut g, bx1, (by1 + by2) / 2);
            win_v(&mut g, bx2 - 1, by1, by2);
            win_h(&mut g, by1, bx1, bx2);
            win_h(&mut g, by2 - 1, bx1, bx2);
        }
    }
    // NW area (x=1..RL-2, y=1..RL-2)
    {
        let (bx1, bx2, by1, by2) = (1, RL - 1, 1, RL - 1);
        if bx2 > bx1 + 4 && by2 > by1 + 4 {
            bld(&mut g, bx1, by1, bx2, by2);
            door(&mut g, bx2 - 1, (by1 + by2) / 2); // east wall door
            win_v(&mut g, bx1, by1, by2);
            win_h(&mut g, by1, bx1, bx2);
            win_h(&mut g, by2 - 1, bx1, bx2);
        }
    }
    g
}

// ─── T-junction: N + S + E exits ─────────────────────────────────────────────
// Canonical "straight NS with branch East"; rotation gives all 4 T orientations.

fn road_t_v1() -> Grid {
    // T-junction, no buildings.
    let mut g = empty();
    v_road(&mut g, RL + RW / 2, RW);
    h_road_seg(&mut g, RL + RW / 2, RW, RH - 1, N); // east branch from road to edge
    g
}

fn road_t_v2() -> Grid {
    // T-junction, building in the NW pocket.
    let mut g = road_t_v1();
    let (bx1, bx2, by1, by2) = (1, RL - 1, 1, RL - 1);
    if bx2 > bx1 + 4 && by2 > by1 + 4 {
        bld(&mut g, bx1, by1, bx2, by2);
        door(&mut g, (bx1 + bx2) / 2, by2 - 1); // south wall door
        win_v(&mut g, bx1, by1, by2);
        win_v(&mut g, bx2 - 1, by1, by2);
        win_h(&mut g, by1, bx1, bx2);
    }
    g
}

fn road_t_v3() -> Grid {
    // T-junction, buildings in both west pockets (NW and SW).
    let mut g = road_t_v1();
    // NW pocket
    {
        let (bx1, bx2, by1, by2) = (1, RL - 1, 1, RL - 1);
        if bx2 > bx1 + 4 && by2 > by1 + 4 {
            bld(&mut g, bx1, by1, bx2, by2);
            door(&mut g, (bx1 + bx2) / 2, by2 - 1);
            win_v(&mut g, bx1, by1, by2);
            win_v(&mut g, bx2 - 1, by1, by2);
            win_h(&mut g, by1, bx1, bx2);
        }
    }
    // SW pocket
    {
        let (bx1, bx2, by1, by2) = (1, RL - 1, RH + 1, N - 1);
        if bx2 > bx1 + 4 && by2 > by1 + 4 {
            bld(&mut g, bx1, by1, bx2, by2);
            door(&mut g, (bx1 + bx2) / 2, by1); // north wall door
            win_v(&mut g, bx1, by1, by2);
            win_v(&mut g, bx2 - 1, by1, by2);
            win_h(&mut g, by2 - 1, bx1, bx2);
        }
    }
    g
}

// ─── Crossing: all 4 exits ───────────────────────────────────────────────────

fn road_cross_v1() -> Grid {
    // Pure 4-way crossing.
    let mut g = empty();
    v_road(&mut g, RL + RW / 2, RW);
    h_road(&mut g, RL + RW / 2, RW);
    g
}

fn road_cross_v2() -> Grid {
    // Crossing with buildings in all four corners.
    let mut g = road_cross_v1();
    let m = 1usize; // margin inside each corner
    let corners: [(usize, usize, usize, usize); 4] = [
        (m, m,        RL - m, RL - m),  // NW
        (RH + m, m,   N - m,  RL - m),  // NE
        (m, RH + m,   RL - m, N - m),   // SW
        (RH + m, RH + m, N - m, N - m), // SE
    ];
    let face_doors = [
        // (door-x, door-y) — door on the wall closest to the road
        ((RL - m - 1), (m + RL - m) / 2),         // NW east wall
        ((RH + m),     (m + RL - m) / 2),          // NE west wall
        ((RL - m - 1), (RH + m + N - m) / 2),     // SW east wall
        ((RH + m),     (RH + m + N - m) / 2),     // SE west wall
    ];
    for (i, &(x1, y1, x2, y2)) in corners.iter().enumerate() {
        if x2 > x1 + 4 && y2 > y1 + 4 {
            bld(&mut g, x1, y1, x2, y2);
            let (dx, dy) = face_doors[i];
            door(&mut g, dx, dy);
            win_h(&mut g, y1, x1, x2);
            win_h(&mut g, y2 - 1, x1, x2);
        }
    }
    g
}

// ─── Dead-end: N exit only, road terminates inside block ─────────────────────
// Rotation gives all 4 dead-end directions.

fn road_dead_v1() -> Grid {
    // Cul-de-sac bulge.
    let mut g = empty();
    let cx = RL + RW / 2;
    v_road_seg(&mut g, cx, RW, 0, 20);
    // Wider turn-around area
    for y in 17..21 {
        let lo = RL.saturating_sub(4);
        let hi = (RH + 4).min(N);
        for x in lo..hi { g[y][x] = T::Road; }
    }
    g
}

fn road_dead_v2() -> Grid {
    // Road terminates at a building entrance.
    let mut g = empty();
    let cx = RL + RW / 2;
    v_road_seg(&mut g, cx, RW, 0, 20);
    // Building spanning the full block width at the south end.
    bld(&mut g, 2, 20, N - 2, 30);
    // Wide door aligned with road.
    for x in RL..RH { door(&mut g, x, 20); }
    win_v(&mut g, 2, 21, 29);
    win_v(&mut g, N - 3, 21, 29);
    win_h(&mut g, 29, 3, N - 3);
    g
}

fn road_dead_v3() -> Grid {
    // Road fans out into an open parking area.
    let mut g = empty();
    let cx = RL + RW / 2;
    v_road_seg(&mut g, cx, RW, 0, 15);
    for y in 15..28 {
        for x in 4..N - 4 { g[y][x] = T::Road; }
    }
    g
}

// ─── Hangar: large building, wide door, no road exit on block edges ───────────

fn road_hangar_v1() -> Grid {
    // Standard hangar with road approach from the North.
    let mut g = empty();
    let cx = RL + RW / 2;
    v_road_seg(&mut g, cx, RW, 0, 14);
    // Large hangar building
    bld(&mut g, 3, 14, N - 3, N - 2);
    // Wide entrance door (8 tiles, centered on road)
    let dlo = cx.saturating_sub(4);
    let dhi = (dlo + 8).min(N - 3);
    for x in dlo..dhi { door(&mut g, x, 14); }
    win_v(&mut g, 3, 15, N - 3);
    win_v(&mut g, N - 4, 15, N - 3);
    win_h(&mut g, N - 3, 4, N - 4);
    g
}

fn road_hangar_v2() -> Grid {
    // Wider hangar filling more of the block.
    let mut g = empty();
    let cx = RL + RW / 2;
    v_road_seg(&mut g, cx, RW, 0, 12);
    bld(&mut g, 1, 12, N - 1, N - 1);
    let dlo = cx.saturating_sub(5);
    let dhi = (dlo + 10).min(N - 1);
    for x in dlo..dhi { door(&mut g, x, 12); }
    win_v(&mut g, 1, 13, N - 2);
    win_v(&mut g, N - 2, 13, N - 2);
    win_h(&mut g, N - 2, 2, N - 2);
    g
}

fn road_hangar_v3() -> Grid {
    // Hangar with a small attached office on the side.
    let mut g = empty();
    let cx = RL + RW / 2;
    v_road_seg(&mut g, cx, RW, 0, 14);
    // Main hangar bay (east part)
    bld(&mut g, EX1 - 2, 14, N - 2, N - 2);
    let dlo = EX1 - 2;
    let dhi = (dlo + 8).min(N - 2);
    for x in dlo..dhi { door(&mut g, x, 14); }
    win_v(&mut g, N - 3, 15, N - 3);
    win_h(&mut g, N - 3, EX1 - 1, N - 3);
    // Side office (west part)
    bld(&mut g, 2, 16, RL - 1, N - 2);
    door(&mut g, RL - 2, (16 + N - 2) / 2);
    win_v(&mut g, 2, 17, N - 3);
    win_h(&mut g, N - 3, 3, RL - 2);
    g
}

// ─── Building blocks (no road exits) ─────────────────────────────────────────
// These all have Ground at block edges → compatible with most interior neighbours.

fn building_v1() -> Grid {
    // Large single building filling most of the block.
    let mut g = empty();
    bld(&mut g, 2, 2, N - 2, N - 2);
    door(&mut g, RL + RW / 2, N - 3);    // south wall entrance
    win_v(&mut g, 2, 3, N - 3);
    win_v(&mut g, N - 3, 3, N - 3);
    win_h(&mut g, 2, 3, N - 3);
    win_h(&mut g, N - 3, 3, N - 3);
    g
}

fn building_v2() -> Grid {
    // Medium building centered in the block.
    let mut g = empty();
    bld(&mut g, 6, 6, N - 6, N - 6);
    door(&mut g, RL + RW / 2, N - 7);
    win_v(&mut g, 6, 7, N - 7);
    win_v(&mut g, N - 7, 7, N - 7);
    win_h(&mut g, 6, 7, N - 7);
    win_h(&mut g, N - 7, 7, N - 7);
    g
}

fn building_v3() -> Grid {
    // Two buildings side by side.
    let mut g = empty();
    bld(&mut g, 1, 3, 15, N - 3);
    door(&mut g, 8, N - 4);
    win_v(&mut g, 1, 4, N - 4);
    win_h(&mut g, 3, 2, 14);
    win_h(&mut g, N - 4, 2, 14);

    bld(&mut g, 17, 3, N - 1, N - 3);
    door(&mut g, 24, N - 4);
    win_v(&mut g, N - 2, 4, N - 4);
    win_h(&mut g, 3, 18, N - 2);
    win_h(&mut g, N - 4, 18, N - 2);
    g
}

fn building_v4() -> Grid {
    // Building with an enclosed outdoor yard on the south side.
    let mut g = empty();
    bld(&mut g, 2, 2, N - 2, 18);
    door(&mut g, RL + RW / 2, 17);  // south wall opens into yard
    win_v(&mut g, 2, 3, 17);
    win_v(&mut g, N - 3, 3, 17);
    win_h(&mut g, 2, 3, N - 3);
    // Yard fencing south and sides
    for x in 2..N - 2 { gs(&mut g, x, 28, T::Fence); }
    for y in 18..28    { gs(&mut g, 2, y, T::Fence); }
    for y in 18..28    { gs(&mut g, N - 3, y, T::Fence); }
    door(&mut g, RL + RW / 2, 28);  // gate on south fence
    g
}

fn building_v5() -> Grid {
    // Two-room building with an internal partition.
    let mut g = empty();
    bld(&mut g, 2, 2, N - 2, N - 2);
    partition_h(&mut g, N / 2, 3, N - 3, RL + RW / 2);
    door(&mut g, RL + RW / 2, N - 3);
    win_v(&mut g, 2, 3, N - 3);
    win_v(&mut g, N - 3, 3, N - 3);
    win_h(&mut g, 2, 3, N - 3);
    win_h(&mut g, N - 3, 3, N - 3);
    g
}

fn building_v6() -> Grid {
    // Main building (north) with a detached annex (south).
    let mut g = empty();
    bld(&mut g, 2, 2, N - 2, 18);
    door(&mut g, RL + RW / 2, 17);
    win_v(&mut g, 2, 3, 17);
    win_v(&mut g, N - 3, 3, 17);
    win_h(&mut g, 2, 3, N - 3);

    bld(&mut g, 8, 21, N - 8, N - 2);
    door(&mut g, RL + RW / 2, 21);  // north wall faces main building
    win_v(&mut g, 8, 22, N - 3);
    win_v(&mut g, N - 9, 22, N - 3);
    win_h(&mut g, N - 3, 9, N - 9);
    g
}

fn building_v7() -> Grid {
    // Open field with a fence perimeter and a central building.
    let mut g = empty();
    // Outer fence
    for x in 2..N - 2 {
        gs(&mut g, x, 2, T::Fence);
        gs(&mut g, x, N - 3, T::Fence);
    }
    for y in 2..N - 2 {
        gs(&mut g, 2, y, T::Fence);
        gs(&mut g, N - 3, y, T::Fence);
    }
    door(&mut g, RL + RW / 2, N - 3);  // south gate
    // Small building inside
    bld(&mut g, 10, 8, N - 10, 20);
    door(&mut g, RL + RW / 2, 19);
    win_h(&mut g, 8, 11, N - 11);
    win_v(&mut g, 10, 9, 19);
    win_v(&mut g, N - 11, 9, 19);
    g
}

fn building_v8() -> Grid {
    // Dense layout: three small buildings.
    let mut g = empty();
    // NW small building
    bld(&mut g, 1, 1, 13, 13);
    door(&mut g, 7, 12);
    win_h(&mut g, 1, 2, 12);
    win_v(&mut g, 1, 2, 12);
    win_v(&mut g, 12, 2, 12);
    // NE small building
    bld(&mut g, 15, 1, N - 1, 13);
    door(&mut g, 23, 12);
    win_h(&mut g, 1, 16, N - 2);
    win_v(&mut g, N - 2, 2, 12);
    // South building (wide, single-room)
    bld(&mut g, 4, 16, N - 4, N - 1);
    door(&mut g, RL + RW / 2, 16);
    win_v(&mut g, 4, 17, N - 2);
    win_v(&mut g, N - 5, 17, N - 2);
    win_h(&mut g, N - 2, 5, N - 5);
    g
}

// ─── Mega-structure blocks ───────────────────────────────────────────────────
// Large buildings that span 2-4 blocks. They work exactly like road blocks:
// a fixed-width building strip exits the block only at specific edge positions,
// with Ground everywhere else. Because Floor↔Ground is INCOMPATIBLE in WFC,
// these blocks are isolated from standard road/building blocks and only connect
// to other mega-structure blocks where their Floor-filled strips align.
//
// The building strip is ML..MH-1 (x=8..23, width 16). Edge profile at an exit:
//   Ground×8 | Wall | Floor×14 | Wall | Ground×8
// This never overlaps the road strip (x=13-18 for standard roads) in terms of
// WFC compatibility, so the two networks coexist without interfering.
//
// One canonical orientation produces all 4 (or 2 for straight) via rotation.

const ML:  usize = 8;            // mega strip left wall (inclusive)
const MW:  usize = 16;           // mega strip width
const MH:  usize = ML + MW;      // mega strip right exclusive = 24
const MCX: usize = ML + MW / 2;  // strip centre x = 16

fn mega_straight_v1() -> Grid {
    // N-S building corridor, exits at both ends. 4 rotations give E-W too.
    let mut g = empty();
    for y in 0..N {
        for x in ML..MH {
            g[y][x] = if x == ML || x + 1 == MH { T::Wall } else { T::Floor };
        }
    }
    win_v(&mut g, ML,     0, N);
    win_v(&mut g, MH - 1, 0, N);
    g
}

fn mega_straight_v2() -> Grid {
    // N-S corridor partitioned into two rooms.
    let mut g = mega_straight_v1();
    for x in ML..MH { g[MCX][x] = T::Wall; }
    gs(&mut g, MCX, MCX, T::Doorway);
    g
}

fn mega_turn_v1() -> Grid {
    // L-shaped building: exits north (y=0) and east (x=31).
    // 90°CW→SE, 180°→SW, 270°CW→WN gives all four corner orientations.
    let mut g = empty();
    // Vertical arm: x=ML..MH-1, y=0..MH-1
    for y in 0..MH {
        for x in ML..MH {
            g[y][x] = T::Floor;
        }
    }
    // East arm extension: y=ML..MH-1, x=MH..N
    for y in ML..MH {
        for x in MH..N {
            g[y][x] = T::Floor;
        }
    }
    // West outer wall (full height of L)
    for y in 0..MH  { g[y][ML] = T::Wall; }
    // South outer wall (full width of L)
    for x in ML..N  { g[MH - 1][x] = T::Wall; }
    // East wall of the north arm above the turn
    for y in 0..=ML { g[y][MH - 1] = T::Wall; }
    // North face of the east arm extension
    for x in MH..N  { g[ML][x] = T::Wall; }
    // Windows
    win_v(&mut g, ML,     0, MH);
    win_h(&mut g, MH - 1, ML, N);
    win_v(&mut g, MH - 1, 0, ML + 1);
    win_h(&mut g, ML,     MH, N);
    g
}

fn mega_t_v1() -> Grid {
    // T-shaped building: exits north, south, and east. Rotations give all 4 T orientations.
    let mut g = empty();
    // Full N-S strip
    for y in 0..N {
        for x in ML..MH {
            g[y][x] = T::Floor;
        }
    }
    // East arm extension
    for y in ML..MH {
        for x in MH..N {
            g[y][x] = T::Floor;
        }
    }
    // West outer wall (full height)
    for y in 0..N       { g[y][ML] = T::Wall; }
    // East wall of N-S strip outside the T-arm
    for y in 0..ML      { g[y][MH - 1] = T::Wall; }
    for y in MH..N      { g[y][MH - 1] = T::Wall; }
    // North/south faces of the east arm extension
    for x in MH..N      { g[ML][x] = T::Wall; }
    for x in MH..N      { g[MH - 1][x] = T::Wall; }
    // Windows
    win_v(&mut g, ML,     0, N);
    win_v(&mut g, MH - 1, 0, ML);
    win_v(&mut g, MH - 1, MH, N);
    win_h(&mut g, ML,     MH, N);
    win_h(&mut g, MH - 1, MH, N);
    g
}

fn mega_cross_v1() -> Grid {
    // + shaped building: exits on all four faces.
    let mut g = empty();
    // Vertical full strip
    for y in 0..N {
        for x in ML..MH { g[y][x] = T::Floor; }
    }
    // Horizontal full strip
    for y in ML..MH {
        for x in 0..N   { g[y][x] = T::Floor; }
    }
    // Walls on vertical strip (outside horizontal junction)
    for y in 0..ML  { g[y][ML] = T::Wall; g[y][MH - 1] = T::Wall; }
    for y in MH..N  { g[y][ML] = T::Wall; g[y][MH - 1] = T::Wall; }
    // Walls on horizontal strip (outside vertical junction)
    for x in 0..ML  { g[ML][x] = T::Wall; g[MH - 1][x] = T::Wall; }
    for x in MH..N  { g[ML][x] = T::Wall; g[MH - 1][x] = T::Wall; }
    // Windows on each arm
    win_v(&mut g, ML,     0, ML);
    win_v(&mut g, MH - 1, 0, ML);
    win_v(&mut g, ML,     MH, N);
    win_v(&mut g, MH - 1, MH, N);
    win_h(&mut g, ML,     0, ML);
    win_h(&mut g, MH - 1, 0, ML);
    win_h(&mut g, ML,     MH, N);
    win_h(&mut g, MH - 1, MH, N);
    g
}

fn mega_dead_v1() -> Grid {
    // Dead-end: exits only at the north face; building terminates inside.
    // Rotations give dead-ends facing east, south, and west.
    let mut g = empty();
    for y in 0..MH {
        for x in ML..MH {
            let on_side = x == ML || x + 1 == MH;
            let on_cap  = y + 1 == MH;
            g[y][x] = if on_side || on_cap { T::Wall } else { T::Floor };
        }
    }
    gs(&mut g, MCX, MH - 1, T::Doorway);  // door in south cap (interior entrance)
    win_v(&mut g, ML,     0, MH);
    win_v(&mut g, MH - 1, 0, MH);
    win_h(&mut g, MH - 1, ML, MH);
    g
}

// ─── Thin path blocks ────────────────────────────────────────────────────────
// 2-tile alleys positioned off-centre (x=8-9 in canonical form). Because the
// path edge profile (road at x=8-9) never matches the 6-wide road (x=13-18),
// these blocks form their own WFC sub-network and sit alongside—but separate
// from—the main road network.

fn path_straight_v1() -> Grid {
    // Bare alley through open ground.
    let mut g = empty();
    v_road(&mut g, PCX, PW);
    g
}

fn path_straight_v2() -> Grid {
    // Alley running alongside a large building on the wide side.
    let mut g = path_straight_v1();
    let rx1 = PH + 1;                          // = 11, first building column
    bld(&mut g, rx1, 2, N - 1, 29);
    door(&mut g, rx1, (2 + 29) / 2);           // west wall door opens onto alley
    win_v(&mut g, N - 2, 3, 28);
    win_h(&mut g, 2,  rx1, N - 1);
    win_h(&mut g, 28, rx1, N - 1);
    g
}

fn path_straight_v3() -> Grid {
    // Alley squeezed between a narrow left building and a wide right building.
    let mut g = path_straight_v1();
    // Narrow left building (x=1..7)
    bld(&mut g, 1, 2, PL, 29);
    door(&mut g, PL - 1, (2 + 29) / 2);        // east wall door faces alley
    win_v(&mut g, 1, 3, 28);
    win_h(&mut g, 2,  1, PL);
    win_h(&mut g, 28, 1, PL);
    // Wide right building (x=11..30)
    let rx1 = PH + 1;
    bld(&mut g, rx1, 2, N - 1, 29);
    door(&mut g, rx1, (2 + 29) / 2);
    win_v(&mut g, N - 2, 3, 28);
    win_h(&mut g, 2,  rx1, N - 1);
    win_h(&mut g, 28, rx1, N - 1);
    g
}

fn path_turn_v1() -> Grid {
    // L-shaped path: N exit + E exit. Rotations give all 4 turn orientations.
    let mut g = empty();
    v_road_seg(&mut g, PCX, PW, 0, PCX + 1);  // vertical arm y=0..9
    h_road_seg(&mut g, PCX, PW, PL, N);        // horizontal arm x=8..31
    g
}

fn path_t_v1() -> Grid {
    // T-junction: straight N-S path with an east branch.
    let mut g = empty();
    v_road(&mut g, PCX, PW);
    h_road_seg(&mut g, PCX, PW, PH, N);        // east branch x=10..31
    g
}

fn path_cross_v1() -> Grid {
    // 4-way crossing of two thin paths.
    let mut g = empty();
    v_road(&mut g, PCX, PW);
    h_road(&mut g, PCX, PW);
    g
}

fn path_dead_v1() -> Grid {
    // Path enters from N and terminates mid-block.
    let mut g = empty();
    v_road_seg(&mut g, PCX, PW, 0, N / 2 + 1);
    g
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let clean = args.iter().any(|a| a == "--clean");

    let dir = args.windows(2)
        .find(|w| w[0] == "--dir")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| PathBuf::from("resources/blocks"));

    if !dir.is_dir() {
        eprintln!("Output directory {:?} does not exist.", dir);
        std::process::exit(1);
    }

    if clean {
        let count = fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("_gen_"))
            .map(|e| { fs::remove_file(e.path()).ok(); })
            .count();
        println!("Removed {} previously generated file(s).", count);
    }

    type GenFn = fn() -> Grid;
    let blocks: &[(&str, GenFn)] = &[
        // Straight N-S road
        ("roadblock_gen_straight_v1", road_straight_v1),
        ("roadblock_gen_straight_v2", road_straight_v2),
        ("roadblock_gen_straight_v3", road_straight_v3),
        ("roadblock_gen_straight_v4", road_straight_v4),
        ("roadblock_gen_straight_v5", road_straight_v5),
        ("roadblock_gen_straight_wide_v1", road_straight_wide_v1),
        // Turn N+E (all 4 turn orientations via rotation)
        ("roadblock_gen_turn_v1", road_turn_v1),
        ("roadblock_gen_turn_v2", road_turn_v2),
        ("roadblock_gen_turn_v3", road_turn_v3),
        // T-junction N+S+E (all 4 via rotation)
        ("roadblock_gen_t_v1", road_t_v1),
        ("roadblock_gen_t_v2", road_t_v2),
        ("roadblock_gen_t_v3", road_t_v3),
        // Crossing
        ("roadblock_gen_cross_v1", road_cross_v1),
        ("roadblock_gen_cross_v2", road_cross_v2),
        // Dead-end from N (all 4 directions via rotation)
        ("roadblock_gen_dead_v1", road_dead_v1),
        ("roadblock_gen_dead_v2", road_dead_v2),
        ("roadblock_gen_dead_v3", road_dead_v3),
        // Hangar
        ("roadblock_gen_hangar_v1", road_hangar_v1),
        ("roadblock_gen_hangar_v2", road_hangar_v2),
        ("roadblock_gen_hangar_v3", road_hangar_v3),
        // Mega-structure blocks (road-like strip system; buildings span 2-4 blocks)
        ("buildingblock_gen_mega_straight_v1", mega_straight_v1),
        ("buildingblock_gen_mega_straight_v2", mega_straight_v2),
        ("buildingblock_gen_mega_turn_v1",     mega_turn_v1),
        ("buildingblock_gen_mega_t_v1",        mega_t_v1),
        ("buildingblock_gen_mega_cross_v1",    mega_cross_v1),
        ("buildingblock_gen_mega_dead_v1",     mega_dead_v1),
        // Thin path blocks (2-tile alley, off-centre at x=8-9)
        ("roadblock_gen_path_straight_v1", path_straight_v1),
        ("roadblock_gen_path_straight_v2", path_straight_v2),
        ("roadblock_gen_path_straight_v3", path_straight_v3),
        ("roadblock_gen_path_turn_v1",     path_turn_v1),
        ("roadblock_gen_path_t_v1",        path_t_v1),
        ("roadblock_gen_path_cross_v1",    path_cross_v1),
        ("roadblock_gen_path_dead_v1",     path_dead_v1),
        // Building blocks (no road exits)
        ("buildingblock_gen_v1", building_v1),
        ("buildingblock_gen_v2", building_v2),
        ("buildingblock_gen_v3", building_v3),
        ("buildingblock_gen_v4", building_v4),
        ("buildingblock_gen_v5", building_v5),
        ("buildingblock_gen_v6", building_v6),
        ("buildingblock_gen_v7", building_v7),
        ("buildingblock_gen_v8", building_v8),
    ];

    let mut ok = 0usize;
    let mut fail = 0usize;
    for &(name, gen) in blocks {
        let path = dir.join(format!("{}.txt", name));
        let txt = to_txt(&gen());
        match fs::write(&path, &txt) {
            Ok(_)  => { println!("  {}.txt", name); ok += 1; }
            Err(e) => { eprintln!("  FAILED {}.txt: {}", name, e); fail += 1; }
        }
    }
    println!(
        "\n{} block(s) written  ({} WFC variants each = {} total candidates).",
        ok, 16, ok * 16
    );
    if fail > 0 { println!("{} block(s) failed.", fail); }
}
