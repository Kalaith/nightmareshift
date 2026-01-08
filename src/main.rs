//! Nightmare Shift - A horror-themed taxi driving survival game.
//!
//! Drive supernatural passengers through the night, follow mysterious rules,
//! and try to survive until dawn.

mod data;
mod engine;
mod screens;
mod state;
mod ui;
mod game;

use macroquad::prelude::*;
use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Nightmare Shift".to_string(),
        window_width: 800,
        window_height: 600,
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
        next_frame().await;
    }
}
