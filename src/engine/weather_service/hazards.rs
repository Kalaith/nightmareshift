//! Environmental hazards spawned by the current weather, time, and season.

use crate::data::*;

use super::WeatherService;

impl WeatherService {
    /// Generate environmental hazards
    pub fn generate_hazards(
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
        season: &Season,
        current_time: f64,
    ) -> Vec<EnvironmentalHazard> {
        let mut hazards = Vec::new();

        let hazard_chance = Self::calculate_hazard_chance(weather, time_of_day, season);

        if macroquad_toolkit::rng::chance(hazard_chance) {
            let hazard_type = Self::select_hazard_type(weather, time_of_day);
            hazards.push(Self::create_hazard(hazard_type, weather, current_time));
        }

        // Weather-specific hazards
        if weather.intensity == WeatherIntensity::Heavy {
            if weather.weather_type == WeatherType::Rain && macroquad_toolkit::rng::chance(0.4) {
                hazards.push(Self::create_weather_hazard(
                    "flooding",
                    weather,
                    current_time,
                ));
            } else if weather.weather_type == WeatherType::Snow
                && macroquad_toolkit::rng::chance(0.5)
            {
                hazards.push(Self::create_weather_hazard(
                    "ice_roads",
                    weather,
                    current_time,
                ));
            } else if weather.weather_type == WeatherType::Fog
                && macroquad_toolkit::rng::chance(0.3)
            {
                hazards.push(Self::create_weather_hazard(
                    "visibility",
                    weather,
                    current_time,
                ));
            }
        }

        hazards
    }

    fn calculate_hazard_chance(
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
        season: &Season,
    ) -> f32 {
        let mut chance: f32 = 0.15;

        if weather.intensity == WeatherIntensity::Moderate {
            chance += 0.1;
        }
        if weather.intensity == WeatherIntensity::Heavy {
            chance += 0.2;
        }

        if matches!(time_of_day.phase, TimePhase::Night | TimePhase::Latenight) {
            chance += 0.15;
        }

        if season.season_type == SeasonType::Winter {
            chance += 0.1;
        }

        chance.min(0.6)
    }

    fn select_hazard_type(weather: &WeatherCondition, time_of_day: &TimeOfDay) -> HazardType {
        let mut types = vec![
            HazardType::Construction,
            HazardType::Accident,
            HazardType::RoadClosure,
        ];

        if weather.weather_type == WeatherType::Thunderstorm
            || time_of_day.supernatural_activity > 70
        {
            types.push(HazardType::SupernaturalEvent);
        }

        if matches!(time_of_day.phase, TimePhase::Night | TimePhase::Latenight) {
            types.push(HazardType::PoliceCheckpoint);
        }

        *macroquad_toolkit::rng::choose(&types).unwrap_or(&HazardType::Construction)
    }

    fn create_hazard(
        hazard_type: HazardType,
        weather: &WeatherCondition,
        current_time: f64,
    ) -> EnvironmentalHazard {
        let locations = [
            "Downtown Bridge",
            "Highway 101",
            "Industrial District",
            "Cemetery Road",
            "Forest Route",
            "Waterfront Drive",
            "University Avenue",
            "Hospital District",
        ];
        let location = macroquad_toolkit::rng::choose(&locations)
            .unwrap_or(&"Unknown")
            .to_string();

        let severity = if macroquad_toolkit::rng::chance(0.1) {
            HazardSeverity::Extreme
        } else if macroquad_toolkit::rng::chance(0.3) {
            HazardSeverity::Major
        } else {
            HazardSeverity::Minor
        };

        let (effects, duration) = Self::get_hazard_effects(hazard_type, severity);

        EnvironmentalHazard {
            id: format!("{:?}_{}", hazard_type, current_time as u64),
            hazard_type,
            location: location.clone(),
            severity,
            description: Self::get_hazard_description(hazard_type, severity, &location),
            effects,
            duration,
            start_time: current_time,
            weather_triggered: weather.intensity == WeatherIntensity::Heavy,
        }
    }

