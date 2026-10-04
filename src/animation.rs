use rltk::{RGB, Rltk, Point};
use crate::Renderable;
use crate::Rect;
use crate::Map;
use crate::MAIN_CONSOLE_INDEX;

fn particle(glyph: rltk::FontCharType, color: RGB, background: RGB) -> Particle {
    Particle::Complete(Renderable { glyph, color, background })
}

/// A pause showing nothing (a hidden step or a flicker gap).
fn blank_frame(duration_ms: u32) -> Frame {
    Frame { particles: vec![], positions: vec![], duration_ms }
}

/// The streak glyph matching the path's dominant direction.
fn streak_glyph(from: Point, to: Point) -> rltk::FontCharType {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dy == 0 || dx.abs() > 2 * dy.abs() {
        rltk::to_cp437('─')
    } else if dx == 0 || dy.abs() > 2 * dx.abs() {
        rltk::to_cp437('│')
    } else if dx * dy > 0 {
        rltk::to_cp437('\\')
    } else {
        rltk::to_cp437('/')
    }
}

/// Gunfire as a tracer: the visible part of the bullet path flashes once per
/// shot, then fades. `None` when the player would see none of it.
pub fn shot_animation(map: &Map, start_pos: Point, target_pos: Point, number_of_shots: i32) -> Option<Animation> {
    let path = rltk::line2d(rltk::LineAlg::Bresenham, start_pos, target_pos);
    if path.len() < 2 {
        return None;
    }
    let streak: Vec<Point> = path[1..].iter().copied().filter(|&p| map.is_visible(p)).collect();
    if streak.is_empty() {
        return None;
    }

    let glyph  = streak_glyph(start_pos, target_pos);
    let bright = particle(glyph, RGB::from_f32(1.0, 0.9, 0.3), RGB::named(rltk::BLACK));
    let fade   = particle(glyph, RGB::from_f32(0.6, 0.35, 0.1), RGB::named(rltk::BLACK));

    let mut frames = vec![];
    for _ in 0..number_of_shots.max(1) {
        frames.push(Frame { particles: vec![bright.clone(); streak.len()], positions: streak.clone(), duration_ms: 50 });
        frames.push(blank_frame(30));
    }
    frames.push(Frame { particles: vec![fade; streak.len()], positions: streak, duration_ms: 70 });
    Some(Animation::from_frames(frames))
}

/// A thrown item stepping tile by tile to its target. Hidden steps keep their
/// timing but show nothing. `None` when no step is visible.
pub fn throw_animation(map: &Map, start_pos: Point, target_pos: Point) -> Option<Animation> {
    let path = rltk::line2d(rltk::LineAlg::Bresenham, start_pos, target_pos);
    if path.len() < 2 || !path[1..].iter().any(|&p| map.is_visible(p)) {
        return None;
    }
    let lobbed = particle(rltk::to_cp437('o'), RGB::from_f32(0.85, 0.85, 0.85), RGB::named(rltk::BLACK));
    let frames = path[1..].iter().map(|&p| {
        if map.is_visible(p) {
            Frame { particles: vec![lobbed.clone()], positions: vec![p], duration_ms: 40 }
        } else {
            blank_frame(40)
        }
    }).collect();
    Some(Animation::from_frames(frames))
}

/// A rocket: a bright head stepping down the path, dragging a short streak,
/// ending in the impact explosion.
pub fn rocket_animation(map: &Map, start_pos: Point, target_pos: Point, radius: u32) -> Animation {
    const TRAIL_LEN: usize = 3;
    let path  = rltk::line2d(rltk::LineAlg::Bresenham, start_pos, target_pos);
    let glyph = streak_glyph(start_pos, target_pos);
    let head  = particle(rltk::to_cp437('*'), RGB::named(rltk::YELLOW), RGB::named(rltk::RED));
    let trail = particle(glyph, RGB::from_f32(0.9, 0.5, 0.1), RGB::named(rltk::BLACK));

    let mut frames = vec![];
    for i in 1..path.len() {
        let mut particles = vec![];
        let mut positions = vec![];
        for j in i.saturating_sub(TRAIL_LEN).max(1)..i {
            if map.is_visible(path[j]) {
                particles.push(trail.clone());
                positions.push(path[j]);
            }
        }
        if map.is_visible(path[i]) {
            particles.push(head.clone());
            positions.push(path[i]);
        }
        frames.push(Frame { particles, positions, duration_ms: 35 });
    }
    frames.extend(explosion_frames(target_pos, radius));
    Animation::from_frames(frames)
}

pub fn fan_fire_animation(positions: Vec<Point>) -> Animation {
    let particle_a = particle(rltk::to_cp437('^'), RGB::named(rltk::YELLOW), RGB::named(rltk::RED));
    let particle_b = particle(rltk::to_cp437('*'), RGB::named(rltk::RED), RGB::named(rltk::YELLOW));

    let frame_1 = Frame {
        particles: vec![particle_a; positions.len()],
        positions: positions.clone(),
        duration_ms: 150
    };
    let frame_2 = Frame {
        particles: vec![particle_b; positions.len()],
        positions,
        duration_ms: 150
    };

    Animation::from_frames(vec![frame_1, frame_2])
}

