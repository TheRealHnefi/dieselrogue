use rltk::Point;

pub fn adjacent(p1: Point, p2: Point) -> bool {
    return (p1.x - p2.x).abs() <= 1 && (p1.y - p2.y).abs() <= 1;
}