    fn create_weather_hazard(
        hazard_name: &str,
        weather: &WeatherCondition,
        current_time: f64,
    ) -> EnvironmentalHazard {
        let (hazard_type, description, effects) = match hazard_name {
            "flooding" => (
                HazardType::RoadClosure,
                "Flash flooding blocks roadway",
                HazardEffects {
                    route_blocked: Some(vec![RouteType::Normal, RouteType::Shortcut]),
                    time_delay: Some(20),
                    risk_increase: Some(2),
                    ..Default::default()
                },
            ),
            "ice_roads" => (
                HazardType::Accident,
                "Ice makes roads treacherous",
                HazardEffects {
                    time_delay: Some(15),
                    risk_increase: Some(3),
                    forced_choice: Some(true),
                    ..Default::default()
                },
            ),
            "visibility" => (
                HazardType::Construction,
                "Poor visibility causes delays",
                HazardEffects {
                    time_delay: Some(10),
                    risk_increase: Some(1),
                    ..Default::default()
                },
            ),
            _ => (
                HazardType::Construction,
                "Unknown hazard",
                HazardEffects::default(),
            ),
        };

        let severity = if weather.intensity == WeatherIntensity::Heavy {
            HazardSeverity::Major
        } else {
            HazardSeverity::Minor
        };

        EnvironmentalHazard {
            id: format!("weather_{}_{}", hazard_name, current_time as u64),
            hazard_type,
            location: "Weather-affected area".to_string(),
            severity,
            description: description.to_string(),
            effects,
            duration: weather.duration / 2,
            start_time: current_time,
            weather_triggered: true,
        }
    }

    fn get_hazard_effects(
        hazard_type: HazardType,
        severity: HazardSeverity,
    ) -> (HazardEffects, u32) {
        let mult = match severity {
            HazardSeverity::Minor => 1,
            HazardSeverity::Major => 2,
            HazardSeverity::Extreme => 3,
        };

        let (effects, base_duration) = match hazard_type {
            HazardType::Construction => (
                HazardEffects {
                    time_delay: Some(5 * mult),
                    fuel_increase: Some(2 * mult),
                    ..Default::default()
                },
                45,
            ),
            HazardType::Accident => (
                HazardEffects {
                    time_delay: Some(10 * mult),
                    risk_increase: Some(mult),
                    ..Default::default()
                },
                25,
            ),
            HazardType::SupernaturalEvent => (
                HazardEffects {
                    risk_increase: Some(2 * mult),
                    forced_choice: Some(true),
                    ..Default::default()
                },
                15,
            ),
            HazardType::RoadClosure => (
                HazardEffects {
                    route_blocked: Some(vec![RouteType::Normal, RouteType::Shortcut]),
                    time_delay: Some(15 * mult),
                    ..Default::default()
                },
                60,
            ),
            HazardType::PoliceCheckpoint => (
                HazardEffects {
                    time_delay: Some(8 * mult),
                    risk_increase: Some(mult),
                    ..Default::default()
                },
                20,
            ),
        };

        let duration_mult = match severity {
            HazardSeverity::Minor => 0.7,
            HazardSeverity::Major => 1.0,
            HazardSeverity::Extreme => 1.5,
        };

        (effects, (base_duration as f32 * duration_mult) as u32)
    }

    fn get_hazard_description(
        hazard_type: HazardType,
        severity: HazardSeverity,
        location: &str,
    ) -> String {
        match (hazard_type, severity) {
            (HazardType::Construction, HazardSeverity::Minor) => {
                format!("Minor road work on {}", location)
            }
            (HazardType::Construction, HazardSeverity::Major) => {
                format!("Major construction project blocks {}", location)
            }
            (HazardType::Construction, HazardSeverity::Extreme) => {
                format!("Emergency road repairs shut down {}", location)
            }
            (HazardType::Accident, HazardSeverity::Minor) => {
                format!("Fender-bender causes delays on {}", location)
            }
            (HazardType::Accident, HazardSeverity::Major) => {
                format!("Multi-car accident blocks lanes on {}", location)
            }
            (HazardType::Accident, HazardSeverity::Extreme) => {
                format!("Major crash completely closes {}", location)
            }
            (HazardType::SupernaturalEvent, HazardSeverity::Minor) => {
                format!("Strange lights reported near {}", location)
            }
            (HazardType::SupernaturalEvent, HazardSeverity::Major) => {
                format!("Unexplained phenomena disrupt traffic at {}", location)
            }
            (HazardType::SupernaturalEvent, HazardSeverity::Extreme) => {
                format!("Supernatural event forces evacuation of {}", location)
            }
            (HazardType::RoadClosure, HazardSeverity::Minor) => {
                format!("Temporary closure of one lane on {}", location)
            }
            (HazardType::RoadClosure, HazardSeverity::Major) => {
                format!("{} closed for maintenance", location)
            }
            (HazardType::RoadClosure, HazardSeverity::Extreme) => {
                format!("{} completely shut down indefinitely", location)
            }
            (HazardType::PoliceCheckpoint, HazardSeverity::Minor) => {
                format!("Routine checkpoint on {}", location)
            }
            (HazardType::PoliceCheckpoint, HazardSeverity::Major) => {
                format!("Extensive police presence at {}", location)
            }
            (HazardType::PoliceCheckpoint, HazardSeverity::Extreme) => {
                format!("Roadblock and search operation on {}", location)
            }
        }
    }
}
