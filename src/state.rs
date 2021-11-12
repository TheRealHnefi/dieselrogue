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
}

impl GameState for State {
    /// Called periodically as real time advances.
    fn tick(&mut self, context: &mut Rltk) {
        let begin = Instant::now();
        
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
                assert!(self.world.resolve_phase(IntentPhase::Instant).is_ok());
                assert!(self.world.resolve_phase(IntentPhase::Movement).is_ok());
                assert!(self.world.resolve_phase(IntentPhase::Inventory).is_ok());
                assert!(self.world.resolve_phase(IntentPhase::Attack).is_ok());
                assert!(self.world.resolve_phase(IntentPhase::Misc).is_ok());

                self.run_state = RunState::DeclareIntent;
            }
        }

        draw_main_screen(self, context);
        if self.run_state == RunState::AwaitingMenuInput {
            draw_menu(self, context);
        }
 
        let tick_time = begin.elapsed().as_millis();
        if tick_time > 160 {
            console::log(format!("Tick time: {}", tick_time));
        }
        let tick_rate = self.last_tick.elapsed().as_micros();
        if tick_rate > 40000 {
            console::log(format!("Time since last tick: {}", tick_rate));
        }
        self.last_tick = Instant::now();
    }
}