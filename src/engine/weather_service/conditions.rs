//! Weather generation: picking a seasonal weather type, deriving its effects,
//! visibility, description, and duration, and drifting it over time.

use crate::data::*;

use super::WeatherService;

impl WeatherService {
    /// Generate initial weather for a shift
    pub fn generate_initial_weather(season: &Season, current_time: f64) -> WeatherCondition {
        let weather_types = Self::get_seasonal_weather_types(season);
        let weather_type =
            *macroquad_toolkit::rng::choose(&weather_types).unwrap_or(&WeatherType::Clear);

        Self::create_weather_condition(weather_type, season, current_time)
    }

    /// Get available weather types for a season
    fn get_seasonal_weather_types(season: &Season) -> Vec<WeatherType> {
        match season.season_type {
            SeasonType::Spring => vec![
                WeatherType::Clear,
                WeatherType::Rain,
                WeatherType::Thunderstorm,
                WeatherType::Wind,
            ],
            SeasonType::Summer => vec![
                WeatherType::Clear,
                WeatherType::Thunderstorm,
                WeatherType::Wind,
            ],
            SeasonType::Fall => vec![
                WeatherType::Clear,
                WeatherType::Rain,
                WeatherType::Fog,
                WeatherType::Wind,
            ],
            SeasonType::Winter => vec![
                WeatherType::Clear,
                WeatherType::Snow,
                WeatherType::Fog,
                WeatherType::Wind,
            ],
        }
    }

    /// Create a weather condition
    fn create_weather_condition(
        weather_type: WeatherType,
        season: &Season,
        current_time: f64,
    ) -> WeatherCondition {
        let intensity = Self::get_random_intensity(weather_type);
        let effects = Self::get_weather_effects(weather_type, intensity);
        let visibility = Self::calculate_visibility(weather_type, intensity);
        let duration = Self::get_weather_duration(weather_type, intensity, season);

        WeatherCondition {
            weather_type,
            intensity,
            visibility,
            description: Self::get_weather_description(weather_type, intensity),
            icon: Self::get_weather_icon(weather_type, intensity),
            effects,
            duration,
            start_time: current_time,
        }
    }

    /// Get random intensity (clear is always light)
    fn get_random_intensity(weather_type: WeatherType) -> WeatherIntensity {
        if weather_type == WeatherType::Clear {
            return WeatherIntensity::Light;
        }

        let rand = macroquad_toolkit::rng::rand();
        if rand < 0.5 {
            WeatherIntensity::Light
        } else if rand < 0.8 {
            WeatherIntensity::Moderate
        } else {
            WeatherIntensity::Heavy
        }
    }

