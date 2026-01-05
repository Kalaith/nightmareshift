//! UI core types and utilities.

use macroquad::prelude::*;

/// Theme colors for the game (game-specific, extends toolkit colors)
pub mod colors {
    use macroquad::prelude::Color;

    // Standard UI colors - prefer using toolkit's dark:: palette:
    // dark::BACKGROUND, dark::PANEL, dark::TEXT, dark::ACCENT, etc.
    
    // Game-specific colors
    pub const PANEL_BG: Color = Color::new(0.176, 0.176, 0.267, 1.0); // #2d2d44
    pub const SUCCESS_BG: Color = Color::new(0.176, 0.302, 0.176, 1.0); // #2d4d2d
    
    pub const TEXT_PRIMARY: Color = Color::new(1.0, 1.0, 1.0, 1.0); // White
    pub const TEXT_SECONDARY: Color = Color::new(0.667, 0.667, 0.667, 1.0); // #aaaaaa
    pub const TEXT_MUTED: Color = Color::new(0.533, 0.533, 0.533, 1.0); // #888888
    
    pub const ACCENT_PRIMARY: Color = Color::new(0.306, 0.804, 0.769, 1.0); // #4ecdc4
    pub const ACCENT_DANGER: Color = Color::new(1.0, 0.420, 0.420, 1.0); // #ff6b6b
    pub const ACCENT_WARNING: Color = Color::new(0.953, 0.612, 0.071, 1.0); // #f39c12
    pub const ACCENT_GOLD: Color = Color::new(1.0, 0.843, 0.0, 1.0); // #ffd700
    pub const ACCENT_SKY: Color = Color::new(0.529, 0.808, 0.922, 1.0); // #87ceeb
    
    pub const FUEL_GOOD: Color = Color::new(0.267, 1.0, 0.267, 1.0); // #44ff44
    pub const FUEL_LOW: Color = Color::new(1.0, 0.667, 0.0, 1.0); // #ffaa00
    pub const FUEL_CRITICAL: Color = Color::new(1.0, 0.267, 0.267, 1.0); // #ff4444
}

/// Standard spacing values
pub mod spacing {
    pub const PADDING_SM: f32 = 8.0;
    pub const PADDING_MD: f32 = 16.0;
    pub const PADDING_LG: f32 = 24.0;
    pub const PADDING_XL: f32 = 32.0;
}

/// Layout constants for consistent UI positioning
pub mod layout {
    // Status bar
    pub const STATUS_BAR_HEIGHT: f32 = 55.0;
    pub const STATUS_BAR_TEXT_Y: f32 = 35.0;
    pub const STATUS_ITEM_SPACING: f32 = 100.0;
    pub const STATUS_EARNINGS_SPACING: f32 = 120.0;
    
    // Buttons
    pub const BUTTON_WIDTH_SM: f32 = 100.0;
    pub const BUTTON_WIDTH_MD: f32 = 150.0;
    pub const BUTTON_WIDTH_LG: f32 = 200.0;
    pub const BUTTON_HEIGHT: f32 = 40.0;
    pub const BUTTON_SPACING: f32 = 20.0;
    
    // Cards and panels
    pub const CARD_WIDTH: f32 = 400.0;
    pub const CARD_HEIGHT: f32 = 350.0;
    pub const PANEL_MIN_WIDTH: f32 = 300.0;
    
    // Game timing defaults
    pub const DEFAULT_START_HOUR: u32 = 20; // 8 PM
    pub const DEFAULT_MONTH: u32 = 10; // October
    
    // Fuel thresholds
    pub const FUEL_CRITICAL_THRESHOLD: f32 = 10.0;
    pub const FUEL_LOW_THRESHOLD: f32 = 20.0;
    pub const FUEL_MEDIUM_THRESHOLD: f32 = 40.0;
    pub const MINIMUM_FUEL_FOR_RIDE: f32 = 5.0;
}

/// Font sizes
pub mod fonts {
    pub const SIZE_XS: f32 = 12.0;
    pub const SIZE_SM: f32 = 14.0;
    pub const SIZE_MD: f32 = 16.0;
    pub const SIZE_LG: f32 = 20.0;
    pub const SIZE_XL: f32 = 24.0;
}

/// A positioned rectangle for UI layouts
#[derive(Debug, Clone, Copy)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl UiRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn centered_x(y: f32, w: f32, h: f32) -> Self {
        Self {
            x: (screen_width() - w) / 2.0,
            y,
            w,
            h,
        }
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.w / 2.0
    }

    pub fn inset(&self, amount: f32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            w: self.w - amount * 2.0,
            h: self.h - amount * 2.0,
        }
    }
}

/// Draw a rounded rectangle panel
pub fn draw_panel(rect: UiRect, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
}

/// Draw a panel with border
pub fn draw_panel_bordered(rect: UiRect, bg: Color, border: Color, border_width: f32) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, border_width, border);
}

/// Get color for fuel level
pub fn get_fuel_color(fuel: f32) -> Color {
    if fuel <= layout::FUEL_CRITICAL_THRESHOLD {
        colors::FUEL_CRITICAL // Red - Critical
    } else if fuel <= layout::FUEL_LOW_THRESHOLD {
        colors::FUEL_LOW // Red/Orange - Low
    } else if fuel <= layout::FUEL_MEDIUM_THRESHOLD {
        colors::ACCENT_WARNING // Yellow - Medium
    } else {
        colors::FUEL_GOOD // Green - Good
    }
}
