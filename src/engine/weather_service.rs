//! Weather and environment service.

use crate::data::*;

/// Weather generation and update service
pub struct WeatherService;

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

    /// Get time of day from hour
    pub fn get_time_of_day(hour: u32) -> TimeOfDay {
        let (phase, description, ambient_light, supernatural_activity) = match hour {
            6..=7 => (
                TimePhase::Dawn,
                "The sky lightens as dawn approaches",
                30,
                70,
            ),
            8..=11 => (
                TimePhase::Morning,
                "Morning light fills the streets",
                85,
                20,
            ),
            12..=16 => (TimePhase::Afternoon, "Bright afternoon sunlight", 100, 10),
            17..=19 => (
                TimePhase::Dusk,
                "The sun sets, casting long shadows",
                40,
                60,
            ),
            20..=23 => (TimePhase::Night, "Darkness settles over the city", 15, 85),
            _ => (
                TimePhase::Latenight,
                "The deepest part of the night",
                5,
                100,
            ),
        };

        TimeOfDay {
            phase,
            hour,
            description: description.to_string(),
            ambient_light,
            supernatural_activity,
        }
    }

    /// Update time of day based on shift progress
    pub fn update_time_of_day(shift_start_time: f64, current_time: f64) -> TimeOfDay {
        let elapsed_hours = (current_time - shift_start_time) / 3600.0;
        let start_hour = 18; // Shift starts at 6 PM
        let current_hour = ((start_hour as f64 + elapsed_hours) % 24.0) as u32;
        Self::get_time_of_day(current_hour)
    }

    /// Get current season from month
    pub fn get_current_season(month: u32) -> Season {
        let (season_type, temperature) = match month {
            3..=5 => (
                SeasonType::Spring,
                if month == 3 {
                    Temperature::Cool
                } else if month == 4 {
                    Temperature::Mild
                } else {
                    Temperature::Warm
                },
            ),
            6..=8 => (
                SeasonType::Summer,
                if month == 6 {
                    Temperature::Warm
                } else {
                    Temperature::Hot
                },
            ),
            9..=11 => (
                SeasonType::Fall,
                if month == 9 {
                    Temperature::Warm
                } else if month == 10 {
                    Temperature::Cool
                } else {
                    Temperature::Cold
                },
            ),
            _ => (SeasonType::Winter, Temperature::Cold),
        };

        let description = match season_type {
            SeasonType::Spring => format!(
                "Spring weather brings {:?} temperatures and frequent changes",
                temperature
            ),
            SeasonType::Summer => format!(
                "Summer heat creates {:?} conditions perfect for night driving",
                temperature
            ),
            SeasonType::Fall => format!(
                "Autumn's {:?} weather brings unpredictable conditions",
                temperature
            ),
            SeasonType::Winter => format!(
                "Winter's {:?} temperatures make every drive challenging",
                temperature
            ),
        };

        Season {
            season_type,
            month,
            temperature,
            description,
        }
    }

    /// Get weather-triggered rule IDs
    pub fn get_weather_triggered_rules(
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
    ) -> Vec<u32> {
        let mut triggered = Vec::new();

        if weather.weather_type == WeatherType::Thunderstorm {
            triggered.push(101); // Don't use windshield wipers during thunderstorms
        }

        if weather.weather_type == WeatherType::Fog && weather.intensity == WeatherIntensity::Heavy
        {
            triggered.push(102); // Keep headlights on during heavy fog
        }

        if weather.weather_type == WeatherType::Snow {
            triggered.push(103); // Drive under 25 mph in snow
        }

        if time_of_day.phase == TimePhase::Latenight && weather.weather_type != WeatherType::Clear {
            triggered.push(104); // No stops during late night bad weather
        }

        if weather.visibility < 30 {
            triggered.push(105); // Don't use AC during low visibility
        }

        if weather.weather_type == WeatherType::Wind && weather.intensity == WeatherIntensity::Heavy
        {
            triggered.push(106); // Keep windows closed during heavy wind
        }

        triggered
    }

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
                    risk_increase: Some(1 * mult),
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
                    risk_increase: Some(1 * mult),
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
