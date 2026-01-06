//! Guideline exception detection engine.

use crate::data::*;
use crate::state::*;

/// Result of guideline evaluation
#[derive(Debug, Clone)]
pub struct GuidelineEvaluationResult {
    pub is_safe: bool,
    pub consequences: Vec<Consequence>,
    pub message: String,
}

/// Guideline engine for exception detection and evaluation
pub struct GuidelineEngine;

impl GuidelineEngine {
    /// Analyze passenger for active tells based on guidelines
    pub fn analyze_passenger(
        passenger: &Passenger,
        weather: &WeatherCondition,
        player_trust: f32,
        guidelines: &[Guideline],
        current_time: f64,
    ) -> Vec<DetectedTell> {
        let mut detected = Vec::new();

        for guideline in guidelines {
            for exception in &guideline.exceptions {
                // Check if passenger matches exception
                if !Self::passenger_matches_exception(passenger, exception) {
                    continue;
                }

                // Check if conditions are met
                if !Self::check_exception_conditions(exception, weather, passenger) {
                    continue;
                }

                // Add detected tells
                for tell in &exception.tells {
                    let player_noticed = Self::calculate_detection_probability(tell, player_trust);
                    detected.push(DetectedTell {
                        tell: tell.clone(),
                        passenger_id: passenger.id,
                        detection_time: current_time,
                        player_noticed,
                        related_guideline: Some(guideline.id),
                        exception_id: Some(exception.id.clone()),
                    });
                }
            }
        }

        detected
    }

    /// Update detection cycle for game loop
    pub fn update_detection(state: &mut GameState, current_time: f64) {
        if matches!(state.game_phase, GamePhase::Driving | GamePhase::Interaction) {
            let passenger_opt = state.current_passenger.clone();
            if let Some(passenger) = passenger_opt {
                let weather = state.current_weather.clone();
                let player_trust = state.player_trust;
                let guidelines = state.current_guidelines.clone();

                let mut new_tells = Self::analyze_passenger(
                    &passenger,
                    &weather,
                    player_trust,
                    &guidelines,
                    current_time
                );

                // Introduce false tells for experienced players
                if Self::should_introduce_false_tells(state) {
                     if let Some(real_tell) = new_tells.first().cloned() {
                        let false_tell = DetectedTell {
                            tell: real_tell.tell.clone(),
                            passenger_id: real_tell.passenger_id,
                            detection_time: current_time,
                            player_noticed: false,
                            related_guideline: real_tell.related_guideline,
                            exception_id: real_tell.exception_id,
                        };
                        new_tells.push(false_tell);
                    }
                }
                
                // Merge
                for tell in new_tells {
                    if !state.detected_tells.iter().any(|t|
                        t.tell.description == tell.tell.description && t.passenger_id == tell.passenger_id
                    ) {
                        state.detected_tells.push(tell);
                    }
                }
            }
        }
    }

    /// Check if passenger matches an exception
    fn passenger_matches_exception(passenger: &Passenger, exception: &GuidelineException) -> bool {
        // Check by ID
        if !exception.passenger_ids.is_empty() && exception.passenger_ids.contains(&passenger.id) {
            return true;
        }

        // Check by type
        if !exception.passenger_types.is_empty() 
            && exception.passenger_types.contains(&passenger.supernatural) 
        {
            return true;
        }

        // Check by exception ID in passenger data
        passenger.guideline_exceptions.contains(&exception.id)
    }

    /// Check if exception conditions are met
    fn check_exception_conditions(
        exception: &GuidelineException,
        weather: &WeatherCondition,
        passenger: &Passenger,
    ) -> bool {
        for condition in &exception.conditions {
            let met = match condition.condition_type.as_str() {
                "passenger_dialogue" => {
                    let value = condition.value.as_str().unwrap_or("");
                    passenger.dialogue.iter().any(|line| {
                        line.to_lowercase().contains(&value.to_lowercase())
                    })
                }
                "passenger_behavior" => {
                    if let Some(value) = condition.value.as_f64() {
                        Self::compare_values(
                            passenger.stress_level as f64,
                            value,
                            condition.operator.as_deref(),
                        )
                    } else {
                        true
                    }
                }
                "environmental" => {
                    let value = condition.value.as_str().unwrap_or("");
                    format!("{:?}", weather.weather_type).to_lowercase() == value.to_lowercase()
                }
                _ => true,
            };

            if !met {
                return false;
            }
        }

        true
    }

    /// Compare values with optional operator
    fn compare_values(actual: f64, expected: f64, operator: Option<&str>) -> bool {
        match operator {
            Some("greater_than") => actual > expected,
            Some("less_than") => actual < expected,
            Some("equals") | None => (actual - expected).abs() < f64::EPSILON,
            _ => true,
        }
    }

