//! Passenger selection service.

use crate::data::*;
use crate::state::PlayerStats;
use rand::prelude::*;
use std::collections::HashMap;

/// Passenger selection service
pub struct PassengerService;

/// Context for passenger selection
pub struct PassengerSelectionContext<'a> {
    pub difficulty_level: u32,
    pub weather: &'a WeatherCondition,
    pub time_of_day: &'a TimeOfDay,
    pub season: &'a Season,
    pub constants: &'a ConstantsData,
}

impl PassengerService {
    /// Get rarity weights adjusted for difficulty
    fn get_adjusted_weights(difficulty_level: u32, constants: &ConstantsData) -> HashMap<Rarity, u32> {
        let mut weights = HashMap::new();
        weights.insert(Rarity::Common, constants.rarity_weights.common);
        
        let rare_mult = match difficulty_level {
            0..=1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => 3.0,
        };
        let legendary_mult = match difficulty_level {
            0..=1 => 1.0,
            2 => 2.0,
            3 => 3.0,
            _ => 5.0,
        };

        weights.insert(Rarity::Uncommon, constants.rarity_weights.uncommon);
        weights.insert(Rarity::Rare, ((constants.rarity_weights.rare as f32) * rare_mult) as u32);
        weights.insert(Rarity::Legendary, ((constants.rarity_weights.legendary as f32) * legendary_mult) as u32);

        weights
    }

    /// Select passenger with weather and time awareness
    pub fn select_weather_aware_passenger(
        passengers: &[Passenger],
        used_passengers: &[u32],
        context: &PassengerSelectionContext,
    ) -> Option<Passenger> {
        let available: Vec<&Passenger> = passengers.iter()
            .filter(|p| !used_passengers.contains(&p.id))
            .collect();

        if available.is_empty() {
            // Reset pool if all used
            return Self::select_weather_aware_from_pool(
                &passengers.iter().collect::<Vec<_>>(),
                context,
            );
        }

        Self::select_weather_aware_from_pool(&available, context)
    }

    /// Internal selection with all environmental factors
    fn select_weather_aware_from_pool(
        passengers: &[&Passenger],
        context: &PassengerSelectionContext,
    ) -> Option<Passenger> {
        let mut rng = rand::thread_rng();

        // Calculate weights with environmental modifiers
        let weighted: Vec<(Passenger, f32)> = passengers.iter().map(|p| {
            let base_weight = Self::get_base_rarity_weight(p.rarity, context.difficulty_level, context.constants);
            let weather_mod = Self::get_weather_modifier(p.id, context.weather);
            let time_mod = Self::get_time_modifier(p.id, context.time_of_day);
            let season_mod = Self::get_seasonal_modifier(p.id, context.season);
            let special_mod = Self::get_special_behavior_modifier(p.id, context.weather, context.time_of_day);

            let final_weight = base_weight * weather_mod * time_mod * season_mod * special_mod;
            ((*p).clone(), final_weight.max(0.1))
        }).collect();
        
        // ... rest of function ...


        // Weighted random selection
        let total_weight: f32 = weighted.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return passengers.first().map(|p| (*p).clone());
        }

        let mut random = rng.gen::<f32>() * total_weight;
        for (passenger, weight) in weighted {
            random -= weight;
            if random <= 0.0 {
                return Some(passenger);
            }
        }

