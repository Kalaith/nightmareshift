//! Time of day and season: deriving the current phase and season, and the
//! rules the current conditions put in force.

use crate::data::*;

use super::WeatherService;

impl WeatherService {
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
}
