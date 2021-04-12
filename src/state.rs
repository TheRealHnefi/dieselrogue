use rltk::{Rltk, GameState, Point, console, RGB};
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
}

pub struct State {
    pub ecs: World,
    last_tick: Instant,
    pub mouse_pos: Point,
    pub menu_stack: Vec<Menu>
}

impl State {
    pub fn new() -> Self {
        Self {
            ecs: World::new(),
            last_tick: Instant::now(),
            mouse_pos: Point {x: 0, y:0},
            menu_stack: Vec::new()
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
            },
            RunState::AwaitingInput => {
                new_run_state = player_input(self, context);
            },
            RunState::MenuInput => {
                new_run_state = menu_input(self, context);
            },
            RunState::TargetingInput => {
                new_run_state = targeting_input(self, context);
            }
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

        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let large_renderables = self.ecs.read_storage::<LargeRenderable>();
            let sizes = self.ecs.read_storage::<Size>();
            let map = self.ecs.fetch::<Map>();

            // TODO: Unify these, for efficiency?
            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    context.set(pos.x, pos.y, render.color, render.background, render.glyph);
                }
            }

            for (pos, render, size) in (&positions, &large_renderables, &sizes).join() {
                assert!(size.x * size.y == render.glyphs.len() as i32, "Size and glyphmap size differ for object");
                for x in 0..size.x {
                    for y in 0..size.y {
                        let idx = map.xy_idx(pos.x + x, pos.y + y);
                        if map.visible_tiles[idx] {
                            context.set(pos.x + x, pos.y + y, render.color, render.background, render.glyphs[(x + size.x * y) as usize]);
                        }
                    }
                }
            }
        }

        draw_ui(self, context);

        if new_run_state == RunState::MenuInput {
            for menu in &self.menu_stack {
                let mut width = 0;
                for row in &menu.rows {
                    if row.text.len() > width {
                        width = row.text.len();
                    }
                }
                context.draw_box(menu.x, menu.y, width + 3, menu.rows.len() + 1, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));
                for (i, row) in menu.rows.iter().enumerate() {
                    context.print(menu.x + 2, menu.y + 1 + i as i32, row.text.to_string());
                }
            }
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