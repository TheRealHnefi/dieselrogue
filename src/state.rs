use rltk::{Rltk, GameState, Point};
use std::cmp::max;
use std::time::Instant;
use crate::Ability;
use crate::AnimationSystem;
use crate::World;
use crate::GameLog;
use crate::Menu;
use crate::PendingAction;
use crate::ExecutionPhase;
use crate::ui::*;
use crate::input::*;
use crate::Rect;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    DeclareIntent,
    AwaitingInput,
    AwaitingMenuInput,
    AwaitingPositionalTargetingInput,
    AwaitingEntityTargetingInput,
    Looking,
    AwaitingJukeInput,
    AwaitingLevelUpInput,
    Resolve(ExecutionPhase),
    RenderAnimations(ExecutionPhase),
    ResolveStatusEffects
}

pub struct State {
    pub run_state: RunState,
    pub cursor_pos: Point,
    pub log: GameLog,

    pub world: World,
    pub animation_system: AnimationSystem,

    pub menu_stack: Vec<Box<dyn Menu>>,
    pub pending_action: Option<PendingAction>,

    pub level_up_options: Vec<Ability>,
    pub level_up_selected: usize,

    pub entity_targets: Vec<usize>,
    pub entity_target_index: usize,

    pub turn: u32,

    start_tick: Instant
}

impl State {
    /// Create new game state.
    /// # Arguments
    /// * `size` - Number of blocks that make up one side of the map.
    pub fn new_game_state(size: usize) -> Self {
        Self {
            run_state: RunState::AwaitingInput,
            cursor_pos: Point {x: 0, y:0},
            log: GameLog {entries: vec![]},
            world: World::new(size),
            animation_system: AnimationSystem::new(),
            menu_stack: vec![],
            pending_action: None,
            level_up_options: vec![],
            level_up_selected: 0,
            entity_targets: vec![],
            entity_target_index: 0,
            turn: 0,
            start_tick: Instant::now()
        }
    }

    /// Create new game state for performance testing.
    pub fn new_performance_test() -> Self {
        Self {
            run_state: RunState::AwaitingInput,
            cursor_pos: Point {x: 0, y:0},
            log: GameLog {entries: vec![]},
            world: World::new_performance_test(),
            animation_system: AnimationSystem::new(),
            menu_stack: vec![],
            pending_action: None,
            level_up_options: vec![],
            level_up_selected: 0,
            entity_targets: vec![],
            entity_target_index: 0,
            turn: 0,
            start_tick: Instant::now()
        }
    }

    pub fn log(&mut self, message: String) {
        self.log.log(message);
    }
}

impl GameState for State {
    // Runs every frame.
    #[tracing::instrument(skip_all)]
    fn tick(&mut self, context: &mut Rltk) {
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
            RunState::AwaitingEntityTargetingInput => {
                self.run_state = entity_targeting_input(self, context);
            },
            RunState::Looking => {
                self.run_state = looking_input(self, context);
            },
            RunState::AwaitingJukeInput => {
                self.run_state = juke_direction_input(self, context);
            },
            RunState::AwaitingLevelUpInput => {
                draw_level_up_screen(self, context);
                self.run_state = level_up_input(self, context);
            },
            RunState::Resolve(phase) => {
                self.resolve(phase, monotime);
            },
            RunState::RenderAnimations(phase) => {
                self.animate(phase, monotime, context);
            },
            RunState::ResolveStatusEffects => {
                self.resolve_status_effects();
            }
        }
    }
}

impl State {
    pub fn get_viewport(&self, width: i32, height: i32) -> Rect {
        let camera_pos = match self.world.get_player() {
            Ok(player) => player.center(),
            Err(_) => Point{x: width / 2, y: height / 2}
        };

        let mut top = max(camera_pos.y - height / 2, 0);
        let mut left = max(camera_pos.x - width / 2, 0);
        let mut bottom = top + height;
        let mut right = left + width;

        if right > self.world.map.width as i32{
            right = self.world.map.width as i32;
            left = right - width as i32;
        }
        if bottom > self.world.map.height as i32 {
            bottom = self.world.map.height as i32;
            top = bottom - height as i32;
        }

        Rect {
            x1: left,
            x2: right,
            y1: top,
            y2: bottom
        }
    }

    fn resolve(&mut self, phase: ExecutionPhase, monotime: u128) {
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
                None => self.run_state = RunState::ResolveStatusEffects
            }
        }
    }

    fn animate(&mut self, phase: ExecutionPhase, monotime: u128, context: &mut Rltk) {
        let animation_done = self.animation_system.render(
            self.get_viewport(VIEWPORT_WIDTH as i32, VIEWPORT_HEIGHT as i32),
            monotime,
            context);
        if animation_done {
            match phase.next() {
                Some(next_phase) => self.run_state = RunState::Resolve(next_phase),
                None => self.run_state = RunState::ResolveStatusEffects
            }
        }
    }

    fn resolve_status_effects(&mut self) {
        self.world.sounds_last_turn = std::mem::take(&mut self.world.sounds);
        self.world.resolve_status_effects(&mut self.log);

        if self.world.pending_levelup {
            self.world.pending_levelup = false;
            let options = self.world.compute_levelup_options();
            if !options.is_empty() {
                self.level_up_options = options;
                self.level_up_selected = 0;
                self.run_state = RunState::AwaitingLevelUpInput;
                return;
            }
        }

        self.run_state = RunState::DeclareIntent;
        self.turn += 1;
    }
}