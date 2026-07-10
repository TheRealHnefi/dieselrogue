use rltk::{Rltk, GameState, Point, VirtualKeyCode};
use crate::Bindings;
use crate::RebindTarget;
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
use crate::FontSize;
use crate::RexAssets;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    WelcomeScreen,
    WelcomeSplash,
    DeclareIntent,
    AwaitingInput,
    AwaitingMenuInput,
    AwaitingRebind(RebindTarget, Option<&'static str>),
    AwaitingPositionalTargetingInput,
    AwaitingEntityTargetingInput,
    Looking,
    AwaitingDirectionalTargetingInput,
    AwaitingLevelUpInput,
    Resolve(ExecutionPhase),
    RenderAnimations(ExecutionPhase),
    ResolveStatusEffects,
    GameOver,
    Victory,
    HelpScreen,
}

pub struct State {
    pub run_state: RunState,
    pub welcome_selected: usize,
    pub seed: u64,
    pub bindings: Bindings,
    pub menu_return_state: RunState,
    pub help_return_state: RunState,
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

    pub freelook: bool,
    pub freelook_pos: Point,

    pub rex_assets: RexAssets,

    /// Font size chosen in the menu but not yet applied (requires restart).
    pub pending_font_size: Option<FontSize>,
    /// Fullscreen toggle chosen in the menu but not yet applied (requires restart).
    pub pending_fullscreen: Option<bool>,

    /// Last key pressed, captured every tick before the state machine runs.
    /// Persists across non-input states so a press during AI/resolve is not dropped.
    pub last_input: Option<VirtualKeyCode>,

    /// True when the strafe modifier key is currently held.
    pub strafe_held: bool,

    start_tick: Instant
}

impl State {
    /// Create new game state.
    /// # Arguments
    /// * `size` - Number of blocks that make up one side of the map.
    /// Create a welcome-screen-only state with a dummy world.
    /// World generation is deferred until the player starts a new game.
    pub fn new_welcome_state(seed: u64, bindings: Bindings) -> Self {
        Self {
            run_state: RunState::WelcomeScreen,
            welcome_selected: 0,
            seed,
            bindings,
            menu_return_state: RunState::AwaitingInput,
            help_return_state: RunState::AwaitingInput,
            cursor_pos: Point {x: 0, y: 0},
            log: GameLog {entries: vec![]},
            world: World::new_test(),
            animation_system: AnimationSystem::new(),
            menu_stack: vec![],
            pending_action: None,
            level_up_options: vec![],
            level_up_selected: 0,
            entity_targets: vec![],
            entity_target_index: 0,
            turn: 0,
            freelook: false,
            freelook_pos: Point {x: 0, y: 0},
            rex_assets: RexAssets::new(),
            pending_font_size: None,
            pending_fullscreen: None,
            last_input: None,
            strafe_held: false,
            start_tick: Instant::now(),
        }
    }

    pub fn new_game_state(size: usize, seed: u64, skip_intro: bool, bindings: Bindings) -> Self {
        Self {
            run_state: if skip_intro { RunState::AwaitingInput } else { RunState::WelcomeScreen },
            welcome_selected: 0,
            seed,
            bindings,
            menu_return_state: RunState::AwaitingInput,
            help_return_state: RunState::AwaitingInput,
            cursor_pos: Point {x: 0, y:0},
            log: GameLog {entries: vec![]},
            world: World::new(size, seed),
            animation_system: AnimationSystem::new(),
            menu_stack: vec![],
            pending_action: None,
            level_up_options: vec![],
            level_up_selected: 0,
            entity_targets: vec![],
            entity_target_index: 0,
            turn: 0,
            freelook: false,
            freelook_pos: Point {x: 0, y: 0},
            rex_assets: RexAssets::new(),
            pending_font_size: None,
            pending_fullscreen: None,
            last_input: None,
            strafe_held: false,
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
        if context.key.is_some() {
            self.last_input = context.key;
        }

        self.strafe_held = match self.bindings.strafe {
            VirtualKeyCode::LShift | VirtualKeyCode::RShift => context.shift,
            VirtualKeyCode::LControl | VirtualKeyCode::RControl => context.control,
            VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => context.alt,
            _ => false,
        };

        // F1 opens help from any awaiting-input gameplay state.
        if self.last_input == Some(VirtualKeyCode::F1) {
            let is_input_state = matches!(self.run_state,
                RunState::AwaitingInput
                | RunState::AwaitingMenuInput
                | RunState::Looking
                | RunState::AwaitingPositionalTargetingInput
                | RunState::AwaitingEntityTargetingInput
                | RunState::AwaitingDirectionalTargetingInput
                | RunState::AwaitingLevelUpInput
            );
            if is_input_state {
                self.last_input = None;
                self.help_return_state = self.run_state;
                self.run_state = RunState::HelpScreen;
            }
        }

        let monotime = self.start_tick.elapsed().as_millis();

        match self.run_state {
            RunState::WelcomeScreen => {
                draw_welcome_screen(self, context);
                self.run_state = welcome_screen_input(self, context);
                return;
            },
            RunState::WelcomeSplash => {
                draw_welcome_splash(context);
                self.run_state = welcome_splash_input(self, context);
                return;
            },
            RunState::GameOver => {
                draw_game_over_screen(context);
                self.run_state = game_over_input(self, context);
                return;
            },
            RunState::Victory => {
                draw_victory_screen(context);
                self.run_state = victory_input(self, context);
                return;
            },
            RunState::HelpScreen => {
                draw_help_screen(self, context);
                if self.last_input.take() == Some(VirtualKeyCode::Escape) {
                    self.run_state = self.help_return_state;
                }
                return;
            },
            _ => {}
        }

        let in_welcome_context = self.menu_return_state == RunState::WelcomeScreen
            && matches!(self.run_state, RunState::AwaitingMenuInput | RunState::AwaitingRebind(_, _));
        if in_welcome_context {
            draw_welcome_screen(self, context);
        } else {
            draw_main_screen(self, context, monotime);
        }

        match self.run_state {
            RunState::DeclareIntent => {
                #[cfg(debug_assertions)]
                puffin::GlobalProfiler::lock().new_frame();
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
            RunState::AwaitingRebind(target, conflict) => {
                draw_menu(self, context, monotime);
                draw_rebind_prompt(target, conflict, context);
                self.run_state = rebind_input(self, context);
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
            RunState::AwaitingDirectionalTargetingInput => {
                self.run_state = directional_targeting_input(self, context);
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
            RunState::WelcomeScreen | RunState::WelcomeSplash | RunState::GameOver | RunState::Victory | RunState::HelpScreen => unreachable!(),
        }
    }
}

impl State {
    pub fn get_viewport(&self, width: i32, height: i32) -> Rect {
        let camera_pos = if self.freelook {
            self.freelook_pos
        } else {
            match self.world.get_player() {
                Ok(player) => player.center(),
                Err(_) => Point{x: width / 2, y: height / 2}
            }
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

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
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

        if self.world.get_player().is_err() {
            self.run_state = RunState::GameOver;
            return;
        }

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