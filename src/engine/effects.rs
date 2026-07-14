//! Visual effects system

use macroquad::prelude::*;
use macroquad_toolkit::fx::{Particle, ParticleSystem};
use macroquad_toolkit::math::{pulse01, pulse_range};
use macroquad_toolkit::rng;

pub use macroquad_toolkit::fx::{ScreenFade, ScreenShake};

/// Ambient weather particle emitters (rain/snow/fog) built on the shared
/// pooled `macroquad_toolkit::fx::ParticleSystem`. Spawn-rate and appearance
/// tuning stays local to this game; pooling, integration, and rendering are
/// shared with every other game via the toolkit.
pub struct WeatherParticles {
    system: ParticleSystem,
}

impl WeatherParticles {
    pub fn new() -> Self {
        Self {
            system: ParticleSystem::new(),
        }
    }

    /// Spawn rain particles
    pub fn spawn_rain(&mut self, count: usize) {
        for _ in 0..count {
            let mut particle = Particle::new(
                Vec2::new(rng::gen_range(0.0, screen_width()), -10.0),
                Vec2::new(rng::gen_range(-20.0, -10.0), rng::gen_range(400.0, 600.0)),
                rng::gen_range(1.0, 2.0),
                rng::gen_range(1.0, 2.0),
                Color::new(0.5, 0.6, 0.8, 0.6),
            );
            particle.max_life = 2.0;
            self.system.spawn(particle);
        }
    }

    /// Spawn snow particles
    pub fn spawn_snow(&mut self, count: usize) {
        for _ in 0..count {
            let mut particle = Particle::new(
                Vec2::new(rng::gen_range(0.0, screen_width()), -10.0),
                Vec2::new(rng::gen_range(-30.0, 30.0), rng::gen_range(50.0, 100.0)),
                rng::gen_range(3.0, 5.0),
                rng::gen_range(2.0, 4.0),
                Color::new(1.0, 1.0, 1.0, 0.8),
            );
            particle.max_life = 5.0;
            self.system.spawn(particle);
        }
    }

    /// Spawn fog particles
    pub fn spawn_fog(&mut self, count: usize) {
        for _ in 0..count {
            let mut particle = Particle::new(
                Vec2::new(
                    rng::gen_range(0.0, screen_width()),
                    rng::gen_range(0.0, screen_height()),
                ),
                Vec2::new(rng::gen_range(-10.0, 10.0), 0.0),
                rng::gen_range(5.0, 10.0),
                rng::gen_range(40.0, 80.0),
                Color::new(0.7, 0.7, 0.7, 0.1),
            );
            particle.max_life = 10.0;
            self.system.spawn(particle);
        }
    }

    /// Update all particles
    pub fn update(&mut self, dt: f32) {
        self.system.update(dt);
    }

    /// Draw all particles
    pub fn draw(&self) {
        self.system.draw();
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.system.clear();
    }

    /// Get particle count
    pub fn count(&self) -> usize {
        self.system.count()
    }
}

impl Default for WeatherParticles {
    fn default() -> Self {
        Self::new()
    }
}

/// Glitch effect for game over
pub fn draw_glitch_effect(intensity: f32) {
    let width = screen_width();
    let height = screen_height();

    // Random horizontal lines
    for _ in 0..(intensity * 20.0) as i32 {
        let y = rng::gen_range(0.0, height);
        let thickness = rng::gen_range(1.0, 5.0);
        let offset = rng::gen_range(-intensity * 50.0, intensity * 50.0);

        draw_rectangle(
            offset,
            y,
            width,
            thickness,
            Color::new(
                rng::gen_range(0.5, 1.0),
                rng::gen_range(0.0, 0.3),
                rng::gen_range(0.0, 0.3),
                rng::gen_range(0.3, 0.7),
            ),
        );
    }

    // RGB split effect
    if intensity > 0.5 {
        let offset = intensity * 5.0;
        draw_rectangle(0.0, 0.0, width, height, Color::new(1.0, 0.0, 0.0, 0.05));
        draw_rectangle(offset, 0.0, width, height, Color::new(0.0, 1.0, 0.0, 0.05));
        draw_rectangle(-offset, 0.0, width, height, Color::new(0.0, 0.0, 1.0, 0.05));
    }
}

/// Draw fog overlay
pub fn draw_fog_overlay(intensity: f32) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.6, 0.6, 0.7, intensity * 0.3),
    );
}

/// Draw danger overlay - subtle red pulsing hue that intensifies with risk
/// intensity: 0.0 to 1.0 (0 = no danger, 1 = maximum danger)
pub fn draw_danger_overlay(intensity: f32) {
    if intensity <= 0.0 {
        return;
    }

    let width = screen_width();
    let height = screen_height();

    // Slow pulse effect for the red overlay
    let pulse = pulse_range(1.5, 0.7, 1.0);
    let alpha = intensity * 0.15 * pulse;

    // Red danger tint
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.8, 0.1, 0.1, alpha));

    // Edge vignette gets more intense with danger
    if intensity > 0.3 {
        let edge_alpha = (intensity - 0.3) * 0.2;
        // Top edge
        draw_rectangle(0.0, 0.0, width, 40.0, Color::new(0.5, 0.0, 0.0, edge_alpha));
        // Bottom edge
        draw_rectangle(
            0.0,
            height - 40.0,
            width,
            40.0,
            Color::new(0.5, 0.0, 0.0, edge_alpha),
        );
    }
}

/// Draw tension vignette - darkening around edges that intensifies with stress
/// intensity: 0.0 to 1.0 (0 = calm, 1 = extreme tension)
pub fn draw_tension_vignette(intensity: f32) {
    if intensity <= 0.0 {
        return;
    }

    let width = screen_width();
    let height = screen_height();

    // Vignette intensity scales with stress
    let vignette_strength = intensity * 0.4;

    // Draw graduated darkness at edges (simulated vignette with rectangles)
    let edge_count = 4;
    for i in 0..edge_count {
        let layer = i as f32 / edge_count as f32;
        let thickness = 30.0 + layer * 40.0;
        let alpha = vignette_strength * (1.0 - layer) * 0.5;

        // Top
        draw_rectangle(0.0, 0.0, width, thickness, Color::new(0.0, 0.0, 0.0, alpha));
        // Bottom
        draw_rectangle(
            0.0,
            height - thickness,
            width,
            thickness,
            Color::new(0.0, 0.0, 0.0, alpha),
        );
        // Left
        draw_rectangle(
            0.0,
            0.0,
            thickness,
            height,
            Color::new(0.0, 0.0, 0.0, alpha),
        );
        // Right
        draw_rectangle(
            width - thickness,
            0.0,
            thickness,
            height,
            Color::new(0.0, 0.0, 0.0, alpha),
        );
    }

    // Breathing effect when very stressed (subtle screen pulse)
    if intensity > 0.6 {
        let breath_alpha = pulse01(0.8) * (intensity - 0.6) * 0.1;
        draw_rectangle(
            0.0,
            0.0,
            width,
            height,
            Color::new(0.0, 0.0, 0.0, breath_alpha),
        );
    }
}
