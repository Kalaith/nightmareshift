//! Passenger selection service.

use crate::data::*;
use crate::state::PlayerStats;

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
    /// Ids of passengers connected to anyone already carried this shift.
    ///
    /// `relationships` is authored on thirteen of the sixteen passengers and
    /// was read by nothing, as was the `RELATED_PASSENGER_SPAWN` probability
    /// waiting for it. Links are followed in both directions because they are
    /// authored one-way in places — Old Pete lists Mrs. Chen, but she lists
    /// no one.
    pub fn kin_of(met: &[u32], passengers: &[Passenger]) -> Vec<u32> {
        let mut kin: Vec<u32> = Vec::new();
        for passenger in passengers {
            if met.contains(&passenger.id) {
                // Everyone this passenger names.
                for id in &passenger.relationships {
                    if !met.contains(id) && !kin.contains(id) {
                        kin.push(*id);
                    }
                }
            } else if passenger.relationships.iter().any(|id| met.contains(id))
                && !kin.contains(&passenger.id)
            {
                // Anyone who names a passenger we have met.
                kin.push(passenger.id);
            }
        }
        kin
    }

    /// Get rarity weights adjusted for difficulty
    fn get_adjusted_weights(
        difficulty_level: u32,
        constants: &ConstantsData,
    ) -> HashMap<Rarity, u32> {
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
        weights.insert(
            Rarity::Rare,
            ((constants.rarity_weights.rare as f32) * rare_mult) as u32,
        );
        weights.insert(
            Rarity::Legendary,
            ((constants.rarity_weights.legendary as f32) * legendary_mult) as u32,
        );

        weights
    }

    /// Select passenger with weather and time awareness
    pub fn select_weather_aware_passenger(
        rng: &mut macroquad_toolkit::rng::SeededRng,
        passengers: &[Passenger],
        used_passengers: &[u32],
        context: &PassengerSelectionContext,
    ) -> Option<Passenger> {
        let available: Vec<&Passenger> = passengers
            .iter()
            .filter(|p| !used_passengers.contains(&p.id))
            .collect();

        if available.is_empty() {
            // Reset pool if all used
            return Self::select_weather_aware_from_pool(
                rng,
                &passengers.iter().collect::<Vec<_>>(),
                context,
            );
        }

        Self::select_weather_aware_from_pool(rng, &available, context)
    }

    /// Internal selection with all environmental factors
    fn select_weather_aware_from_pool(
        rng: &mut macroquad_toolkit::rng::SeededRng,
        passengers: &[&Passenger],
        context: &PassengerSelectionContext,
    ) -> Option<Passenger> {
        // Calculate weights with environmental modifiers
        let weighted: Vec<(Passenger, f32)> = passengers
            .iter()
            .map(|p| {
                let base_weight = Self::get_base_rarity_weight(
                    p.rarity,
                    context.difficulty_level,
                    context.constants,
                );
                let weather_mod = Self::get_weather_modifier(p, context.weather);
                let time_mod = Self::get_time_modifier(p, context.time_of_day);
                let season_mod = Self::get_seasonal_modifier(p, context.season);
                let special_mod =
                    Self::get_special_behavior_modifier(p, context.weather, context.time_of_day);

                let final_weight = base_weight * weather_mod * time_mod * season_mod * special_mod;
                ((*p).clone(), final_weight.max(0.1))
            })
            .collect();

        // ... rest of function ...

        // Weighted random selection
        let total_weight: f32 = weighted.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return passengers.first().map(|p| (*p).clone());
        }

        let mut random = rng.next_f32() * total_weight;
        for (passenger, weight) in weighted {
            random -= weight;
            if random <= 0.0 {
                return Some(passenger);
            }
        }

        passengers.last().map(|p| (*p).clone())
    }

    /// Get base weight from rarity
    fn get_base_rarity_weight(
        rarity: Rarity,
        difficulty_level: u32,
        constants: &ConstantsData,
    ) -> f32 {
        let weights = Self::get_adjusted_weights(difficulty_level, constants);
        weights.get(&rarity).copied().unwrap_or(1) as f32
    }

    /// Lowercase key for a weather type, matching the JSON `spawnWeighting` keys.
    fn weather_key(weather_type: WeatherType) -> &'static str {
        match weather_type {
            WeatherType::Clear => "clear",
            WeatherType::Rain => "rain",
            WeatherType::Fog => "fog",
            WeatherType::Snow => "snow",
            WeatherType::Thunderstorm => "thunderstorm",
            WeatherType::Wind => "wind",
        }
    }

    /// Lowercase key for a time phase, matching the JSON `spawnWeighting` keys.
    fn phase_key(phase: TimePhase) -> &'static str {
        match phase {
            TimePhase::Dawn => "dawn",
            TimePhase::Morning => "morning",
            TimePhase::Afternoon => "afternoon",
            TimePhase::Dusk => "dusk",
            TimePhase::Night => "night",
            TimePhase::Latenight => "latenight",
        }
    }

    /// Lowercase key for a season, matching the JSON `spawnWeighting` keys.
    fn season_key(season_type: SeasonType) -> &'static str {
        match season_type {
            SeasonType::Spring => "spring",
            SeasonType::Summer => "summer",
            SeasonType::Fall => "fall",
            SeasonType::Winter => "winter",
        }
    }

    /// Weather preference modifier, driven by the passenger's `spawnWeighting`.
    fn get_weather_modifier(passenger: &Passenger, weather: &WeatherCondition) -> f32 {
        let Some(sw) = &passenger.spawn_weighting else {
            return 1.0;
        };
        let base = sw
            .weather
            .get(Self::weather_key(weather.weather_type))
            .copied()
            .unwrap_or(1.0);

        if weather.intensity == WeatherIntensity::Heavy {
            base * sw.heavy_weather_boost
        } else {
            base
        }
    }

    /// Time-of-day preference modifier, driven by the passenger's `spawnWeighting`.
    fn get_time_modifier(passenger: &Passenger, time_of_day: &TimeOfDay) -> f32 {
        let Some(sw) = &passenger.spawn_weighting else {
            return 1.0;
        };
        let base = sw
            .time
            .get(Self::phase_key(time_of_day.phase))
            .copied()
            .unwrap_or(1.0);

        if sw.supernatural_time_scaling {
            let supernatural_mod = time_of_day.supernatural_activity as f32 / 100.0;
            base * (0.7 + supernatural_mod * 0.6)
        } else {
            base
        }
    }

    /// Seasonal preference modifier, driven by the passenger's `spawnWeighting`.
    fn get_seasonal_modifier(passenger: &Passenger, season: &Season) -> f32 {
        passenger
            .spawn_weighting
            .as_ref()
            .and_then(|sw| sw.season.get(Self::season_key(season.season_type)).copied())
            .unwrap_or(1.0)
    }

    /// Combined-condition modifier, driven by the passenger's `spawnWeighting`.
    fn get_special_behavior_modifier(
        passenger: &Passenger,
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
    ) -> f32 {
        let Some(sw) = &passenger.spawn_weighting else {
            return 1.0;
        };
        let mut modifier = 1.0;

        if weather.weather_type == WeatherType::Thunderstorm
            && time_of_day.phase == TimePhase::Latenight
        {
            modifier *= sw.storm_latenight_boost;
        }

        if weather.weather_type == WeatherType::Fog && time_of_day.is_night() {
            modifier *= sw.fog_night_boost;
        }

        modifier
    }

    /// Check if backstory should unlock
    pub fn check_backstory_unlock(
        rng: &mut macroquad_toolkit::rng::SeededRng,
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

        rng.next_f32() < chance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_passengers;

    /// Every id a passenger names as a relation must be a real passenger, or
    /// the kin pool silently skips it.
    #[test]
    fn every_relationship_names_a_real_passenger() {
        let passengers = load_passengers();
        let ids: Vec<u32> = passengers.iter().map(|p| p.id).collect();
        for passenger in &passengers {
            for id in &passenger.relationships {
                assert!(
                    ids.contains(id),
                    "{} names unknown relation {id}",
                    passenger.name
                );
            }
            assert!(
                !passenger.relationships.contains(&passenger.id),
                "{} is related to itself",
                passenger.name
            );
        }
    }

    /// Carrying someone must make at least one associate available, in either
    /// direction, or the mechanic never fires for that passenger.
    #[test]
    fn kin_is_found_in_both_directions() {
        let passengers = load_passengers();

        // Forward: Jake Morrison (2) names Dr. Hollow (4).
        let kin = PassengerService::kin_of(&[2], &passengers);
        assert!(kin.contains(&4), "forward link not followed: {kin:?}");

        // Backward: Mrs. Chen (1) names nobody, but Old Pete (10) names her.
        let kin = PassengerService::kin_of(&[1], &passengers);
        assert!(kin.contains(&10), "backward link not followed: {kin:?}");
    }

    /// Someone already carried this shift must never be offered again as kin.
    #[test]
    fn kin_excludes_passengers_already_carried() {
        let passengers = load_passengers();
        let met = [2, 4];
        for id in PassengerService::kin_of(&met, &passengers) {
            assert!(!met.contains(&id), "kin pool re-offered passenger {id}");
        }
    }

    /// With nobody carried yet there is no kin pool, so the first fare of a
    /// shift is always drawn normally.
    #[test]
    fn no_kin_before_the_first_fare() {
        assert!(PassengerService::kin_of(&[], &load_passengers()).is_empty());
    }

    /// The relationship web must be broad enough to matter: most of the
    /// roster should be reachable as someone's associate.
    #[test]
    fn most_of_the_roster_is_reachable_as_kin() {
        let passengers = load_passengers();
        let reachable: usize = passengers
            .iter()
            .filter(|p| !PassengerService::kin_of(&[p.id], &passengers).is_empty())
            .count();
        assert!(
            reachable * 2 > passengers.len(),
            "only {reachable} of {} passengers lead anywhere",
            passengers.len()
        );
    }
}
