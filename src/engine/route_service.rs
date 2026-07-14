//! Route cost calculation service.

use crate::data::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Calculated route costs
#[derive(Debug, Clone)]
pub struct RouteCosts {
    pub fuel: u32,
    pub time: u32,
    pub risk: u32,
}

/// Route calculation service
pub struct RouteService;

impl RouteService {
    /// Calculate costs for a route type with all modifiers
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_route_costs(
        route: RouteType,
        constants: &ConstantsData,
        passenger_risk: u32,
        weather: Option<&WeatherCondition>,
        time_of_day: Option<&TimeOfDay>,
        hazards: &[EnvironmentalHazard],
        route_mastery: &HashMap<RouteType, u32>,
        passenger: Option<&Passenger>,
        skill_mods: &crate::engine::SkillModifiers,
    ) -> RouteCosts {
        // Base costs from constants
        let (base_fuel, base_time, base_risk) = match route {
            RouteType::Normal => (
                constants.game_constants.fuel_cost_normal,
                constants.game_constants.time_cost_normal,
                constants.game_constants.risk_normal,
            ),
            RouteType::Shortcut => (
                constants.game_constants.fuel_cost_shortcut,
                constants.game_constants.time_cost_shortcut,
                constants.game_constants.risk_shortcut,
            ),
            RouteType::Scenic => (
                constants.game_constants.fuel_cost_scenic,
                constants.game_constants.time_cost_scenic,
                constants.game_constants.risk_scenic,
            ),
            RouteType::Police => (
                constants.game_constants.fuel_cost_police,
                constants.game_constants.time_cost_police,
                constants.game_constants.risk_police,
            ),
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
                    WeatherEffectType::SupernaturalAttraction => {
                        risk += effect.value as f32 / 30.0;
                    }
                    WeatherEffectType::PassengerBehavior => {
                        if passenger.is_some() {
                            risk += effect.value as f32 / 50.0;
                            time += effect.value.max(0) as f32 / 4.0;
                        }
                    }
                    WeatherEffectType::RouteBlockage => {
                        if Self::weather_effect_applies_to_route(effect, route) {
                            time += effect.value.max(0) as f32 / 2.0;
                            risk = (risk + effect.value.max(0) as f32 / 10.0).min(5.0);
                        }
                    }
                    WeatherEffectType::RuleModification => {
                        risk += effect.value.max(0) as f32 / 50.0;
                    }
                }
            }

