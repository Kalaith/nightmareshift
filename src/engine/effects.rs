//! Visual effects system

use macroquad::prelude::*;

/// Screen transition effects
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionState {
    None,
    FadeIn,
}

pub struct ScreenTransition {
    pub state: TransitionState,
    pub alpha: f32,
    pub duration: f32,
    elapsed: f32,
}

impl ScreenTransition {
    pub fn new() -> Self {
        Self {
            state: TransitionState::None,
            alpha: 0.0,
            duration: 0.3,
            elapsed: 0.0,
        }
    }

    /// Start fade in transition
    pub fn fade_in(&mut self) {
        self.state = TransitionState::FadeIn;
        self.alpha = 1.0;
        self.elapsed = 0.0;
    }

    /// Update transition
    pub fn update(&mut self, dt: f32) {
        if self.state == TransitionState::None {
            return;
        }

        self.elapsed += dt;
        let progress = (self.elapsed / self.duration).min(1.0);

        match self.state {
            TransitionState::FadeIn => {
                self.alpha = 1.0 - progress;
                if progress >= 1.0 {
                    self.state = TransitionState::None;
                    self.alpha = 0.0;
                }
            }
            TransitionState::None => {}
        }
    }

    /// Draw the transition overlay
    pub fn draw(&self) {
        if self.alpha > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, self.alpha),
            );
        }
    }
}

/// Screen shake effect
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    elapsed: f32,
}

impl ScreenShake {
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            elapsed: 0.0,
        }
    }

    /// Trigger screen shake
    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = duration;
        self.elapsed = 0.0;
    }

    /// Update shake
    pub fn update(&mut self, dt: f32) {
        if self.elapsed < self.duration {
            self.elapsed += dt;
        } else {
            self.intensity = 0.0;
        }
    }

    /// Get current offset
    pub fn get_offset(&self) -> (f32, f32) {
        if self.elapsed >= self.duration {
            return (0.0, 0.0);
        }

        let decay = 1.0 - (self.elapsed / self.duration);
        let current_intensity = self.intensity * decay;

        (
            (rand::gen_range(-1.0, 1.0) * current_intensity) as f32,
            (rand::gen_range(-1.0, 1.0) * current_intensity) as f32,
        )
    }
}

/// Particle for visual effects
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

impl Particle {
    pub fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.life -= dt;
    }

    pub fn is_dead(&self) -> bool {
        self.life <= 0.0
    }

    pub fn draw(&self) {
        let alpha = (self.life / self.max_life).clamp(0.0, 1.0);
        let mut color = self.color;
        color.a = alpha;
        draw_circle(self.x, self.y, self.size, color);
    }
}

/// Particle system
pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    /// Spawn rain particles
    pub fn spawn_rain(&mut self, count: usize) {
        for _ in 0..count {
            self.particles.push(Particle {
                x: rand::gen_range(0.0, screen_width()),
                y: -10.0,
                vx: rand::gen_range(-20.0, -10.0),
                vy: rand::gen_range(400.0, 600.0),
                life: rand::gen_range(1.0, 2.0),
                max_life: 2.0,
                size: rand::gen_range(1.0, 2.0),
                color: Color::new(0.5, 0.6, 0.8, 0.6),
            });
        }
    }

    /// Spawn snow particles
    pub fn spawn_snow(&mut self, count: usize) {
        for _ in 0..count {
            self.particles.push(Particle {
                x: rand::gen_range(0.0, screen_width()),
                y: -10.0,
                vx: rand::gen_range(-30.0, 30.0),
                vy: rand::gen_range(50.0, 100.0),
                life: rand::gen_range(3.0, 5.0),
                max_life: 5.0,
                size: rand::gen_range(2.0, 4.0),
                color: Color::new(1.0, 1.0, 1.0, 0.8),
            });
        }
    }

    /// Spawn fog particles
    pub fn spawn_fog(&mut self, count: usize) {
        for _ in 0..count {
            self.particles.push(Particle {
                x: rand::gen_range(0.0, screen_width()),
                y: rand::gen_range(0.0, screen_height()),
                vx: rand::gen_range(-10.0, 10.0),
                vy: 0.0,
                life: rand::gen_range(5.0, 10.0),
                max_life: 10.0,
                size: rand::gen_range(40.0, 80.0),
                color: Color::new(0.7, 0.7, 0.7, 0.1),
            });
        }
    }

    /// Update all particles
    pub fn update(&mut self, dt: f32) {
        for particle in &mut self.particles {
            particle.update(dt);
        }
        self.particles.retain(|p| !p.is_dead());
    }

    /// Draw all particles
    pub fn draw(&self) {
        for particle in &self.particles {
            particle.draw();
        }
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// Get particle count
    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

/// Glitch effect for game over
pub fn draw_glitch_effect(intensity: f32) {
    let width = screen_width();
    let height = screen_height();

    // Random horizontal lines
    for _ in 0..(intensity * 20.0) as i32 {
        let y = rand::gen_range(0.0, height);
        let thickness = rand::gen_range(1.0, 5.0);
        let offset = rand::gen_range(-intensity * 50.0, intensity * 50.0);

        draw_rectangle(
            offset,
            y,
            width,
            thickness,
            Color::new(
                rand::gen_range(0.5, 1.0),
                rand::gen_range(0.0, 0.3),
                rand::gen_range(0.0, 0.3),
                rand::gen_range(0.3, 0.7),
            ),
        );
    }

    // RGB split effect
    if intensity > 0.5 {
        let offset = intensity * 5.0;
        draw_rectangle(
            0.0,
            0.0,
            width,
            height,
            Color::new(1.0, 0.0, 0.0, 0.05),
        );
        draw_rectangle(
            offset,
            0.0,
            width,
            height,
            Color::new(0.0, 1.0, 0.0, 0.05),
        );
        draw_rectangle(
            -offset,
            0.0,
            width,
            height,
            Color::new(0.0, 0.0, 1.0, 0.05),
        );
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
