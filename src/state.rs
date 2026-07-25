use rltk::{Rltk, GameState, console};
use specs::prelude::*;
use super::*;
use std::time::{Instant};

pub struct State {
    pub ecs: World,
    last_tick: Instant
}

impl State {
    pub fn new() -> Self {
        Self {
            ecs: World::new(),
            last_tick: Instant::now()
        }
    }

    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        let begin = Instant::now();
        
        context.cls();
        
        player_input(self, context);

        self.run_systems();

        draw_map(&self.ecs, context); // Expensive

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            context.set(pos.x, pos.y, render.color, render.background, render.glyph);
        }

        let tick_time = begin.elapsed().as_micros();
        if tick_time > 2000 {
            console::log(format!("Tick time: {}", tick_time));
        }
        let tick_rate = self.last_tick.elapsed().as_micros();
        if tick_rate > 20000 {
            console::log(format!("Time since last tick: {}", tick_rate));
        }
        self.last_tick = Instant::now();
    }
}