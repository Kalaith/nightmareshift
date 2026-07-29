//! How the playtest bot decides.
//!
//! Route choice, the guideline decision, and which cab action settles a
//! stressed passenger. These deliberately mirror what a player can see: the
//! bot may only consult a passenger's file at the almanac level it has been
//! given, because a harness that knows more than the player measures a game
//! nobody is playing.

use super::{PlaytestBot, PlaytestStrategy};
use crate::data::{ActionType, GameData, Passenger, PreferenceLevel, RouteType};
use crate::engine::{GuidelineEngine, RouteService};
use crate::state::{GameState, NeedStage, PlayerStats, RouteStreak};
use crate::ui::UiAction;

impl PlaytestBot {
    /// The cab action that would relieve the current passenger, if they are
    /// stressed enough for their exception to be active and it has not been
    /// used on this leg yet.
    ///
    /// Breaking the rule that belongs to a passenger's own exception is worth
    /// `exceptionNeedAdjustment` — between -12 and -30 — which is the largest
    /// single relief in the game.
    pub(super) fn soothing_action(
        &mut self,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
    ) -> Option<String> {
        let data = data?;
        let passenger = state.current_passenger.as_ref()?;
        let need = state.current_passenger_need_state.as_ref()?;
        if need.stage < NeedStage::Warning {
            return None;
        }

        // Every forbidden action tonight's rules name.
        let forbidden: Vec<(&str, Option<u32>)> = state
            .current_rules
            .iter()
            .chain(state.hidden_rules.iter())
            .filter(|rule| rule.action_type == Some(ActionType::Forbidden))
            .filter_map(|rule| {
                rule.action_key
                    .as_deref()
                    .map(|key| (key, rule.related_guideline_id))
            })
            .collect();
        if forbidden.is_empty() {
            return None;
        }

        // Knowing which action settles a passenger is what the almanac's
        // first level buys: the dossier names their need and the levels it
        // turns at. Without it the driver has only a stability percentage and
        // a line of dialogue, so the bot guesses — and under exception
        // matching a guess usually soothes nothing.
        if stats.get_almanac_entry(passenger.id).knowledge_level == 0 {
            // A tell the driver actually noticed names the guideline it is
            // about, and that is on the screen whether or not the passenger
            // has been studied. Acting on it is what an attentive unstudied
            // driver does; the rotation is the fallback for having spotted
            // nothing yet.
            let noticed: Vec<u32> = state
                .detected_tells
                .iter()
                .filter(|tell| tell.passenger_id == passenger.id && tell.player_noticed)
                .filter_map(|tell| tell.related_guideline)
                .collect();
            if let Some((key, _)) = forbidden
                .iter()
                .find(|(_, related)| related.is_some_and(|id| noticed.contains(&id)))
            {
                return Some((*key).to_string());
            }

            let guess = forbidden[self.soothe_cursor % forbidden.len()]
                .0
                .to_string();
            self.soothe_cursor += 1;
            return Some(guess);
        }

        let exception_id = need.profile.exception_id.as_deref()?;
        let guideline_id = data
            .guidelines
            .iter()
            .find(|guideline| guideline.exceptions.iter().any(|e| e.id == exception_id))
            .map(|guideline| guideline.id)?;

        forbidden
            .into_iter()
            .find(|(_, related)| *related == Some(guideline_id))
            .map(|(key, _)| key.to_string())
    }

