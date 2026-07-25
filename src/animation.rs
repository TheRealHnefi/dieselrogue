use rltk::Point;
use rltk::Rltk;
use crate::Renderable;
use crate::RunState;
use crate::Rect;

pub struct Frame {
    pub renderables: Vec<Renderable>,
    pub positions: Vec<Point>,
    pub duration_ms: u32
}

pub struct Animation {
    pub frames: Vec<Frame>,
    pub current_frame: usize,
    pub time_spent_in_current_frame: u32,
    pub done: bool
}

pub struct AnimationSystem {
    pub animations: Vec<Animation>,
    pub start_time: u128
}

impl AnimationSystem {
    pub fn new() -> Self {
        AnimationSystem {
            animations: vec!(),
            start_time: 0
        }
    }

    pub fn init_render(&mut self, monotime: u128) {
        self.start_time = monotime;
        for animation in &mut self.animations {
            animation.current_frame = 0;
            animation.time_spent_in_current_frame = 0;
            animation.done = false;
        }
    }

    pub fn render(&mut self, viewport: Rect, monotime: u128, context: &mut Rltk) -> RunState {
        let delta_time = (monotime - self.start_time) as u32;
        self.start_time = monotime;

        let mut all_done = true;
        for animation in &mut self.animations {
            animation.render(viewport, delta_time, context);
            if !animation.done {
                all_done = false;
            }
        }

        if all_done {
            return RunState::DeclareIntent;
        }
        else {
            return RunState::RenderAnimations;
        }
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

        for i in 0..self.frames[self.current_frame].renderables.len() {
            let renderable = self.frames[self.current_frame].renderables[i];
            let position = self.frames[self.current_frame].positions[i];

            let screen_pos = Point {
                x: position.x - viewport.x1,
                y: position.y - viewport.y1
            };

            context.set(screen_pos.x, screen_pos.y, renderable.color, renderable.background, renderable.glyph);
        }
    }
}