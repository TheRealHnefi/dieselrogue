use rltk::{Rltk, GameState, Point, console};
use super::*;
use std::time::{Instant};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
}

pub struct State {
    pub run_state: RunState,
    pub mouse_pos: Point,
    pub log: GameLog,

    pub world: World,

    last_tick: Instant,
}

impl State {
    pub fn new() -> Self {
        Self {
            run_state: RunState::AwaitingInput,
            mouse_pos: Point {x: 0, y:0},
            log: GameLog {entries: vec![]},
            world: World::new(),
            last_tick: Instant::now(),
        }
    }

    /// Moves the state machine forward.
    fn execute(&mut self) {
    }
}

impl GameState for State {
    /// Called periodically as real time advances.
    fn tick(&mut self, context: &mut Rltk) {
        let begin = Instant::now();
        
        context.cls();

        match self.run_state {
            RunState::AwaitingInput => {
                self.execute();
                self.run_state = RunState::AwaitingInput;
            },
            _ => ()
        }

        draw_main_screen(self, context);
 
        let tick_time = begin.elapsed().as_micros();
        if tick_time > 6000 {
            console::log(format!("Tick time: {}", tick_time));
        }
        let tick_rate = self.last_tick.elapsed().as_micros();
        if tick_rate > 40000 {
            console::log(format!("Time since last tick: {}", tick_rate));
        }
        self.last_tick = Instant::now();
    }
}