fn chebyshev_ring(pos: Point, radius: i32) -> Vec<Point> {
    let mut pts = vec![];
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx.abs().max(dy.abs()) == radius {
                pts.push(Point { x: pos.x + dx, y: pos.y + dy });
            }
        }
    }
    pts
}

pub fn flashbang_animation(pos: Point, radius: u32) -> Animation {
    let bright = particle(rltk::to_cp437('█'), RGB::named(rltk::WHITE), RGB::named(rltk::WHITE));
    let fade   = particle(rltk::to_cp437('█'), RGB::from_f32(0.8, 0.8, 0.8), RGB::from_f32(0.65, 0.65, 0.65));

    let mut frames = vec![
        Frame { particles: vec![bright.clone()], positions: vec![pos], duration_ms: 80 },
    ];
    for r in 1..radius as i32 {
        let pts = chebyshev_ring(pos, r);
        frames.push(Frame { particles: vec![bright.clone(); pts.len()], positions: pts, duration_ms: 80 });
    }
    let outer = chebyshev_ring(pos, radius as i32);
    frames.push(Frame { particles: vec![fade; outer.len()], positions: outer, duration_ms: 120 });

    Animation::from_frames(frames)
}

/// The expanding-ring explosion frames, shared by [`explosion_animation`] and
/// the tail of [`rocket_animation`].
fn explosion_frames(pos: Point, radius: u32) -> Vec<Frame> {
    let blast = particle(rltk::to_cp437('*'), RGB::named(rltk::RED), RGB::named(rltk::YELLOW));

    let mut frames = vec![
        Frame { particles: vec![blast.clone()], positions: vec![pos], duration_ms: 250 },
    ];
    for r in 1..=radius as i32 {
        let pts = chebyshev_ring(pos, r);
        frames.push(Frame { particles: vec![blast.clone(); pts.len()], positions: pts, duration_ms: 250 });
    }
    frames
}

pub fn explosion_animation(pos: Point, radius: u32) -> Animation {
    Animation::from_frames(explosion_frames(pos, radius))
}

#[derive(Clone)]
enum Particle {
    Complete(Renderable)
    // Background-only particle makes sense, but does not work with rltk::fancy_console
}

#[derive(Clone)]
struct Frame {
    particles: Vec<Particle>,
    positions: Vec<Point>,
    duration_ms: u32
}

#[derive(Clone)]
pub struct Animation {
    frames: Vec<Frame>,
    current_frame: usize,
    time_spent_in_current_frame: u32,
    done: bool
}

pub struct AnimationSystem {
    animations: Vec<Animation>,
    start_time: u128
}

impl AnimationSystem {
    pub fn new() -> Self {
        AnimationSystem {
            animations: vec!(),
            start_time: 0
        }
    }

    pub fn init(&mut self, animations: Vec<Animation>, monotime: u128) {
        self.animations = animations;
        self.start_time = monotime;
        for animation in &mut self.animations {
            animation.current_frame = 0;
            animation.time_spent_in_current_frame = 0;
            animation.done = false;
        }
    }

    pub fn render(&mut self, viewport: Rect, monotime: u128, context: &mut Rltk) -> bool {
        context.set_active_console(MAIN_CONSOLE_INDEX);
        let delta_time = (monotime - self.start_time) as u32;
        self.start_time = monotime;

        let mut all_done = true;
        for animation in &mut self.animations {
            animation.render(viewport, delta_time, context);
            if !animation.done {
                all_done = false;
            }
        }

        return all_done;
    }
}

impl Animation {
    fn from_frames(frames: Vec<Frame>) -> Self {
        Animation { frames, current_frame: 0, time_spent_in_current_frame: 0, done: false }
    }

    pub fn render(&mut self, viewport: Rect, delta_time: u32, context: &mut Rltk) {
        // The system keeps rendering every animation until ALL are done, so a
        // finished one (current_frame == frames.len()) must stay inert.
        if self.done {
            return;
        }
        self.time_spent_in_current_frame += delta_time;

        if self.time_spent_in_current_frame >= self.frames[self.current_frame].duration_ms {
            self.time_spent_in_current_frame -= self.frames[self.current_frame].duration_ms;

            self.current_frame += 1;
            if self.current_frame >= self.frames.len() {
                self.done = true;
                return;
            }
        }

        for i in 0..self.frames[self.current_frame].particles.len() {
            let particle = &self.frames[self.current_frame].particles[i];
            let position = self.frames[self.current_frame].positions[i];

            let screen_pos = Point {
                x: position.x - viewport.x1,
                y: position.y - viewport.y1
            };

            match particle {
                Particle::Complete(renderable) =>
                    context.set(screen_pos.x, screen_pos.y, renderable.color, renderable.background, renderable.glyph)
            }
        }
    }
}
