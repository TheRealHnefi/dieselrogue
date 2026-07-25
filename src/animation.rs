use rltk::{RGB, Rltk, Point};
use crate::Renderable;
use crate::Rect;
use crate::MAIN_CONSOLE_INDEX;

pub fn shot_animation(start_pos: Point, target_pos: Point, number_of_shots: i32) -> Animation {
    let particle_red = Particle::Complete(
        Renderable {
            glyph: rltk::to_cp437(' '),
            color: RGB::named(rltk::BLACK),
            background: RGB::named(rltk::RED)
        }
    );
    let particle_yellow = Particle::Complete(
        Renderable {
            glyph: rltk::to_cp437(' '),
            color: RGB::named(rltk::BLACK),
            background: RGB::named(rltk::YELLOW)
        }
    );
    let mut frames = vec!();
    
    for _ in 0..number_of_shots {
        frames.push(Frame {
            particles: vec!(particle_red.clone(), particle_red.clone()),
            positions: vec!(start_pos, target_pos),
            duration_ms: 100
        });

        frames.push(Frame {
            particles: vec!(particle_yellow.clone(), particle_yellow.clone()),
            positions: vec!(start_pos, target_pos),
            duration_ms: 100
        });
    }

    Animation {
        frames: frames,
        current_frame: 0,
        time_spent_in_current_frame: 0,
        done: false
    }
}

pub fn fan_fire_animation(positions: Vec<Point>) -> Animation {
    let particle_a = Particle::Complete(Renderable {
        glyph: rltk::to_cp437('^'),
        color: RGB::named(rltk::YELLOW),
        background: RGB::named(rltk::RED)
    });
    let particle_b = Particle::Complete(Renderable {
        glyph: rltk::to_cp437('*'),
        color: RGB::named(rltk::RED),
        background: RGB::named(rltk::YELLOW)
    });

    let frame_1 = Frame {
        particles: positions.iter().map(|_| particle_a.clone()).collect(),
        positions: positions.clone(),
        duration_ms: 150
    };
    let frame_2 = Frame {
        particles: positions.iter().map(|_| particle_b.clone()).collect(),
        positions: positions.clone(),
        duration_ms: 150
    };

    Animation {
        frames: vec!(frame_1, frame_2),
        current_frame: 0,
        time_spent_in_current_frame: 0,
        done: false
    }
}

pub fn explosion_animation(pos: Point) -> Animation {
    let particle = Particle::Complete(
        Renderable {
            glyph: rltk::to_cp437('*'),
            color: RGB::named(rltk::RED),
            background: RGB::named(rltk::YELLOW)
        }
    );

    let frame_1 = Frame {
        particles: vec!(particle.clone()),
        positions: vec!(pos),
        duration_ms: 250
    };

    let frame_2 = Frame {
        particles: vec!(
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone()
        ),
        positions: vec!(
            pos,
            Point {x: pos.x + 1, y: pos.y},
            Point {x: pos.x - 1, y: pos.y},
            Point {x: pos.x, y: pos.y + 1},
            Point {x: pos.x, y: pos.y - 1},
        ),
        duration_ms: 250
    };

    let frame_3 = Frame {
        particles: vec!(
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone(),
            particle.clone()
        ),
        positions: vec!(
            Point {x: pos.x + 1, y: pos.y},
            Point {x: pos.x + 1, y: pos.y + 1},
            Point {x: pos.x, y: pos.y + 1},
            Point {x: pos.x - 1, y: pos.y + 1},
            Point {x: pos.x - 1, y: pos.y},
            Point {x: pos.x - 1, y: pos.y - 1},
            Point {x: pos.x, y: pos.y - 1},
            Point {x: pos.x + 1, y: pos.y - 1},
        ),
        duration_ms: 250
    };

    Animation {
        frames: vec!(frame_1, frame_2, frame_3),
        current_frame: 0,
        time_spent_in_current_frame: 0,
        done: false
    }
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
    pub fn render(&mut self, viewport: Rect, delta_time: u32, context: &mut Rltk) {
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