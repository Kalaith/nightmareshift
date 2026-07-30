//! What a dangerous road actually does to you.
//!
//! `RouteService::calculate_route_costs` builds a risk figure out of the
//! route type, the pickup's `riskLevel`, the weather, active hazards, route
//! mastery and any curse pressure — and until now that figure was written
//! into `route_history` and drawn on the map, and nothing else. Risk was a
//! readout, not a hazard.
//!
//! The constants for making it bite were already authored and equally
//! unread: `RISK.SUPERNATURAL_THRESHOLD` and `RISK.HIGH_RISK` name the two
//! levels that matter, `PROBABILITIES.SUPERNATURAL_ENCOUNTER` and
//! `PROBABILITIES.HIGH_RISK_ENCOUNTER` how often each bites, and
//! `RISK.MAX_RISK_LEVEL` the ceiling the costs should clamp to.

use crate::data::*;
use crate::engine::{PassengerStateMachine, ProtectionService, RouteCosts};
use crate::state::*;

use super::RideService;

/// What a risky leg did to the player, for narration.
pub(super) struct RiskEncounter {
    pub message: String,
}

impl RideService {
    /// Roll the two risk encounters for a leg that has just been travelled.
    ///
    /// Both are gated on the leg's own risk figure, so a Police-route crawl
    /// through clear weather is genuinely safer than a shortcut in fog, which
    /// is the distinction the risk calculation was already making and nothing
    /// was acting on.
    pub(super) fn apply_risk_encounters(
        state: &mut GameState,
        costs: &RouteCosts,
        constants: &ConstantsData,
        current_time: f64,
    ) -> Option<RiskEncounter> {
        let risk = costs.risk;
        let risk_constants = &constants.risk;
        let probabilities = &constants.probabilities;

        // The worse encounter is checked first so a high-risk leg cannot be
        // downgraded into the milder one by ordering.
        if risk >= risk_constants.high_risk
            && macroquad_toolkit::rng::chance(probabilities.high_risk_encounter)
        {
            return Some(Self::high_risk_encounter(state));
        }

        if risk >= risk_constants.supernatural_threshold
            && macroquad_toolkit::rng::chance(probabilities.supernatural_encounter)
        {
            return Some(Self::supernatural_encounter(state, current_time));
        }

        None
    }

    /// A hard road costs more than the route said it would.
    fn high_risk_encounter(state: &mut GameState) -> RiskEncounter {
        let fuel_lost = 6.0_f32.min(state.fuel);
        state.fuel -= fuel_lost;
        state.time_remaining = state.time_remaining.saturating_sub(10);
        RiskEncounter {
            message: "Something in the road costs you a detour. Fuel and minutes both.".to_string(),
        }
    }

    /// Something brushes the cab. A ward will take it; otherwise the
    /// passenger feels it, which is what makes risk reach the need system and
    /// therefore the meltdown that ends most shifts.
    fn supernatural_encounter(state: &mut GameState, current_time: f64) -> RiskEncounter {
        let passenger_id = state.current_passenger.as_ref().map(|p| p.id);
        if let Some(ward) = ProtectionService::consume_ward(
            &mut state.inventory,
            ProtectionType::SupernaturalImmunity,
            passenger_id,
        ) {
            return RiskEncounter {
                message: format!(
                    "Something pulls alongside. The {} turns it away.",
                    ward.describe()
                ),
            };
        }

        if let (Some(mut need), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            let triggered =
                PassengerStateMachine::apply_stress_delta(&mut need, &passenger, 12, current_time);
            state.current_passenger_need_state = Some(need);
            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered,
                &passenger,
                state.player_trust,
                current_time,
                &state.current_guidelines,
            );
        }

        RiskEncounter {
            message: "Something pulls alongside for a while, then drops back. Your fare noticed."
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_constants;

    /// The thresholds must be ordered and reachable, or one of the two
    /// encounters can never fire: a `HIGH_RISK` above `MAX_RISK_LEVEL` would
    /// be unreachable, and one below `SUPERNATURAL_THRESHOLD` would mean the
    /// milder encounter is never even considered.
    #[test]
    fn risk_thresholds_are_ordered_and_reachable() {
        let risk = load_constants().risk;
        assert!(
            risk.supernatural_threshold < risk.high_risk,
            "supernatural threshold {} is not below high risk {}",
            risk.supernatural_threshold,
            risk.high_risk
        );
        assert!(
            risk.high_risk <= risk.max_risk_level,
            "high risk {} is above the {} ceiling and can never be reached",
            risk.high_risk,
            risk.max_risk_level
        );
        assert!(risk.extreme_risk <= risk.max_risk_level);
    }

    /// Both encounter probabilities must be live. A zero would leave the
    /// threshold authored but the encounter silent.
    #[test]
    fn both_encounter_probabilities_can_fire() {
        let probabilities = load_constants().probabilities;
        assert!(probabilities.supernatural_encounter > 0.0);
        assert!(probabilities.high_risk_encounter > 0.0);
        assert!(probabilities.supernatural_encounter <= 1.0);
        assert!(probabilities.high_risk_encounter <= 1.0);
    }
}
