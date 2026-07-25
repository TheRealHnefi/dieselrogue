use rltk::{Rltk, GameState, console};
use specs::prelude::*;
use super::*;
use std::time::{Instant};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    EnemyTurn,
}

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
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut enemy_ai = EnemyAI {};
        enemy_ai.run_now(&self.ecs);
        let mut inventory_system = InventorySystem {};
        inventory_system.run_now(&self.ecs);
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        let begin = Instant::now();
        
        context.cls();

        let mut new_run_state;
        {
            let run_state = self.ecs.fetch::<RunState>();
            new_run_state = *run_state;
        }
        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            },
            RunState::AwaitingInput => {
                new_run_state = player_input(self, context);
            },
            RunState::PlayerTurn => {
                self.run_systems();
                new_run_state = RunState::EnemyTurn;                
            },
            RunState::EnemyTurn => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
        }
        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        draw_map(&self.ecs, context);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                context.set(pos.x, pos.y, render.color, render.background, render.glyph);
            }
        }

        draw_ui(&self.ecs, context);

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