mod state;
pub use state::*;
mod world;
pub use world::*;
mod entity;
pub use entity::*;
mod item;
pub use item::*;
mod components;
pub use components::*;
mod player;
pub use player::*;
mod map;
pub use map::*;
mod tile;
pub use tile::*;
mod block;
pub use block::*;
mod rect;
pub use rect::*;
mod ui;
pub use ui::*;
mod gamelog;
pub use gamelog::*;
mod menu;
pub use menu::*;
mod input;
pub use input::*;
mod error;
pub use error::*;
mod body;
pub use body::*;
mod ability;
pub use ability::*;
mod ai;
pub use ai::*;
mod viewshed;
pub use viewshed::*;
mod util;
pub use util::*;
mod intent;
pub use intent::*;
mod sprite;
pub use sprite::*;
mod animation;
pub use animation::*;
mod actions;
pub use actions::*;
mod settings;
pub use settings::*;
mod rex_assets;
pub use rex_assets::*;
mod pathfinding;
pub use pathfinding::*;
mod spawn;
pub use spawn::*;

use std::time::Instant;
use tracing::{span, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};
use tracing_subscriber::prelude::*;

struct SpanStart(Instant);

struct SlowSpanLayer {
    threshold_ms: u128,
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for SlowSpanLayer {
    fn on_new_span(&self, _: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanStart(Instant::now()));
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            if let Some(start) = span.extensions().get::<SpanStart>() {
                let elapsed_ms = start.0.elapsed().as_millis();
                if elapsed_ms >= self.threshold_ms {
                    tracing::warn!(span = span.name(), duration_ms = elapsed_ms, "SLOW");
                }
            }
        }
    }
}

fn parse_args() -> (u64, bool) {
    let args: Vec<String> = std::env::args().collect();
    let mut seed: Option<u64> = None;
    let mut skip_intro = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                if let Some(val) = args.get(i + 1) {
                    if let Ok(n) = val.parse::<u64>() {
                        seed = Some(n);
                        i += 1;
                    } else {
                        eprintln!("Warning: '--seed {}' is not a valid u64; using random seed.", val);
                    }
                } else {
                    eprintln!("Warning: '--seed' requires a value; using random seed.");
                }
            },
            "--skip-intro" => skip_intro = true,
            _ => {}
        }
        i += 1;
    }
    let resolved_seed = seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    });
    (resolved_seed, skip_intro)
}

fn main() -> rltk::BError {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .without_time()
            .with_filter(tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("dieselrogue=debug".parse().unwrap())))
        .with(SlowSpanLayer { threshold_ms: 20 })
        .init();

    let settings = Settings::load();
    let tile_px        = settings.font_size.tile_px();
    let font_file      = settings.font_size.font_file();
    let font_native_px = settings.font_size.font_native_px();

    let mut builder = rltk::RltkBuilder::new()
        .with_fancy_console(ui::SCREEN_WIDTH, ui::SCREEN_HEIGHT, font_file)
        .with_fancy_console(ui::SCREEN_WIDTH, ui::SCREEN_HEIGHT, font_file)
        .with_dimensions(ui::SCREEN_WIDTH, ui::SCREEN_HEIGHT)
        .with_title("Diesel Rogue")
        .with_resource_path("resources")
        .with_font(font_file, font_native_px, font_native_px)
        .with_tile_dimensions(tile_px, tile_px);

    if settings.fullscreen {
        builder = builder.with_fullscreen(true);
    }

    let context = builder.build()?;

    let (seed, skip_intro) = parse_args();
    println!("RNG seed: {}", seed);

    let mut state = if skip_intro {
        State::new_game_state(25, seed, true, settings.bindings)
    } else {
        State::new_welcome_state(seed, settings.bindings)
    };

    state.log.entries.push("Welcome!".to_string());

    rltk::main_loop(context, state)
}