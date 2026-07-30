//! Guideline exception detection engine.

use crate::data::*;
use crate::state::*;

/// Result of guideline evaluation
#[derive(Debug, Clone)]
pub struct GuidelineEvaluationResult {
    pub is_safe: bool,
    pub consequences: Vec<Consequence>,
    pub message: String,
    /// The exception the player read correctly, if any. Naming it lets the
    /// caller relieve the passenger's need when it is the one their
    /// `stateProfile.exceptionId` points at.
    pub satisfied_exception: Option<String>,
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
                    let player_noticed =
                        Self::calculate_detection_probability(tell, passenger, player_trust);
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
        if matches!(
            state.game_phase,
            GamePhase::Driving | GamePhase::Interaction
        ) {
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
                    current_time,
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
                    if !state.detected_tells.iter().any(|t| {
                        t.tell.description == tell.tell.description
                            && t.passenger_id == tell.passenger_id
                    }) {
                        state.detected_tells.push(tell);
                    }
                }
            }
        }
    }

    /// Check if passenger matches an exception
    pub(crate) fn passenger_matches_exception(
        passenger: &Passenger,
        exception: &GuidelineException,
    ) -> bool {
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
    pub(crate) fn check_exception_conditions(
        exception: &GuidelineException,
        weather: &WeatherCondition,
        passenger: &Passenger,
    ) -> bool {
        for condition in &exception.conditions {
            let met = match condition.condition_type.as_str() {
                "passenger_dialogue" => {
                    let value = condition.value.as_str().unwrap_or("");
                    passenger
                        .dialogue
                        .iter()
                        .any(|line| line.to_lowercase().contains(&value.to_lowercase()))
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

    /// Calculate if the player notices a tell.
    ///
    /// Two authored passenger fields feed this and were read by nothing.
    /// `deceptionLevel` is how well a passenger covers what they are, and
    /// scales the tell down; `trustRequired` is how much they need to trust
    /// the driver before they let anything slip at all, and below it their
    /// tells are much harder to catch. Together they mean the Midnight Mayor
    /// and Death's Taxi Driver — 0.6 and 0.7 deception — are genuinely hard
    /// to read, while Tommy Sullivan hides nothing, and that reading anyone
    /// gets easier as the night earns their trust.
    /// Whether the driver catches this tell.
    ///
    /// Public because the state machine needs it too: a tell raised by a
    /// passenger escalating deserves the same roll as one raised by a condition
    /// being met, and for a long time it was exempt and recorded as never
    /// noticed.
    pub fn notices_tell(tell: &PassengerTell, passenger: &Passenger, player_trust: f32) -> bool {
        Self::calculate_detection_probability(tell, passenger, player_trust)
    }

    fn calculate_detection_probability(
        tell: &PassengerTell,
        passenger: &Passenger,
        player_trust: f32,
    ) -> bool {
        let base_prob = tell.reliability;

        let intensity_mult = match tell.intensity {
            TellIntensity::Subtle => 0.3,
            TellIntensity::Moderate => 0.7,
            TellIntensity::Obvious => 1.0,
        };

        let candour = (1.0 - passenger.deception_level).clamp(0.0, 1.0);
        let guarded = if player_trust < passenger.trust_required {
            0.5
        } else {
            1.0
        };

        let final_prob =
            base_prob * intensity_mult * candour * guarded * (0.5 + player_trust * 0.5);
        macroquad_toolkit::rng::rand() < final_prob
    }

    /// Evaluate a guideline choice
    pub fn evaluate_guideline_choice(
        guideline: &Guideline,
        action: GuidelineAction,
        passenger: &Passenger,
        state: &GameState,
    ) -> GuidelineEvaluationResult {
        // Find active exception
        let active_exception =
            Self::find_active_exception(guideline, passenger, &state.current_weather);

        match (active_exception, action) {
            (Some(exc), GuidelineAction::Break) if exc.breaking_safer => {
                // Breaking was correct
                GuidelineEvaluationResult {
                    is_safe: true,
                    consequences: guideline.exception_rewards.clone(),
                    message: format!(
                        "Breaking \"{}\" was the right choice - {}",
                        guideline.title, exc.description
                    ),
                    satisfied_exception: Some(exc.id.clone()),
                }
            }
            (Some(exc), GuidelineAction::Follow) if !exc.breaking_safer => {
                // Following was correct
                GuidelineEvaluationResult {
                    is_safe: true,
                    consequences: guideline.follow_consequences.clone(),
                    message: format!("Following \"{}\" was the right choice", guideline.title),
                    satisfied_exception: Some(exc.id.clone()),
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
                    satisfied_exception: None,
                }
            }
            (None, GuidelineAction::Follow) => {
                // No exception, following is safe
                GuidelineEvaluationResult {
                    is_safe: true,
                    consequences: guideline.follow_consequences.clone(),
                    message: format!("Following \"{}\" was the safe choice", guideline.title),
                    satisfied_exception: None,
                }
            }
            (None, GuidelineAction::Break) => {
                // No exception, breaking is dangerous
                GuidelineEvaluationResult {
                    is_safe: false,
                    consequences: guideline.break_consequences.clone(),
                    message: format!("Breaking \"{}\" was dangerous", guideline.title),
                    satisfied_exception: None,
                }
            }
        }
    }

    /// Find active exception for passenger
    /// The exception on `guideline` that currently applies to `passenger`, if
    /// any — the same check `evaluate_guideline_choice` judges the player's
    /// decision by.
    ///
    /// This was private, so nothing outside could ask the question the game
    /// was about to answer. The playtest bot guessed from whether a detected
    /// tell mentioned a `breakingSafer` exception, ignoring the conditions
    /// that decide whether it is live, and broke guidelines that were not
    /// excepted — eight of twenty shifts with a full skill tree ended on
    /// "Breaking X was dangerous". A decider and a judge disagreeing about
    /// the same question is the same fault as the route quote.
    pub fn find_active_exception(
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
        let correct = state
            .decision_history
            .iter()
            .filter(|d| d.was_correct)
            .count();
        let skill_level = if total_rides > 0 {
            correct as f32 / total_rides as f32
        } else {
            0.0
        };

        if total_rides > 20 && skill_level > 0.7 {
            return macroquad_toolkit::rng::rand() < 0.3;
        }

        if total_rides > 35 && skill_level > 0.6 {
            return macroquad_toolkit::rng::rand() < 0.5;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::data::environment::WeatherCondition;
    use crate::data::loader::{load_guidelines, load_passengers};
    use crate::data::ConsequenceType;
    use std::collections::HashSet;

    /// Reading a passenger correctly pays what the guideline authors.
    ///
    /// This used to be a hardcoded pair built in `calculate_positive_
    /// consequences`, so all eighteen guidelines authored `exceptionRewards`
    /// -- reputation and a chance at the passenger's story -- and none of it
    /// ever paid. An empty list here would silently restore that.
    #[test]
    fn every_guideline_pays_something_for_a_correct_read() {
        for guideline in load_guidelines() {
            assert!(
                !guideline.exception_rewards.is_empty(),
                "guideline {} pays nothing for reading the passenger right",
                guideline.id
            );
            for reward in &guideline.exception_rewards {
                assert!(
                    reward.probability > 0.0,
                    "guideline {} authors a reward at zero probability",
                    guideline.id
                );
            }
        }
    }

    /// And at least one of them has to be the story unlock, since that is the
    /// only route from reading a passenger well to the almanac knowing more
    /// about them.
    #[test]
    fn a_correct_read_can_reveal_a_passengers_story() {
        let reveals = load_guidelines().iter().any(|guideline| {
            guideline
                .exception_rewards
                .iter()
                .any(|reward| reward.consequence_type == ConsequenceType::StoryUnlock)
        });
        assert!(
            reveals,
            "no guideline can reveal a story, so tell-reading feeds the almanac nothing"
        );
    }

    /// A passenger's `stateProfile.exceptionId` is what lets satisfying a
    /// guideline exception relieve their need. If it names an exception that
    /// does not exist, or one that does not target them, the relief path is
    /// unreachable and `exceptionRelief` is dead balance.
    #[test]
    fn every_profile_exception_targets_its_own_passenger() {
        let guidelines = load_guidelines();
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let Some(exception_id) = &profile.exception_id else {
                continue;
            };
            let matched = guidelines
                .iter()
                .flat_map(|g| g.exceptions.iter())
                .find(|e| &e.id == exception_id)
                .unwrap_or_else(|| {
                    panic!(
                        "{} names unknown exception {exception_id:?}",
                        passenger.name
                    )
                });
            assert!(
                matched.passenger_ids.contains(&passenger.id)
                    || matched.passenger_types.contains(&passenger.supernatural),
                "exception {exception_id:?} does not target {}",
                passenger.name
            );
        }
    }

    /// Deception must be a real gradient. If every passenger hides the same
    /// amount the field may as well not exist, and the almanac's Candour line
    /// tells the player nothing worth paying for.
    #[test]
    fn deception_varies_across_the_roster() {
        let levels: Vec<f32> = load_passengers()
            .iter()
            .map(|p| p.deception_level)
            .collect();
        let lowest = levels.iter().cloned().fold(f32::INFINITY, f32::min);
        let highest = levels.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            highest - lowest > 0.4,
            "deception spans only {lowest}..{highest}"
        );
    }

    /// Nobody may be authored past total deception or total secrecy, which
    /// would make their tells undetectable however well the player plays.
    #[test]
    fn nobody_is_completely_unreadable() {
        for passenger in load_passengers() {
            assert!(
                (0.0..1.0).contains(&passenger.deception_level),
                "{} has deception {}",
                passenger.name,
                passenger.deception_level
            );
            assert!(
                (0.0..=1.0).contains(&passenger.trust_required),
                "{} requires trust {}",
                passenger.name,
                passenger.trust_required
            );
        }
    }

    /// The harder a passenger is to read, the more trust they should want
    /// first — otherwise the two fields pull against each other and the
    /// difficulty they describe is incoherent.
    #[test]
    fn deception_and_trust_required_agree() {
        let mut passengers = load_passengers();
        passengers.sort_by(|a, b| a.deception_level.total_cmp(&b.deception_level));
        let least = &passengers[0];
        let most = passengers.last().expect("roster is not empty");
        assert!(
            most.trust_required >= least.trust_required,
            "{} hides most but asks less trust than {}",
            most.name,
            least.name
        );
    }

    /// Relief is the only downward pressure on a passenger's need, so a
    /// profile that authors none can never be settled by reading it right.
    #[test]
    fn every_profile_authors_relief() {
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            assert!(
                profile.need_change.exception_relief > 0,
                "{} has no exceptionRelief",
                passenger.name
            );
        }
    }

    /// Reading a passenger correctly must win back more than a single leg of
    /// the ride costs, or the relief is cosmetic and the need still ratchets
    /// to meltdown no matter how well the player plays.
    #[test]
    fn relief_outpaces_a_leg_of_need_growth() {
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let change = &profile.need_change;
            let worst_leg = change.passive + change.obey.max(change.break_rule);
            assert!(
                change.exception_relief > worst_leg,
                "{}: relief {} does not beat one leg's {worst_leg}",
                passenger.name,
                change.exception_relief
            );
        }
    }

    /// Conversely, an exception that names passenger ids must name real ones.
    #[test]
    fn every_exception_targets_real_passengers() {
        let ids: HashSet<u32> = load_passengers().iter().map(|p| p.id).collect();
        for guideline in load_guidelines() {
            for exception in &guideline.exceptions {
                for id in &exception.passenger_ids {
                    assert!(
                        ids.contains(id),
                        "exception {:?} targets unknown passenger {id}",
                        exception.id
                    );
                }
            }
        }
    }

    /// An exception gated on words nobody says is an exception that never
    /// happens.
    ///
    /// `passenger_dialogue` conditions substring-match the value against the
    /// passenger's own `dialogue` lines, so the guideline file and the
    /// passenger file have to agree about a word -- and they are different
    /// files with nothing between them. Twelve of the fourteen agreed. Two did
    /// not: Sister Agnes's blessing and Death's offer to trade places were
    /// authored in full, each with two tells, a `breakingSafer` flag and a
    /// `requiredStage`, and neither passenger had a line containing "bless" or
    /// "trade". Both exceptions were unreachable, which also put their rewards,
    /// their relief and their tells out of the player's reach.
    ///
    /// Eligibility is checked the same way the engine checks it, so this fails
    /// if the word goes missing or if the exception is pointed at a passenger
    /// who never says it.
    #[test]
    fn every_dialogue_exception_has_someone_who_says_the_words() {
        let passengers = load_passengers();
        let mut checked = 0;
        for guideline in load_guidelines() {
            for exception in &guideline.exceptions {
                for condition in &exception.conditions {
                    if condition.condition_type != "passenger_dialogue" {
                        continue;
                    }
                    let word = condition
                        .value
                        .as_str()
                        .expect("a dialogue condition matches a string")
                        .to_lowercase();
                    checked += 1;
                    let speaker = passengers.iter().find(|passenger| {
                        super::GuidelineEngine::passenger_matches_exception(passenger, exception)
                            && passenger
                                .dialogue
                                .iter()
                                .any(|line| line.to_lowercase().contains(&word))
                    });
                    assert!(
                        speaker.is_some(),
                        "exception {:?} on guideline {} waits to hear {:?}, and no \
                         passenger it applies to has a line containing it",
                        exception.id,
                        guideline.id,
                        word
                    );
                }
            }
        }
        assert!(checked > 0, "no dialogue conditions found to check");
    }

    /// A passenger's own relief has to be something they can actually earn.
    ///
    /// Sixteen passengers each name an exception in `stateProfile.exceptionId`
    /// and author an `exceptionRelief` -- 24 to 35 points off the need that is
    /// about to break them -- and reading them right is the only way to spend
    /// it. That makes the exception's conditions a promise to that specific
    /// passenger, which is a stronger claim than the conditions merely being
    /// satisfiable by somebody.
    ///
    /// Jake Morrison could not keep it. He is named on `shortcut_time_critical`
    /// by id, points his own state profile at it, and every staged line he has
    /// begs for a faster route -- "the thirst is unbearable, take the alleys".
    /// The exception asks for stress above zero. `stressLevel` is
    /// `serde(default)`, nine of the sixteen omit it, and 0.0 is not a calm
    /// passenger but an unauthored one. So the vampire racing dawn could never
    /// have his shortcut read as the right call, could never collect the 35, and
    /// breaking the rule for him was never the safer play it is written to be.
    ///
    /// This checks both condition kinds, so it also covers the two dialogue
    /// exceptions fixed alongside it.
    #[test]
    fn every_passenger_can_reach_the_relief_they_are_promised() {
        let guidelines = load_guidelines();
        let mut checked = 0;
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let Some(wanted) = &profile.exception_id else {
                continue;
            };
            let found = guidelines
                .iter()
                .flat_map(|guideline| &guideline.exceptions)
                .find(|exception| &exception.id == wanted);
            let exception = found.unwrap_or_else(|| {
                panic!(
                    "{} relies on exception {wanted:?}, which no guideline authors",
                    passenger.name
                )
            });
            checked += 1;
            assert!(
                super::GuidelineEngine::passenger_matches_exception(&passenger, exception),
                "{} points at exception {wanted:?} and is not eligible for it",
                passenger.name
            );
            assert!(
                super::GuidelineEngine::check_exception_conditions(
                    exception,
                    &WeatherCondition::default(),
                    &passenger,
                ),
                "{} can never satisfy exception {wanted:?}, so the relief their \
                 state profile authors is unreachable",
                passenger.name
            );
        }
        assert!(checked > 0, "no passenger relief to check");
    }
}
