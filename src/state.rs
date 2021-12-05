use rltk::{Rltk, GameState, Point, console};
use std::cmp::max;
use std::time::Instant;
use crate::AnimationSystem;
use crate::World;
use crate::GameLog;
use crate::ItemAction;
use crate::Item;
use crate::Menu;
use crate::IntentPhase;
use crate::ui::*;
use crate::components::*;
use crate::input::*;
use crate::Rect;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    DeclareIntent,
    AwaitingInput,
    AwaitingMenuInput,
    AwaitingPositionalTargetingInput,
    Resolve(IntentPhase),
    RenderAnimations(IntentPhase)
}

pub struct State {
    pub run_state: RunState,
    pub cursor_pos: Point,
    pub log: GameLog,

    pub world: World,
    pub animation_system: AnimationSystem,

    pub menu_stack: Vec<Box<dyn Menu>>,
    pub action_being_used: Option<ItemAction>,
    pub action_item: Option<Item>,
    pub action_slot: Option<SlotType>,

    last_tick: Instant,
    start_tick: Instant
}

impl State {
    pub fn new() -> Self {
        Self {
            run_state: RunState::AwaitingInput,
            cursor_pos: Point {x: 0, y:0},
            log: GameLog {entries: vec![]},
            world: World::new(),
            animation_system: AnimationSystem::new(),
            menu_stack: vec![],
            action_being_used: None,
            action_item: None,
            action_slot: None,
            last_tick: Instant::now(),
            start_tick: Instant::now()
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
        let monotime = self.start_tick.elapsed().as_millis();
        draw_main_screen(self, context, monotime);

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
                draw_menu(self, context, monotime);
            },
            RunState::AwaitingPositionalTargetingInput => {
                self.run_state = positional_targeting_input(self, context);
            },
            RunState::Resolve(phase) => {
                self.resolve(phase, monotime);
            },
            RunState::RenderAnimations(phase) => {
                self.animate(phase, monotime, context);
            }
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

impl State {
    pub fn get_viewport(&self) -> Rect {
        let camera_pos = match self.world.get_player() {
            Ok(player) => player.position,
            Err(_) => Point{x: (SCREEN_WIDTH / 2) as i32, y: (SCREEN_HEIGHT / 2) as i32}
        };

        let mut top = max(camera_pos.y - SCREEN_HEIGHT as i32 / 2, 0);
        let mut left = max(camera_pos.x - SCREEN_WIDTH as i32 / 2, 0);
        let mut bottom = top + VIEWPORT_HEIGHT as i32;
        let mut right = left + SCREEN_WIDTH as i32;

        if right > self.world.map.width as i32{
            right = self.world.map.width as i32;
            left = right - SCREEN_WIDTH as i32;
        }
        if bottom > self.world.map.height as i32 {
            bottom = self.world.map.height as i32;
            top = bottom - VIEWPORT_HEIGHT as i32;
        }

        Rect {
            x1: left,
            x2: right,
            y1: top,
            y2: bottom
        }
    }

    fn resolve(&mut self, phase: IntentPhase, monotime: u128) {
        let mut animations = vec!();
        let mut maybe_next_phase = phase.next();
        while animations.len() == 0 && maybe_next_phase.is_some() {
            let next_phase = maybe_next_phase.unwrap();
            animations = self.world.resolve_phase(next_phase, &mut self.log);
            maybe_next_phase = next_phase.next();
        }

        if animations.len() > 0 {
            self.animation_system.init(animations, monotime);
            self.run_state = RunState::RenderAnimations(phase);
        } else {
            match phase.next() {
                Some(next_phase) => self.run_state = RunState::Resolve(next_phase),
                None => self.run_state = RunState::DeclareIntent
            }
        }
    }

    fn animate(&mut self, phase: IntentPhase, monotime: u128, context: &mut Rltk) {
        let animation_done = self.animation_system.render(self.get_viewport(), monotime, context);
        if animation_done {
            match phase.next() {
                Some(next_phase) => self.run_state = RunState::Resolve(next_phase),
                None => self.run_state = RunState::DeclareIntent
            }
        }
    }
}