    /// Calculate if player notices a tell
    fn calculate_detection_probability(tell: &PassengerTell, player_trust: f32) -> bool {
        let base_prob = tell.reliability;

        let intensity_mult = match tell.intensity {
            TellIntensity::Subtle => 0.3,
            TellIntensity::Moderate => 0.7,
            TellIntensity::Obvious => 1.0,
        };

        let final_prob = base_prob * intensity_mult * (0.5 + player_trust * 0.5);
        rand::random::<f32>() < final_prob
    }

    /// Evaluate a guideline choice
    pub fn evaluate_guideline_choice(
        guideline: &Guideline,
        action: GuidelineAction,
        passenger: &Passenger,
        state: &GameState,
    ) -> GuidelineEvaluationResult {
        // Find active exception
        let active_exception = Self::find_active_exception(guideline, passenger, &state.current_weather);

        match (active_exception, action) {
            (Some(exc), GuidelineAction::Break) if exc.breaking_safer => {
                // Breaking was correct
                GuidelineEvaluationResult {
                    is_safe: true,
                    consequences: Self::calculate_positive_consequences(guideline, &exc),
                    message: format!(
                        "Breaking \"{}\" was the right choice - {}",
                        guideline.title, exc.description
                    ),
                }
            }
            (Some(exc), GuidelineAction::Follow) if !exc.breaking_safer => {
                // Following was correct
                GuidelineEvaluationResult {
                    is_safe: true,
                    consequences: guideline.follow_consequences.clone(),
                    message: format!("Following \"{}\" was the right choice", guideline.title),
                }
            }
            (Some(_exc), _) => {
                // Wrong choice
                GuidelineEvaluationResult {
                    is_safe: false,
                    consequences: Self::calculate_negative_consequences(guideline),
                    message: format!(
                        "Wrong choice regarding \"{}\" - misread the passenger",
                        guideline.title
                    ),
                }
            }
            (None, GuidelineAction::Follow) => {
                // No exception, following is safe
                GuidelineEvaluationResult {
                    is_safe: true,
                    consequences: guideline.follow_consequences.clone(),
                    message: format!("Following \"{}\" was the safe choice", guideline.title),
                }
            }
            (None, GuidelineAction::Break) => {
                // No exception, breaking is dangerous
                GuidelineEvaluationResult {
                    is_safe: false,
                    consequences: guideline.break_consequences.clone(),
                    message: format!("Breaking \"{}\" was dangerous", guideline.title),
                }
            }
        }
    }

    /// Find active exception for passenger
    fn find_active_exception(
        guideline: &Guideline,
        passenger: &Passenger,
        weather: &WeatherCondition,
    ) -> Option<GuidelineException> {
        for exception in &guideline.exceptions {
            if Self::passenger_matches_exception(passenger, exception)
                && Self::check_exception_conditions(exception, weather, passenger)
            {
                return Some(exception.clone());
            }
        }
        None
    }

    /// Calculate positive consequences for correct exception reading
    fn calculate_positive_consequences(
        guideline: &Guideline,
        exception: &GuidelineException,
    ) -> Vec<Consequence> {
        vec![
            Consequence {
                consequence_type: ConsequenceType::Survival,
                value: 1,
                description: format!(
                    "Breaking \"{}\" was the right choice - {}",
                    guideline.title, exception.description
                ),
                probability: exception.probability,
            },
            Consequence {
                consequence_type: ConsequenceType::Reputation,
                value: 10,
                description: "Gained passenger trust through correct reading".to_string(),
                probability: 0.8,
            },
        ]
    }

    /// Calculate negative consequences for wrong choice
    fn calculate_negative_consequences(guideline: &Guideline) -> Vec<Consequence> {
        vec![
            Consequence {
                consequence_type: ConsequenceType::Death,
                value: 1,
                description: format!(
                    "Wrong choice regarding \"{}\" - misread the passenger",
                    guideline.title
                ),
                probability: 0.7,
            },
            Consequence {
                consequence_type: ConsequenceType::Reputation,
                value: -20,
                description: "Lost passenger trust through incorrect reading".to_string(),
                probability: 0.9,
            },
        ]
    }

    /// Check if false tells should be introduced
    pub fn should_introduce_false_tells(state: &GameState) -> bool {
        let total_rides = state.rides_completed;
        let correct = state.decision_history.iter().filter(|d| d.was_correct).count();
        let skill_level = if total_rides > 0 {
            correct as f32 / total_rides as f32
        } else {
            0.0
        };

        if total_rides > 20 && skill_level > 0.7 {
            return rand::random::<f32>() < 0.3;
        }

        if total_rides > 35 && skill_level > 0.6 {
            return rand::random::<f32>() < 0.5;
        }

        false
    }
}
