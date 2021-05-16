use rltk::{Rltk, GameState, Point, console};
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
    pub resources: Resources,
    pub schedule: Schedule,
    pub mouse_pos: Point,
    pub menu_stack: Vec<Menu>,
    pub inventory_screen_selection: i32,
    pub run_state: RunState,
    last_tick: Instant,
}

impl State {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            .add_system(update_visibility_system())
            .add_system(map_index_blockables_system())
            .add_system(map_index_items_system())
            .build();
        Self {
            ecs: World::default(),
            resources: Resources::default(),
            schedule: schedule,
            last_tick: Instant::now(),
            mouse_pos: Point {x: 0, y:0},
            menu_stack: Vec::new(),
            inventory_screen_selection: 0,
            run_state: RunState::PreRun
        }
    }

    fn run_systems(&mut self) {
        match self.resources.get_mut::<Map>() {
            Some(mut map) => {
                map.index_walls();
                map.clear_blockers_index();
                map.clear_items_index();
            },
            None => ()
        }

        self.schedule.execute(&mut self.ecs, &mut self.resources);
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        let begin = Instant::now();
        
        context.cls();

        match self.run_state {
            RunState::PreRun => {
                self.run_systems();
                self.run_state = RunState::AwaitingInput;
                draw_main_screen(self, context);
            },
            RunState::AwaitingInput => {
                self.run_state = main_screen_input(self, context);
                draw_main_screen(self, context);
            },
            RunState::MenuInput => {
                self.run_state = menu_input(self, context);
                draw_main_screen(self, context);
            },
            RunState::TargetingInput => {
                self.run_state = targeting_input(self, context);
                draw_main_screen(self, context);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.run_state = RunState::EnemyTurn;
                draw_main_screen(self, context);
            },
            RunState::EnemyTurn => {
                self.run_systems();
                self.run_state = RunState::AwaitingInput;
                draw_main_screen(self, context);
            },
            RunState::Saving => {
                // {
                //     let result = saveload_system::save_game(&mut self.ecs);
                //     let mut game_log = self.ecs.fetch_mut::<GameLog>();
                //     match result {
                //         Ok(_) => game_log.entries.push("Game saved.".to_string()),
                //         Err(_) => game_log.entries.push("Game could not be saved.".to_string())
                //     }
                //     new_run_state = RunState::AwaitingInput;
                // }
                // draw_main_screen(self, context);
                self.run_state = RunState::AwaitingInput;
            },
            RunState::Loading => {
                // {
                //     let result = saveload_system::load_game(&mut self.ecs);
                //     let mut game_log = self.ecs.fetch_mut::<GameLog>();
                //     match result {
                //         Ok(_) => game_log.entries.push("Game loaded.".to_string()),
                //         Err(_) => game_log.entries.push("Game could not be loaded.".to_string())
                //     }
                //     new_run_state = RunState::AwaitingInput;
                // }
                // draw_main_screen(self, context);
                self.run_state = RunState::AwaitingInput;
            },
            RunState::InventoryScreen => {
                // new_run_state = inventory_screen_input(self, context);
                // draw_inventory_screen(self, context);
                self.run_state = RunState::AwaitingInput;
            }
        }

        if self.run_state == RunState::MenuInput {
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