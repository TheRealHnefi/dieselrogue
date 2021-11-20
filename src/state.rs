use rltk::{Rltk, GameState, Point, console};
use super::*;
use std::time::{Instant};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    DeclareIntent,
    AwaitingInput,
    AwaitingMenuInput,
    AwaitingPositionalTargetingInput,
    Resolve
}

pub struct State {
    pub run_state: RunState,
    pub cursor_pos: Point,
    pub log: GameLog,

    pub world: World,

    pub menu_stack: Vec<Box<dyn Menu>>,
    pub action_being_used: Option<ItemAction>,
    pub action_item: Option<Item>,
    pub action_slot: Option<SlotType>,

    last_tick: Instant,
}

impl State {
    pub fn new() -> Self {
        Self {
            run_state: RunState::AwaitingInput,
            cursor_pos: Point {x: 0, y:0},
            log: GameLog {entries: vec![]},
            world: World::new(),
            menu_stack: vec![],
            action_being_used: None,
            action_item: None,
            action_slot: None,
            last_tick: Instant::now(),
        }
    }

    pub fn log(&mut self, message: String) {
        self.log.log(message);
    }
}

const PROFILING: bool = true;

impl GameState for State {
    /// Called periodically as real time advances.
    fn tick(&mut self, context: &mut Rltk) {
        let begin = Instant::now();
        let tick_interval = self.last_tick.elapsed().as_millis();
        
        context.cls();

        match self.run_state {
            RunState::DeclareIntent => {
                self.world.resolve_intent_declaration();
                self.run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                self.run_state = main_screen_input(self, context);
            },
            RunState::AwaitingMenuInput => {
                self.run_state = menu_input(self, context);
            },
            RunState::AwaitingPositionalTargetingInput => {
                self.run_state = positional_targeting_input(self, context);
            },
            RunState::Resolve => {
                self.world.resolve_phase(IntentPhase::Instant, &mut self.log);
                self.world.resolve_phase(IntentPhase::Inventory, &mut self.log);
                self.world.resolve_phase(IntentPhase::Attack, &mut self.log);
                self.world.resolve_phase(IntentPhase::Movement, &mut self.log);
                self.world.resolve_phase(IntentPhase::Misc, &mut self.log);
                
                self.world.update_views();

                self.run_state = RunState::DeclareIntent;
            }
        }

        draw_main_screen(self, context);
        if self.run_state == RunState::AwaitingMenuInput {
            draw_menu(self, context);
        }
 
        if PROFILING {
            let tick_time = begin.elapsed().as_millis();
            if tick_time + tick_interval > 30 {
                console::log(format!("Tick duration,interval: {}, {}  ", tick_time, tick_interval));
            }
            self.last_tick = Instant::now();
        }
    }
}