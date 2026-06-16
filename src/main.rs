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

const DEFAULT_WINDOW_WIDTH: i32 = 1920;
const DEFAULT_WINDOW_HEIGHT: i32 = 1080;

fn window_conf() -> Conf {
    Conf {
        window_title: "Nightmare Shift".to_string(),
        window_width: DEFAULT_WINDOW_WIDTH,
        window_height: DEFAULT_WINDOW_HEIGHT,
        window_resizable: true,
        sample_count: 0,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        game.update();
        game.handle_input();
        let action = game.draw();
        game.handle_ui_action(action);
        game.handle_playtest_bot();
        next_frame().await;
    }
}