    /// Decide a guideline the way an informed player would: if a tell detected
    /// for this guideline names an exception where breaking is the safer
    /// course, break it; otherwise follow.
    ///
    /// Blindly following was harmless while the decision phase was
    /// unreachable. Now that it fires, following an exception with
    /// `breakingSafer` set is the "misread the passenger" death, so the
    /// non-coverage strategies have to actually read.
    pub(super) fn read_the_passenger(state: &GameState, stats: &PlayerStats) -> UiAction {
        let Some(guideline) = state.active_guideline.as_ref() else {
            return UiAction::FollowGuideline;
        };
        let Some(passenger) = state.current_passenger.as_ref() else {
            return UiAction::FollowGuideline;
        };

        // The verdict is what almanac Lv.2 buys, so the bot may only consult
        // it on a passenger it has studied. Reading it unconditionally left
        // the bot better informed than any unstudied player and hid the cost
        // of not studying: guideline deaths measured zero at every level.
        if stats.get_almanac_entry(passenger.id).knowledge_level >= 2 {
            return match GuidelineEngine::find_active_exception(
                guideline,
                passenger,
                &state.current_weather,
            ) {
                Some(exception) if exception.breaking_safer => UiAction::BreakGuideline,
                _ => UiAction::FollowGuideline,
            };
        }

        // Unstudied, the player still has the tells on the screen in front of
        // them. Inferring from those is what the decision costs without the
        // almanac — worse than the verdict, but not blind, which is what
        // always following would have modelled.
        let hints_at_exception = state
            .detected_tells
            .iter()
            .filter(|tell| tell.related_guideline == Some(guideline.id) && tell.player_noticed)
            .filter_map(|tell| tell.exception_id.as_deref())
            .any(|exception_id| {
                guideline
                    .exceptions
                    .iter()
                    .any(|e| e.id == exception_id && e.breaking_safer)
            });

        if hints_at_exception {
            UiAction::BreakGuideline
        } else {
            UiAction::FollowGuideline
        }
    }

    pub(super) fn choose_route_index(
        &mut self,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
    ) -> usize {
        let candidates = available_route_indices(state);
        if candidates.is_empty() {
            return 0;
        }

        match self.strategy {
            PlaytestStrategy::Coverage => {
                let idx = candidates[self.route_cursor % candidates.len()];
                self.route_cursor += 1;
                idx
            }
            PlaytestStrategy::Conservative => [0, 3, 2, 1]
                .into_iter()
                .find(|idx| candidates.contains(idx))
                .unwrap_or(candidates[0]),
            PlaytestStrategy::Learned => self.choose_learned_route(state, stats, data, &candidates),
        }
    }