        passengers.last().map(|p| (*p).clone())
    }

    /// Get base weight from rarity
    fn get_base_rarity_weight(rarity: Rarity, difficulty_level: u32, constants: &ConstantsData) -> f32 {
        let weights = Self::get_adjusted_weights(difficulty_level, constants);
        weights.get(&rarity).copied().unwrap_or(1) as f32
    }

    /// Weather preference modifiers per passenger
    fn get_weather_modifier(passenger_id: u32, weather: &WeatherCondition) -> f32 {
        let base = match (passenger_id, weather.weather_type) {
            // Mrs. Chen - ghosts like misty conditions
            (1, WeatherType::Fog) => 1.5,
            (1, WeatherType::Rain) => 1.3,
            // Sarah Woods - escaped during storms
            (3, WeatherType::Thunderstorm) => 1.4,
            (3, WeatherType::Rain) => 1.2,
            // Dr. Hollow - mysterious conditions
            (4, WeatherType::Fog) => 1.3,
            (4, WeatherType::Thunderstorm) => 1.2,
            // The Collector - dramatic weather
            (5, WeatherType::Clear) => 0.8,
            (5, WeatherType::Thunderstorm) => 1.5,
            // Elena Vasquez - prefers clear nights
            (7, WeatherType::Clear) => 1.2,
            (7, WeatherType::Rain) => 0.9,
            // Old Pete - water-related weather
            (10, WeatherType::Rain) => 1.4,
            (10, WeatherType::Fog) => 1.3,
            // Madame Zelda - supernatural weather
            (11, WeatherType::Thunderstorm) => 1.6,
            (11, WeatherType::Fog) => 1.4,
            // Frank the Pianist - melancholy weather
            (12, WeatherType::Rain) => 1.2,
            (12, WeatherType::Clear) => 1.1,
            // Sister Agnes - active in dangerous weather
            (13, WeatherType::Thunderstorm) => 1.3,
            (13, WeatherType::Snow) => 1.2,
            // The Midnight Mayor - dramatic weather
            (15, WeatherType::Thunderstorm) => 1.8,
            (15, WeatherType::Fog) => 1.5,
            // Death's Taxi Driver - ominous conditions
            (16, WeatherType::Thunderstorm) => 2.0,
            (16, WeatherType::Fog) => 1.7,
            _ => 1.0,
        };

        // Heavy weather intensifies supernatural activity
        if weather.intensity == WeatherIntensity::Heavy {
            // Supernatural passengers more likely in heavy weather
            match passenger_id {
                1 | 4 | 11 | 15 | 16 => base * 1.3,
                _ => base,
            }
        } else {
            base
        }
    }

    /// Time of day preference modifiers
    fn get_time_modifier(passenger_id: u32, time_of_day: &TimeOfDay) -> f32 {
        let base = match (passenger_id, time_of_day.phase) {
            // Mrs. Chen - ghost most active at night
            (1, TimePhase::Night) => 1.4,
            (1, TimePhase::Latenight) => 1.6,
            // Jake Morrison - night shift worker
            (2, TimePhase::Night) => 1.3,
            (2, TimePhase::Latenight) => 1.2,
            // Dr. Hollow - mysterious nighttime visits
            (4, TimePhase::Night) => 1.5,
            (4, TimePhase::Latenight) => 1.3,
            // The Collector - evening deals
            (5, TimePhase::Dusk) => 1.2,
            (5, TimePhase::Night) => 1.4,
            // Tommy Sullivan - child, less likely very late
            (6, TimePhase::Latenight) => 0.7,
            // Elena Vasquez - evening performance times
            (7, TimePhase::Dusk) => 1.3,
            (7, TimePhase::Night) => 1.2,
            // Madame Zelda - fortune telling at night
            (11, TimePhase::Night) => 1.5,
            (11, TimePhase::Latenight) => 1.6,
            // The Midnight Mayor - rules the night
            (15, TimePhase::Night) => 1.8,
            (15, TimePhase::Latenight) => 2.0,
            // Death's Taxi Driver - witching hour
            (16, TimePhase::Latenight) => 2.5,
            _ => 1.0,
        };

        // Supernatural activity modifier for non-human passengers
        let supernatural_mod = time_of_day.supernatural_activity as f32 / 100.0;
        match passenger_id {
            1 | 4 | 5 | 11 | 15 | 16 => base * (0.7 + supernatural_mod * 0.6),
            _ => base,
        }
    }

    /// Seasonal preference modifiers
    fn get_seasonal_modifier(passenger_id: u32, season: &Season) -> f32 {
        match (passenger_id, season.season_type) {
            // Nature spirits more active in spring
            (3, SeasonType::Spring) => 1.5,
            (8, SeasonType::Spring) => 1.3,
            (10, SeasonType::Spring) => 1.2,
            // Melancholy souls more active in fall
            (1, SeasonType::Fall) => 1.3,
            (12, SeasonType::Fall) => 1.4,
            // Frost entities more active in winter
            (4, SeasonType::Winter) => 1.5,
            (15, SeasonType::Winter) => 1.4,
            (16, SeasonType::Winter) => 1.5,
            // Holiday spirits
            (6, SeasonType::Winter) => 1.2,
            (13, SeasonType::Winter) => 1.3,
            _ => 1.0,
        }
    }

    /// Special behavior modifiers for combined conditions
    fn get_special_behavior_modifier(
        passenger_id: u32,
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
    ) -> f32 {
        let mut modifier = 1.0;

        // Thunderstorm + latenight = perfect supernatural conditions
        if weather.weather_type == WeatherType::Thunderstorm && time_of_day.phase == TimePhase::Latenight {
            match passenger_id {
                15 | 16 | 11 => modifier *= 2.0,
                _ => {}
            }
        }

        // Fog + night = mysterious conditions
        if weather.weather_type == WeatherType::Fog && time_of_day.is_night() {
            match passenger_id {
                1 | 4 => modifier *= 1.5,
                _ => {}
            }
        }

        modifier
    }

    /// Check if backstory should unlock
    pub fn check_backstory_unlock(
        passenger_id: u32,
        player_stats: &PlayerStats,
        constants: &ConstantsData,
    ) -> bool {
        // Already unlocked
        if player_stats.is_backstory_unlocked(passenger_id) {
            return true;
        }

        let is_first = player_stats.is_first_encounter(passenger_id);
        let chance = if is_first {
            constants.game_constants.backstory_unlock_first
        } else {
            constants.game_constants.backstory_unlock_repeat
        };

        rand::thread_rng().gen::<f32>() < chance
    }
}
