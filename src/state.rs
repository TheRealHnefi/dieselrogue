use rltk::{Rltk, GameState, Point, console};
use specs::prelude::*;
use super::*;
use std::time::{Instant};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    TargetingInput,
    MenuInput,
    PreRun,
    PlayerTurn,
    EnemyTurn,
    Saving,
    Loading,
    InventoryScreen
}

pub struct State {
    pub ecs: World,
    pub mouse_pos: Point,
    pub menu_stack: Vec<Menu>,
    pub inventory_screen_selection: i32,

    last_tick: Instant,
}

impl State {
    pub fn new() -> Self {
        Self {
            ecs: World::new(),
            last_tick: Instant::now(),
            mouse_pos: Point {x: 0, y:0},
            menu_stack: Vec::new(),
            inventory_screen_selection: 0
        }
    }

    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut enemy_ai = EnemyAI {};
        enemy_ai.run_now(&self.ecs);
        let mut tank_ai = TankAI {};
        tank_ai.run_now(&self.ecs);
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
                draw_main_screen(self, context);
            },
            RunState::AwaitingInput => {
                new_run_state = main_screen_input(self, context);
                draw_main_screen(self, context);
            },
            RunState::MenuInput => {
                new_run_state = menu_input(self, context);
                draw_main_screen(self, context);
            },
            RunState::TargetingInput => {
                new_run_state = targeting_input(self, context);
                draw_main_screen(self, context);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_run_state = RunState::EnemyTurn;
                draw_main_screen(self, context);
            },
            RunState::EnemyTurn => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
                draw_main_screen(self, context);
            },
            RunState::Saving => {
                {
                    let result = saveload_system::save_game(&mut self.ecs);
                    let mut game_log = self.ecs.fetch_mut::<GameLog>();
                    match result {
                        Ok(_) => game_log.entries.push("Game saved.".to_string()),
                        Err(_) => game_log.entries.push("Game could not be saved.".to_string())
                    }
                    new_run_state = RunState::AwaitingInput;
                }
                draw_main_screen(self, context);
            },
            RunState::Loading => {
                {
                    let result = saveload_system::load_game(&mut self.ecs);
                    let mut game_log = self.ecs.fetch_mut::<GameLog>();
                    match result {
                        Ok(_) => game_log.entries.push("Game loaded.".to_string()),
                        Err(_) => game_log.entries.push("Game could not be loaded.".to_string())
                    }
                    new_run_state = RunState::AwaitingInput;
                }
                draw_main_screen(self, context);
            },
            RunState::InventoryScreen => {
                new_run_state = inventory_screen_input(self, context);
                draw_inventory_screen(self, context);
            }
        }
        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = new_run_state;
        }

        if new_run_state == RunState::MenuInput {
            draw_menu(self, context);
        }

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