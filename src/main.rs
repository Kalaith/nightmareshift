//! Nightmare Shift - A horror-themed taxi driving survival game.
//!
//! Drive supernatural passengers through the night, follow mysterious rules,
//! and try to survive until dawn.

#![allow(clippy::too_many_arguments)]

mod bot;
mod data;
mod engine;
mod game;
mod screens;
mod state;
mod ui;

use game::Game;
use macroquad::prelude::*;
use macroquad_toolkit::capture;

const DEFAULT_WINDOW_WIDTH: i32 = 1920;
const DEFAULT_WINDOW_HEIGHT: i32 = 1080;

fn window_conf() -> Conf {
    // Built by hand (not capture::capture_window_conf) to keep sample_count: 0
    // and the always-off high_dpi that this game already relied on.
    Conf {
        window_title: "Nightmare Shift".to_string(),
        window_width: capture::env_i32("NIGHTMARE_SHIFT_WINDOW_WIDTH", DEFAULT_WINDOW_WIDTH),
        window_height: capture::env_i32("NIGHTMARE_SHIFT_WINDOW_HEIGHT", DEFAULT_WINDOW_HEIGHT),
        window_resizable: true,
        sample_count: 0,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    // Screenshot harness: when NIGHTMARE_SHIFT_CAPTURE_PATH is set, seed a
    // scene, simulate deterministic frames, write a PNG, and exit.
    if let Some(config) = capture::CaptureConfig::from_env("NIGHTMARE_SHIFT") {
        game.begin_capture_scene(&config.scene);
        capture::run_capture(&config, |_dt| {
            game.update();
            game.handle_input();
            let action = game.draw();
            game.handle_ui_action(action);
            game.handle_playtest_bot();
        })
        .await;
        return;
    }

    loop {
        game.update();
        game.handle_input();
        let action = game.draw();
        game.handle_ui_action(action);
        game.handle_playtest_bot();
        next_frame().await;
    }
}
