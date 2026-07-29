//! Dispatches `UiAction`s produced by the draw phase into game mutations.

use super::Game;
use crate::data::RouteType;
use crate::engine::*;
use crate::screens::Screen;
use crate::state::*;
use crate::ui::UiAction;

impl Game {
    /// Handle UI actions from draw phase
    pub fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::StartGame => {
                if self.screen == Screen::MainMenu {
                    self.start_game();
                } else if self.screen == Screen::Briefing {
                    self.start_shift();
                }
            }
            UiAction::AcceptRide => {
                if self.screen == Screen::Game {
                    self.accept_ride();
                }
            }
            UiAction::DeclineRide => {
                if self.screen == Screen::Game {
                    self.decline_ride();
                }
            }
            UiAction::SelectRoute(idx) => {
                let route_type = match idx {
                    0 => RouteType::Normal,
                    1 => RouteType::Shortcut,
                    2 => RouteType::Scenic,
                    3 => RouteType::Police,
                    _ => RouteType::Normal,
                };
                if self.screen == Screen::Game {
                    self.choose_route(route_type);
                }
            }
            UiAction::SelectEventChoice(idx) => {
                if self.screen == Screen::Game {
                    RideService::resolve_event_choice(&mut self.game_state, idx);
                }
            }
            UiAction::Continue => {
                if self.screen == Screen::Game {
                    match self.game_state.game_phase {
                        GamePhase::Waiting => self.spawn_passenger(),
                        GamePhase::Interaction => self.continue_to_destination(),
                        GamePhase::DropOff => self.continue_from_dropoff(),
                        _ => {}
                    }
                }
            }
            UiAction::ReturnToMenu => {
                // On an interim-night results screen, the confirm key presses on
                // into the next night rather than abandoning the run.
                if self.screen == Screen::Success && !self.game_state.run_complete {
                    self.advance_night();
                } else if self.screen == Screen::GameOver
                    || self.screen == Screen::Success
                    || self.screen == Screen::SkillTree
                    || self.screen == Screen::Almanac
                    || self.screen == Screen::Leaderboard
                    || (self.screen == Screen::Game && self.show_pause_menu)
                {
                    self.show_pause_menu = false;
                    self.return_to_menu();
                }
            }
            UiAction::TryAgain => {
                if self.screen == Screen::GameOver || self.screen == Screen::Success {
                    self.return_to_menu();
                    self.start_game();
                }
            }
            UiAction::NextNight => {
                if self.screen == Screen::Success && !self.game_state.run_complete {
                    self.advance_night();
                }
            }
            UiAction::EndShift => {
                if self.screen == Screen::Game
                    && self.game_state.game_phase == GamePhase::Waiting
                    && self.game_state.earnings >= self.game_state.minimum_earnings
                {
                    self.end_shift(true);
                }
            }
            UiAction::RefuelFull => {
                if self.screen == Screen::Game && self.game_state.game_phase == GamePhase::Waiting {
                    self.refuel_full();
                }
            }
            UiAction::RefuelPartial => {
                if self.screen == Screen::Game && self.game_state.game_phase == GamePhase::Waiting {
                    self.refuel_partial();
                }
            }
            UiAction::ToggleRules => {
                if self.screen == Screen::Game {
                    self.show_rules = !self.show_rules;
                }
            }
            UiAction::ToggleInventory => {
                if self.screen == Screen::Game {
                    self.show_inventory = !self.show_inventory;
                }
            }
            UiAction::TogglePauseMenu => {
                if self.screen == Screen::Game {
                    self.show_pause_menu = !self.show_pause_menu;
                    // Close other overlays when opening pause menu
                    if self.show_pause_menu {
                        self.show_rules = false;
                        self.show_inventory = false;
                    }
                }
            }
            UiAction::UseItem(idx) => {
                if self.screen == Screen::Game && idx < self.game_state.inventory.len() {
                    self.use_item(idx);
                }
            }
            UiAction::PerformRuleAction(action_key) => {
                if self.screen == Screen::Game {
                    self.perform_rule_action(action_key);
                }
            }
            UiAction::AcceptTrade(item_idx) => {
                if let Some((_, offered_item)) = self.game_state.pending_trade.take() {
                    if item_idx < self.game_state.inventory.len() {
                        // Remove the given item
                        self.game_state.inventory.remove(item_idx);
                        // Add the received item
                        self.game_state.inventory.push(offered_item);
                    }
                }
            }
            UiAction::DeclineTrade => {
                // Clear pending trade
                self.game_state.pending_trade = None;
            }
            UiAction::FollowGuideline => {
                if self.game_state.game_phase == GamePhase::GuidelineDecision {
                    self.evaluate_guideline_decision(GuidelineAction::Follow);
                }
            }
            UiAction::BreakGuideline => {
                if self.game_state.game_phase == GamePhase::GuidelineDecision {
                    self.evaluate_guideline_decision(GuidelineAction::Break);
                }
            }
            UiAction::OpenSkillTree => {
                self.change_screen(Screen::SkillTree);
            }
            UiAction::OpenAlmanac => {
                self.change_screen(Screen::Almanac);
            }
            UiAction::OpenLeaderboard => {
                self.change_screen(Screen::Leaderboard);
            }
            UiAction::DeleteSave => {
                if Persistence::delete_save().is_ok() {
                    self.player_stats = PlayerStats::new();
                    self.player_stats.init_achievements();
                }
            }
            UiAction::PurchaseSkill(skill_id) => {
                if let Some(ref data) = self.game_data {
                    if let Some(skill) = data.skills.iter().find(|s| s.id == skill_id) {
                        if self.player_stats.purchase_skill(&skill.id, skill.cost) {
                            let unlocked = self.player_stats.check_achievements(
                                self.game_state.earnings,
                                false,
                                self.game_state.rules_violated,
                            );
                            self.pay_achievement_rewards(&unlocked);
                            self.save_stats();
                        }
                    }
                }
            }
            UiAction::ExchangeLoreForBank => {
                if let Some(ref data) = self.game_data {
                    let rate = data.rewards.lore_exchange;
                    if rate.is_available()
                        && self
                            .player_stats
                            .exchange_lore_for_bank(rate.lore, rate.bank)
                    {
                        self.save_stats();
                    }
                }
            }
            UiAction::UpgradeAlmanacKnowledge(passenger_id) => {
                if let Some(ref data) = self.game_data {
                    let current_level = self
                        .player_stats
                        .get_almanac_entry(passenger_id)
                        .knowledge_level;
                    let cost = data.almanac.get_upgrade_cost(current_level + 1);
                    if self
                        .player_stats
                        .upgrade_almanac_knowledge(passenger_id, cost)
                    {
                        let unlocked = self.player_stats.check_achievements(
                            self.game_state.earnings,
                            false,
                            self.game_state.rules_violated,
                        );
                        self.pay_achievement_rewards(&unlocked);
                        self.save_stats();
                    }
                }
            }
            UiAction::None => {}
        }
    }
}
