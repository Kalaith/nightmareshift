//! Route cost calculation service.

use crate::data::*;
use std::collections::HashMap;

/// Calculated route costs
#[derive(Debug, Clone, Copy)]
pub struct RouteCosts {
    pub fuel: u32,
    pub time: u32,
    pub risk: u32,
}

/// Route calculation service
pub struct RouteService;

impl RouteService {
    /// Calculate costs for a route type with all modifiers
    pub fn calculate_route_costs(
        route: RouteType,
        constants: &ConstantsData,
        passenger_risk: u32,
        weather: Option<&WeatherCondition>,
        time_of_day: Option<&TimeOfDay>,
        hazards: &[EnvironmentalHazard],
        route_mastery: &HashMap<RouteType, u32>,
        passenger: Option<&Passenger>,
    ) -> RouteCosts {
        // Base costs from constants
        let (base_fuel, base_time, base_risk) = match route {
            RouteType::Normal => (constants.game_constants.fuel_cost_normal, constants.game_constants.time_cost_normal, constants.game_constants.risk_normal),
            RouteType::Shortcut => (constants.game_constants.fuel_cost_shortcut, constants.game_constants.time_cost_shortcut, constants.game_constants.risk_shortcut),
            RouteType::Scenic => (constants.game_constants.fuel_cost_scenic, constants.game_constants.time_cost_scenic, constants.game_constants.risk_scenic),
            RouteType::Police => (constants.game_constants.fuel_cost_police, constants.game_constants.time_cost_police, constants.game_constants.risk_police),
        };

        let mut fuel = base_fuel as f32;
        let mut time = base_time as f32;
        let mut risk = base_risk as f32 + (passenger_risk as f32 - 1.0);

        // Apply weather effects
        if let Some(w) = weather {
            for effect in &w.effects {
                match effect.effect_type {
                    WeatherEffectType::FuelConsumption => fuel *= 1.0 + effect.value as f32 / 100.0,
                    WeatherEffectType::TimeDelay => time *= 1.0 + effect.value as f32 / 100.0,
                    WeatherEffectType::VisibilityReduction => risk += effect.value as f32 / 100.0,
                    _ => {}
                }
            }

            // Heavy weather penalties
            if w.intensity == WeatherIntensity::Heavy {
                if route == RouteType::Shortcut && matches!(w.weather_type, WeatherType::Rain | WeatherType::Fog | WeatherType::Snow) {
                    fuel += 8.0;
                    time += 10.0;
                    risk = (risk + 2.0).min(5.0);
                }
                if route == RouteType::Scenic && w.weather_type == WeatherType::Thunderstorm {
                    fuel += 5.0;
                    time += 15.0;
                    risk = (risk + 1.0).min(5.0);
                }
            }
        }

        // Time of day effects
        if let Some(tod) = time_of_day {
            if matches!(tod.phase, TimePhase::Night | TimePhase::Latenight) {
                risk += 0.2;
            }
            if tod.ambient_light < 30 {
                fuel *= 1.1; // Headlights
            }
            if route == RouteType::Scenic && tod.phase == TimePhase::Latenight {
                fuel += 3.0;
                time += 8.0;
                risk = (risk + 1.0).min(5.0);
            }
        }

        // Passenger fear penalty
        if let Some(p) = passenger {
            if p.fears_route(route) {
                fuel += 2.0;
                time += 3.0;
                risk = (risk + 1.0).min(5.0);
            }
        }

        // Hazard effects
        for hazard in hazards {
            if hazard.blocks_route(route) {
                risk = (risk + 2.0).min(5.0);
            }
            if let Some(f) = hazard.effects.fuel_increase {
                fuel += f as f32;
            }
            if let Some(t) = hazard.effects.time_delay {
                time += t as f32;
            }
            if let Some(r) = hazard.effects.risk_increase {
                risk += r as f32;
            }
        }

        // Route mastery bonuses
        if let Some(&mastery) = route_mastery.get(&route) {
            let fuel_reduction = (mastery / 3).min(4) as f32;
            let time_reduction = (mastery / 2).min(6) as f32;
            let risk_reduction = (mastery / 10).min(1) as f32;
            
            fuel = (fuel - fuel_reduction).max(constants.route_variations.minimum_fuel_cost as f32);
            time = (time - time_reduction).max(constants.route_variations.minimum_time_cost as f32);
            risk = (risk - risk_reduction).max(0.0);
        }

        RouteCosts {
            fuel: fuel.round() as u32,
            time: time.round() as u32,
            risk: risk.round().clamp(0.0, 5.0) as u32,
        }
    }
}
