//! Weather and environment service.
//!
//! Weather generation lives in `conditions`, time-of-day and season in
//! `calendar`, and hazard spawning in `hazards`. Each extends `WeatherService`
//! with a further `impl` block.

mod calendar;
mod conditions;
mod hazards;

/// Weather generation and update service
pub struct WeatherService;