            // Heavy weather penalties
            if w.intensity == WeatherIntensity::Heavy {
                if route == RouteType::Shortcut
                    && matches!(
                        w.weather_type,
                        WeatherType::Rain | WeatherType::Fog | WeatherType::Snow
                    )
                {
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

        // Hazard effects, accumulated separately so the Reinforced Chassis skill
        // (hazard_mult) can soften only the hazard-inflicted portion.
        let mut hazard_fuel = 0.0;
        let mut hazard_time = 0.0;
        let mut hazard_risk = 0.0;
        for hazard in hazards {
            if hazard.blocks_route(route) {
                hazard_time += 12.0;
                hazard_fuel += 5.0;
                hazard_risk += 2.0;
            }
            if let Some(f) = hazard.effects.fuel_increase {
                hazard_fuel += f as f32;
            }
            if let Some(t) = hazard.effects.time_delay {
                hazard_time += t as f32;
            }
            if let Some(r) = hazard.effects.risk_increase {
                hazard_risk += r as f32;
            }
            if hazard.effects.forced_choice == Some(true) {
                hazard_time += 5.0;
                hazard_risk += 1.0;
            }
        }
        fuel += hazard_fuel * skill_mods.hazard_mult;
        time += hazard_time * skill_mods.hazard_mult;
        risk = (risk + hazard_risk * skill_mods.hazard_mult).min(5.0);

        // Route mastery bonuses
        if let Some(&mastery) = route_mastery.get(&route) {
            let fuel_reduction = (mastery / 3).min(4u32) as f32;
            let time_reduction = (mastery / 2).min(6u32) as f32;
            let risk_reduction = (mastery / 10).min(1u32) as f32;

            fuel = (fuel - fuel_reduction).max(constants.route_variations.minimum_fuel_cost as f32);
            time = (time - time_reduction).max(constants.route_variations.minimum_time_cost as f32);
            risk = (risk - risk_reduction).max(0.0);
        }

        // Fuel-efficiency skills (Hybrid Injection) scale the whole fuel cost.
        fuel = (fuel * skill_mods.fuel_cost_mult)
            .max(constants.route_variations.minimum_fuel_cost as f32);

        RouteCosts {
            fuel: fuel.round() as u32,
            time: time.round() as u32,
            risk: risk.round().clamp(0.0, 5.0) as u32,
        }
    }

    /// Generate 3 risk tags for a route context
    pub fn generate_risk_tags(
        route: RouteType,
        weather: Option<&WeatherCondition>,
        time_of_day: Option<&TimeOfDay>,
        _passenger: Option<&Passenger>,
        seed: Option<u64>,
    ) -> Vec<RiskTag> {
        let mut potential_risks = Vec::new();

        // Base risks by route type
        match route {
            RouteType::Normal => {
                potential_risks.push(RiskTag::HighTraffic);
                potential_risks.push(RiskTag::RoadConstruction);
                potential_risks.push(RiskTag::PolicePatrol);
            }
            RouteType::Shortcut => {
                potential_risks.push(RiskTag::Potholes);
                potential_risks.push(RiskTag::GangActivity);
                potential_risks.push(RiskTag::StrangeNoises);
            }
            RouteType::Scenic => {
                potential_risks.push(RiskTag::SlipperyRoads);
                potential_risks.push(RiskTag::SpiritualDisturbance);
                potential_risks.push(RiskTag::DenseFog);
            }
            RouteType::Police => {
                potential_risks.push(RiskTag::HighTraffic);
                potential_risks.push(RiskTag::SpiritualDisturbance); // Escort duty?
                potential_risks.push(RiskTag::PolicePatrol);
            }
        }

        // Weather risks
        if let Some(w) = weather {
            match w.weather_type {
                WeatherType::Rain | WeatherType::Thunderstorm => {
                    potential_risks.push(RiskTag::SlipperyRoads);
                    potential_risks.push(RiskTag::FlashFloods);
                }
                WeatherType::Fog => potential_risks.push(RiskTag::DenseFog),
                _ => {}
            }
        }

        // Time risks
        if let Some(t) = time_of_day {
            if matches!(t.phase, TimePhase::Night | TimePhase::Latenight) {
                potential_risks.push(RiskTag::StrangeNoises);
                potential_risks.push(RiskTag::SpiritualDisturbance);
            }
        }

        // Fill with generic risks if not enough
        let generics = [
            RiskTag::HighTraffic,
            RiskTag::RoadConstruction,
            RiskTag::Potholes,
        ];

        for r in generics.iter() {
            if !potential_risks.contains(r) {
                potential_risks.push(*r);
            }
        }

        if let Some(seed) = seed {
            potential_risks.sort_by_key(|risk| {
                let mut hasher = DefaultHasher::new();
                seed.hash(&mut hasher);
                risk.hash(&mut hasher);
                hasher.finish()
            });
        } else {
            macroquad_toolkit::rng::shuffle(&mut potential_risks);
        }

        // Ensure unique
        let mut selected = Vec::new();
        for r in potential_risks {
            if !selected.contains(&r) && selected.len() < 3 {
                selected.push(r);
            }
        }

        // Fallback if still < 3 (shouldn't happen with generics adding)
        while selected.len() < 3 {
            selected.push(RiskTag::HighTraffic); // Safe fallback
        }

        selected
    }

    fn weather_effect_applies_to_route(effect: &WeatherEffect, route: RouteType) -> bool {
        let Some(applies_to) = effect.applies_to.as_deref() else {
            return true;
        };

        let route_key = match route {
            RouteType::Normal => "normal",
            RouteType::Shortcut => "shortcut",
            RouteType::Scenic => "scenic",
            RouteType::Police => "police",
        };

        applies_to.eq_ignore_ascii_case(route_key) || applies_to.eq_ignore_ascii_case("all")
    }
}