    /// Get weather effects
    fn get_weather_effects(
        weather_type: WeatherType,
        intensity: WeatherIntensity,
    ) -> Vec<WeatherEffect> {
        let mult = match intensity {
            WeatherIntensity::Light => 1.0,
            WeatherIntensity::Moderate => 1.5,
            WeatherIntensity::Heavy => 2.0,
        };

        let mut effects = Vec::new();

        match weather_type {
            WeatherType::Rain => {
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::VisibilityReduction,
                    value: (10.0 * mult) as i32,
                    description: "Reduced visibility".to_string(),
                    applies_to: None,
                });
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::FuelConsumption,
                    value: (5.0 * mult) as i32,
                    description: "Increased fuel use".to_string(),
                    applies_to: None,
                });
            }
            WeatherType::Fog => {
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::VisibilityReduction,
                    value: (20.0 * mult) as i32,
                    description: "Poor visibility".to_string(),
                    applies_to: None,
                });
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::TimeDelay,
                    value: (15.0 * mult) as i32,
                    description: "Slower driving".to_string(),
                    applies_to: None,
                });
            }
            WeatherType::Snow => {
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::VisibilityReduction,
                    value: (15.0 * mult) as i32,
                    description: "Snow obscures view".to_string(),
                    applies_to: None,
                });
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::FuelConsumption,
                    value: (10.0 * mult) as i32,
                    description: "Cold weather fuel usage".to_string(),
                    applies_to: None,
                });
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::TimeDelay,
                    value: (20.0 * mult) as i32,
                    description: "Careful driving required".to_string(),
                    applies_to: None,
                });
            }
            WeatherType::Thunderstorm => {
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::VisibilityReduction,
                    value: (25.0 * mult) as i32,
                    description: "Heavy rain and darkness".to_string(),
                    applies_to: None,
                });
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::SupernaturalAttraction,
                    value: (30.0 * mult) as i32,
                    description: "Supernatural activity increases".to_string(),
                    applies_to: None,
                });
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::PassengerBehavior,
                    value: (20.0 * mult) as i32,
                    description: "Passengers more agitated".to_string(),
                    applies_to: None,
                });
            }
            WeatherType::Wind => {
                effects.push(WeatherEffect {
                    effect_type: WeatherEffectType::FuelConsumption,
                    value: (8.0 * mult) as i32,
                    description: "Fighting headwinds".to_string(),
                    applies_to: None,
                });
                if intensity == WeatherIntensity::Heavy {
                    effects.push(WeatherEffect {
                        effect_type: WeatherEffectType::RouteBlockage,
                        value: 20,
                        description: "Some routes blocked by debris".to_string(),
                        applies_to: None,
                    });
                }
            }
            WeatherType::Clear => {}
        }

        effects
    }

    /// Calculate visibility percentage
    fn calculate_visibility(weather_type: WeatherType, intensity: WeatherIntensity) -> u32 {
        match weather_type {
            WeatherType::Fog => match intensity {
                WeatherIntensity::Light => 60,
                WeatherIntensity::Moderate => 30,
                WeatherIntensity::Heavy => 10,
            },
            WeatherType::Rain | WeatherType::Thunderstorm => match intensity {
                WeatherIntensity::Light => 80,
                WeatherIntensity::Moderate => 60,
                WeatherIntensity::Heavy => 40,
            },
            WeatherType::Snow => match intensity {
                WeatherIntensity::Light => 70,
                WeatherIntensity::Moderate => 50,
                WeatherIntensity::Heavy => 25,
            },
            _ => 100,
        }
    }

    /// Get weather description
    fn get_weather_description(weather_type: WeatherType, intensity: WeatherIntensity) -> String {
        match (weather_type, intensity) {
            (WeatherType::Clear, _) => "Clear skies with good visibility".to_string(),
            (WeatherType::Rain, WeatherIntensity::Light) => {
                "Light drizzle dampens the streets".to_string()
            }
            (WeatherType::Rain, WeatherIntensity::Moderate) => {
                "Steady rain creates puddles and reflections".to_string()
            }
            (WeatherType::Rain, WeatherIntensity::Heavy) => {
                "Heavy downpour reduces visibility significantly".to_string()
            }
            (WeatherType::Fog, WeatherIntensity::Light) => {
                "Thin fog creates an eerie atmosphere".to_string()
            }
            (WeatherType::Fog, WeatherIntensity::Moderate) => {
                "Dense fog obscures distant objects".to_string()
            }
            (WeatherType::Fog, WeatherIntensity::Heavy) => {
                "Thick fog makes driving treacherous".to_string()
            }
            (WeatherType::Snow, WeatherIntensity::Light) => {
                "Light snowfall dusts the ground".to_string()
            }
            (WeatherType::Snow, WeatherIntensity::Moderate) => {
                "Steady snow accumulates on roads".to_string()
            }
            (WeatherType::Snow, WeatherIntensity::Heavy) => {
                "Heavy snowstorm creates whiteout conditions".to_string()
            }
            (WeatherType::Thunderstorm, WeatherIntensity::Light) => {
                "Distant thunder rumbles ominously".to_string()
            }
            (WeatherType::Thunderstorm, WeatherIntensity::Moderate) => {
                "Lightning illuminates the dark clouds".to_string()
            }
            (WeatherType::Thunderstorm, WeatherIntensity::Heavy) => {
                "Violent thunderstorm rages overhead".to_string()
            }
            (WeatherType::Wind, WeatherIntensity::Light) => {
                "Gentle breeze stirs the air".to_string()
            }
            (WeatherType::Wind, WeatherIntensity::Moderate) => {
                "Strong winds rock the vehicle".to_string()
            }
            (WeatherType::Wind, WeatherIntensity::Heavy) => {
                "Powerful gusts threaten to push cars off course".to_string()
            }
        }
    }

    /// Get weather icon
    fn get_weather_icon(weather_type: WeatherType, intensity: WeatherIntensity) -> String {
        match weather_type {
            WeatherType::Clear => "☀️".to_string(),
            WeatherType::Rain => if intensity == WeatherIntensity::Heavy {
                "🌧️"
            } else {
                "🌦️"
            }
            .to_string(),
            WeatherType::Fog => "🌫️".to_string(),
            WeatherType::Snow => if intensity == WeatherIntensity::Heavy {
                "❄️"
            } else {
                "🌨️"
            }
            .to_string(),
            WeatherType::Thunderstorm => "⛈️".to_string(),
            WeatherType::Wind => "💨".to_string(),
        }
    }

    /// Get weather duration in minutes
    fn get_weather_duration(
        weather_type: WeatherType,
        intensity: WeatherIntensity,
        season: &Season,
    ) -> u32 {
        let base = match weather_type {
            WeatherType::Clear => 60,
            WeatherType::Rain => 30,
            WeatherType::Fog => 45,
            WeatherType::Snow => 40,
            WeatherType::Thunderstorm => 20,
            WeatherType::Wind => 35,
        };

        let intensity_mult = match intensity {
            WeatherIntensity::Light => 1.5,
            WeatherIntensity::Moderate => 1.0,
            WeatherIntensity::Heavy => 0.7,
        };

        let season_mult = if season.season_type == SeasonType::Winter
            && matches!(weather_type, WeatherType::Snow | WeatherType::Fog)
        {
            1.3
        } else {
            1.0
        };

        (base as f32 * intensity_mult * season_mult) as u32
    }

    /// Update weather over time
    pub fn update_weather(
        current: &WeatherCondition,
        season: &Season,
        current_time: f64,
    ) -> WeatherCondition {
        let elapsed = current_time - current.start_time;
        let duration_secs = current.duration as f64 * 60.0;

        // Check if weather should change
        if elapsed >= duration_secs {
            let change_chance: f32 = 0.3;
            if macroquad_toolkit::rng::chance(change_chance) {
                return Self::generate_initial_weather(season, current_time);
            }
        }

        // Check for intensity change
        if macroquad_toolkit::rng::chance(0.1) {
            let new_intensity = Self::get_random_intensity(current.weather_type);
            if new_intensity != current.intensity {
                return WeatherCondition {
                    intensity: new_intensity,
                    effects: Self::get_weather_effects(current.weather_type, new_intensity),
                    description: Self::get_weather_description(current.weather_type, new_intensity),
                    visibility: Self::calculate_visibility(current.weather_type, new_intensity),
                    icon: Self::get_weather_icon(current.weather_type, new_intensity),
                    ..current.clone()
                };
            }
        }

        current.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::{load_constants, load_passengers};
    use crate::engine::{RouteCosts, RouteService, SkillModifiers};
    use std::collections::HashMap;

    /// Route costs under a given sky, everything else held equal.
    ///
    /// The condition is built here rather than through
    /// `create_weather_condition`, which rolls a random intensity — the point
    /// is to compare the skies, so the intensity is pinned.
    fn costs_under(weather_type: WeatherType) -> RouteCosts {
        let constants = load_constants();
        let intensity = WeatherIntensity::Moderate;
        let weather = WeatherCondition {
            weather_type,
            intensity,
            visibility: WeatherService::calculate_visibility(weather_type, intensity),
            description: String::new(),
            icon: String::new(),
            effects: WeatherService::get_weather_effects(weather_type, intensity),
            duration: 60,
            start_time: 0.0,
        };
        let passenger = load_passengers().into_iter().next();
        RouteService::calculate_route_costs(
            RouteType::Normal,
            &constants,
            2,
            Some(&weather),
            None,
            &[],
            &HashMap::new(),
            passenger.as_ref(),
            &SkillModifiers::default(),
        )
    }

    /// Clear weather must cost nothing extra, or there is no baseline to be
    /// worse than.
    #[test]
    fn clear_weather_is_the_baseline() {
        let clear = costs_under(WeatherType::Clear);
        for worse in [
            WeatherType::Rain,
            WeatherType::Fog,
            WeatherType::Snow,
            WeatherType::Thunderstorm,
        ] {
            let costs = costs_under(worse);
            assert!(
                costs.fuel >= clear.fuel && costs.time >= clear.time,
                "{worse:?} is cheaper than clear: {costs:?} against {clear:?}"
            );
        }
    }

    /// The weather types must actually differ. Six skies that all cost the
    /// same would make the forecast on the status bar decoration, and route
    /// risk — which has real consequences — blind to it.
    #[test]
    fn the_skies_are_not_all_the_same() {
        let profiles: Vec<(WeatherType, RouteCosts)> = [
            WeatherType::Clear,
            WeatherType::Rain,
            WeatherType::Fog,
            WeatherType::Snow,
            WeatherType::Thunderstorm,
            WeatherType::Wind,
        ]
        .into_iter()
        .map(|kind| (kind, costs_under(kind)))
        .collect();

        let distinct: std::collections::HashSet<(u32, u32, u32)> = profiles
            .iter()
            .map(|(_, c)| (c.fuel, c.time, c.risk))
            .collect();

        assert!(
            distinct.len() >= 4,
            "only {} distinct cost profiles across six skies: {:?}",
            distinct.len(),
            profiles
                .iter()
                .map(|(k, c)| (format!("{k:?}"), c.fuel, c.time, c.risk))
                .collect::<Vec<_>>()
        );
    }

    /// A thunderstorm is the worst sky the game has — it is the only one that
    /// draws the supernatural and unsettles the passenger — so it must not
    /// price below a shower.
    #[test]
    fn a_thunderstorm_outweighs_a_shower() {
        let storm = costs_under(WeatherType::Thunderstorm);
        let rain = costs_under(WeatherType::Rain);
        assert!(
            storm.risk >= rain.risk,
            "a thunderstorm is no riskier than rain: {storm:?} against {rain:?}"
        );
    }
}