    fn choose_learned_route(
        &mut self,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
        candidates: &[usize],
    ) -> usize {
        let Some(passenger) = state.current_passenger.as_ref() else {
            return candidates[0];
        };
        let Some(data) = data else {
            let idx = candidates[self.route_cursor % candidates.len()];
            self.route_cursor += 1;
            return idx;
        };

        let knowledge = stats.get_almanac_entry(passenger.id).knowledge_level;
        if knowledge < 2 {
            // Without almanac knowledge the bot still picks blindly, but it
            // may only pick something it can pay for — the driving screen
            // refuses unaffordable routes, so a bot that took them would
            // measure a difficulty no player can encounter.
            let affordable: Vec<usize> = candidates
                .iter()
                .copied()
                .filter(|idx| {
                    let costs =
                        RouteService::quote_route(route_for_index(*idx), state, data, stats);
                    state.fuel >= costs.fuel as f32 && state.time_remaining >= costs.time
                })
                .collect();
            let pool = if affordable.is_empty() {
                candidates
            } else {
                &affordable
            };
            let idx = pool[self.route_cursor % pool.len()];
            self.route_cursor += 1;
            return idx;
        }

        let mut evaluated = Vec::new();

        for idx in candidates {
            let route = route_for_index(*idx);
            // The same quote the engine charges and the driving screen shows.
            // The bot used to rebuild this itself and omitted the reputation
            // adjustment to passenger risk, so it could talk itself into a
            // route it could not actually pay for.
            let costs = RouteService::quote_route(route, state, data, stats);

            if state.fuel < costs.fuel as f32 || state.time_remaining < costs.time {
                continue;
            }

            evaluated.push((*idx, route, costs));
        }

        if evaluated.is_empty() {
            return candidates[0];
        }

        let min_next_route_time = evaluated
            .iter()
            .map(|(_, _, costs)| costs.time)
            .min()
            .unwrap_or(0);

        evaluated
            .iter()
            .max_by_key(|(_, route, costs)| {
                let fare = estimate_fare(data, state, passenger, *route);
                let projected_earnings = state.earnings + fare;
                let remaining_time = state.time_remaining.saturating_sub(costs.time);
                let preference_score = passenger
                    .get_route_preference(*route)
                    .map(|pref| match pref.preference {
                        PreferenceLevel::Loves => 45,
                        PreferenceLevel::Likes => 28,
                        PreferenceLevel::Neutral => 8,
                        PreferenceLevel::Dislikes => -18,
                        PreferenceLevel::Fears => -55,
                    })
                    .unwrap_or(0);

                let finish_score = if projected_earnings >= state.minimum_earnings {
                    if remaining_time == 0 {
                        260
                    } else if remaining_time < min_next_route_time {
                        -180
                    } else {
                        90 - remaining_time.min(90) as i32
                    }
                } else if remaining_time > 0 && remaining_time < min_next_route_time {
                    -240
                } else {
                    0
                };

                let shortcut_rule_penalty = if *route == RouteType::Shortcut
                    && state.rides_completed > 0
                    && state
                        .current_rules
                        .iter()
                        .any(|rule| rule.forbids_action("take_shortcut"))
                {
                    -300
                } else {
                    0
                };

                fare as i32 * 12
                    + preference_score
                    + finish_score
                    + shortcut_rule_penalty
                    + stats.get_route_usage(*route).min(20) as i32
                    - costs.time as i32 * 4
                    - costs.fuel as i32 * 3
                    - costs.risk as i32 * 8
            })
            .map(|(idx, _, _)| *idx)
            .unwrap_or(candidates[0])
    }
}

fn available_route_indices(state: &GameState) -> Vec<usize> {
    [0, 1, 2, 3]
        .into_iter()
        .filter(|idx| {
            let route = route_for_index(*idx);
            !state
                .environmental_hazards
                .iter()
                .any(|hazard| hazard.blocks_route(route))
        })
        .collect()
}

fn route_for_index(idx: usize) -> RouteType {
    match idx {
        1 => RouteType::Shortcut,
        2 => RouteType::Scenic,
        3 => RouteType::Police,
        _ => RouteType::Normal,
    }
}

fn estimate_fare(
    data: &GameData,
    state: &GameState,
    passenger: &Passenger,
    route: RouteType,
) -> u32 {
    let route_mult = match route {
        RouteType::Shortcut => data.constants.route_fares.shortcut,
        RouteType::Normal => data.constants.route_fares.normal,
        RouteType::Scenic => data.constants.route_fares.scenic,
        RouteType::Police => data.constants.route_fares.police,
    };

    let pref_mult = passenger
        .get_route_preference(route)
        .map(|pref| pref.fare_modifier)
        .unwrap_or(1.0);

    let streak_mult = route_streak_multiplier(
        state.consecutive_route_streak.as_ref(),
        route,
        data.constants.consecutive_route.penalty_per_repeat,
    );
    let rep_mult = state
        .passenger_reputation
        .get(&passenger.id)
        .map(|rep| rep.fare_multiplier(&data.constants.reputation))
        .unwrap_or(1.0);

    (passenger.fare as f32 * route_mult * pref_mult * streak_mult * rep_mult)
        .max(5.0)
        .round() as u32
}

fn route_streak_multiplier(
    streak: Option<&RouteStreak>,
    route: RouteType,
    penalty_per_repeat: f32,
) -> f32 {
    if let Some(streak) = streak {
        if streak.route_type == route && streak.count >= 2 {
            return (1.0 - (streak.count - 1) as f32 * penalty_per_repeat).max(0.25);
        }
    }

    1.0
}
