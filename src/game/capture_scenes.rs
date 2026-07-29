//! Seeding for the screenshot harness.
//!
//! `capture_ui.ps1` drives the game through `NIGHTMARE_SHIFT_CAPTURE_*` and
//! names a scene; this puts the game into that state before the first frame.
//! It lives apart from the game loop because none of it is playing the game —
//! it exists so a screen can be looked at, which is how several faults in
//! this project were found.

use super::Game;
use crate::data::RouteType;
use crate::engine::RideService;
use crate::screens::Screen;
use crate::state::*;
use macroquad::prelude::*;

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_mode = true;
        match scene {
            "briefing" => self.start_game(),
            // The skill tree with currency in hand, so the purchase buttons
            // and the lore exchange are both live in the capture.
            "skill_tree" => {
                self.player_stats.bank_balance += 2500;
                self.player_stats.lore_fragments += 40;
                self.change_screen(Screen::SkillTree);
            }
            "gameplay" => {
                self.start_game();
                self.start_shift();
            }
            // A mid-ride event with the passenger's ability choice present:
            // the almanac studied and every skill bought, so both kinds of
            // footnote appear at once — the risk tags on the authored
            // choices and the trait hint on the earned one.
            "event" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                if let Some(passenger) = self.game_state.current_passenger.clone() {
                    self.player_stats.mark_passenger_encountered(passenger.id);
                    self.player_stats.bank_balance += 5000;

                    let upgrades: Vec<u32> = (0..3)
                        .map(|step| {
                            self.game_data
                                .as_ref()
                                .map(|data| data.almanac.get_upgrade_cost(step + 1))
                                .unwrap_or(0)
                        })
                        .collect();
                    for cost in upgrades {
                        self.player_stats.lore_fragments += 99;
                        self.player_stats
                            .upgrade_almanac_knowledge(passenger.id, cost);
                    }

                    let purchases: Vec<(String, u32)> = self
                        .game_data
                        .as_ref()
                        .map(|data| {
                            passenger
                                .traits
                                .iter()
                                .filter_map(|trait_name| {
                                    let id = RideService::trait_skill_id(trait_name);
                                    data.skills
                                        .iter()
                                        .find(|skill| skill.id == id)
                                        .map(|skill| (skill.id.clone(), skill.cost))
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    for (id, cost) in purchases {
                        self.player_stats.purchase_skill(&id, cost);
                    }

                    let event = self.game_data.as_ref().map(|data| {
                        RideService::generate_mid_ride_event(
                            &self.game_state,
                            data,
                            &self.player_stats,
                            RouteType::Normal,
                        )
                    });
                    self.game_state.current_event = event;
                    self.game_state.game_phase = GamePhase::Interaction;
                }
            }
            // Where a run ends. Three states of the same screen: a lost
            // shift, an interim night survived, and a completed run.
            "game_over" => {
                self.start_game();
                self.game_state.night = 3;
                self.game_state.earnings = 118;
                self.game_state.rides_completed = 4;
                self.game_state.time_remaining = 96;
                self.game_state.game_over_reason =
                    Some("The passenger's need became uncontrollable.".to_string());
                self.game_state.game_phase = GamePhase::GameOver;
                self.change_screen(Screen::GameOver);
            }
            "night_complete" => {
                self.start_game();
                self.game_state.night = 2;
                self.game_state.earnings = 288;
                self.game_state.rides_completed = 7;
                self.game_state.time_remaining = 74;
                self.game_state.run_complete = false;
                self.game_state.shift_payout = MetaPayout {
                    bank: 144,
                    lore: 11,
                    ..MetaPayout::default()
                };
                self.game_state.game_phase = GamePhase::Success;
                self.change_screen(Screen::Success);
            }
            "run_complete" => {
                self.start_game();
                self.game_state.night = 5;
                self.game_state.earnings = 471;
                self.game_state.rides_completed = 9;
                self.game_state.time_remaining = 51;
                self.game_state.run_complete = true;
                self.game_state.shift_payout = MetaPayout {
                    bank: 235,
                    lore: 14,
                    run_bonus_bank: 1500,
                    run_bonus_lore: 15,
                };
                self.game_state.game_phase = GamePhase::Success;
                self.change_screen(Screen::Success);
            }
            // The leaderboard with a spread of recorded runs, so the ranking
            // and the achievement list are both populated in the capture.
            // Paused mid-shift with money on the meter, so the pause menu's
            // forfeit warning is visible in the capture.
            "paused" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                self.game_state.earnings = 186;
                self.game_state.rides_completed = 4;
                self.game_state.time_remaining = 233;
                self.game_state.fuel = 61.0;
                self.show_pause_menu = true;
            }
            // The menu with the delete button already armed, so the warning
            // state is visible without a click.
            "delete_armed" => {
                self.player_stats.bank_balance += 4200;
                self.player_stats.lore_fragments += 260;
                self.delete_armed_until = Some(get_time() + 3600.0);
                self.change_screen(Screen::MainMenu);
            }
            "leaderboard" => {
                let entries = [
                    (1840_u32, 9_u32, 4_u32, 0_u32, true),
                    (1470, 7, 3, 1, true),
                    (1120, 6, 3, 0, true),
                    (860, 5, 2, 2, false),
                    (410, 3, 1, 1, false),
                    (150, 1, 0, 3, false),
                ];
                for (score, rides, difficulty, violations, survived) in entries {
                    self.player_stats.add_leaderboard_entry(LeaderboardEntry {
                        score,
                        date: "2026-07-29 23:15".to_string(),
                        survived,
                        passengers_transported: rides,
                        difficulty_level: difficulty,
                        rules_violated: violations,
                    });
                }
                self.change_screen(Screen::Leaderboard);
            }
            // The inventory holding a cursed item and a plain one, so the
            // curse line and its way out are visible in the capture.
            "inventory" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                if let Some(data) = &self.game_data {
                    let now = get_time();
                    for name in ["Old Locket", "Crystal Pendant", "Crumpled Note"] {
                        let item = data.items.create_item(name, "Mrs. Chen", now);
                        self.game_state.inventory.push(item);
                    }
                }
                self.show_inventory = true;
            }
            // The rules panel mid-ride, so each rule's authored reason for
            // existing is visible in the capture.
            "rules_panel" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                self.show_rules = true;
            }
            // A trade offer with a mixed inventory: an item this passenger
            // wants, one they do not, and one that cannot be traded at all.
            // Exercises the wanted-item highlight and the tradeable filter.
            "trade" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                if let Some(data) = &self.game_data {
                    if let Some(passenger) = self.game_state.current_passenger.clone() {
                        let now = get_time();
                        let wanted = passenger
                            .wanted_items
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "Old Key".to_string());
                        for name in ["Blessed Medallion", "Crumpled Note", wanted.as_str()] {
                            let item = data.items.create_item(name, &passenger.name, now);
                            self.game_state.inventory.push(item);
                        }
                        let offered = data.items.create_item("Tarot Card", &passenger.name, now);
                        self.game_state.last_ride_completion = Some(RideCompletion {
                            passenger: passenger.clone(),
                            fare_earned: passenger.fare,
                            items_received: Vec::new(),
                            backstory_unlocked: None,
                        });
                        self.game_state.pending_trade = Some((passenger.name.clone(), offered));
                        self.game_state.game_phase = GamePhase::DropOff;
                    }
                }
            }
            // The same drop-off one click later, with the swap already made.
            // The result of a trade is the part the player is told about, and
            // it was written into a field only the driving screen renders, so
            // there was nothing to look at until now.
            "trade_done" => {
                self.begin_capture_scene("trade");
                let wanted = self
                    .game_state
                    .current_passenger
                    .as_ref()
                    .and_then(|passenger| passenger.wanted_items.first().cloned());
                let idx = wanted
                    .and_then(|name| {
                        self.game_state
                            .inventory
                            .iter()
                            .position(|item| item.name == name)
                    })
                    .unwrap_or(0);
                self.complete_trade(idx);
            }
            // A ride offer with the almanac fully studied, so the dossier the
            // request screen draws is visible in the capture.
            "ride_request" => {
                self.start_game();
                self.start_shift();
                if let Some(data) = &self.game_data {
                    for passenger in &data.passengers {
                        self.player_stats.mark_passenger_encountered(passenger.id);
                        for _ in 0..3 {
                            self.player_stats.lore_fragments += 99;
                            let level = self
                                .player_stats
                                .get_almanac_entry(passenger.id)
                                .knowledge_level;
                            let cost = data.almanac.get_upgrade_cost(level + 1);
                            self.player_stats
                                .upgrade_almanac_knowledge(passenger.id, cost);
                        }
                    }
                }
                self.spawn_passenger();
            }
            _ => {
                // Default: main menu. The boot flow lands here automatically
                // after a couple of loading frames (see `update`), so no
                // seeding is needed.
            }
        }
    }
}
