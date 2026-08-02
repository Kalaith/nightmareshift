//! Seeding for the screenshot harness.
//!
//! `capture_ui.ps1` drives the game through `NIGHTMARE_SHIFT_CAPTURE_*` and
//! names a scene; this puts the game into that state before the first frame.
//! It lives apart from the game loop because none of it is playing the game —
//! it exists so a screen can be looked at, which is how several faults in
//! this project were found.

use super::Game;
use crate::data::{RouteType, WeatherCondition, WeatherIntensity, WeatherType};
use crate::engine::{PassengerStateMachine, RideService, WeatherService};
use crate::screens::Screen;
use crate::state::*;
use macroquad::prelude::*;

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_mode = true;
        match scene {
            "briefing" => self.start_game(),
            // The main menu with the seed modal open mid-entry, so the
            // daily/seeded commands and the entry panel can be looked at.
            "seed_entry" => {
                self.seed_entry = Some("20670".to_string());
            }
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
            // On the road with a fare aboard and their need already climbing,
            // which is the only place the passenger gauge appears. No scene had
            // ever shown the driving screen mid-ride, so the gauge went
            // unlooked-at.
            "driving" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                self.game_state.earnings = 96;
                self.game_state.rides_completed = 2;
                self.game_state.time_remaining = 298;
                self.game_state.fuel = 71.0;
                if let Some(passenger) = self.game_state.current_passenger.clone() {
                    self.player_stats.mark_passenger_encountered(passenger.id);
                    let upgrades: Vec<u32> = (0..2)
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
                    self.game_state.current_passenger_need_state =
                        PassengerStateMachine::initialize(&passenger, 0.0).map(|mut need| {
                            let thresholds = need.profile.thresholds.clone();
                            need.level = thresholds.warning + 4;
                            need.stage =
                                PassengerNeedState::calculate_stage(need.level, &thresholds);
                            need.stability = 1.0 - (need.level as f32 / 100.0);
                            need
                        });
                }
                self.game_state.game_phase = GamePhase::Driving;
                self.game_state.driving_phase = Some(DrivingPhase::Pickup);
            }
            // Between rides with the tank down and the discount skills bought,
            // so the refuel prices on the buttons are the discounted ones.
            "refuelling" => {
                self.start_game();
                self.start_shift();
                self.game_state.fuel = 34.0;
                self.game_state.earnings = 128;
                self.game_state.rides_completed = 3;
                self.game_state.time_remaining = 205;
                self.player_stats.bank_balance += 5000;
                let discounts: Vec<(String, u32)> = self
                    .game_data
                    .as_ref()
                    .map(|data| {
                        data.skills
                            .iter()
                            .filter(|skill| skill.effect.target == "refuel_discount")
                            .map(|skill| (skill.id.clone(), skill.cost))
                            .collect()
                    })
                    .unwrap_or_default();
                for (id, cost) in discounts {
                    self.player_stats.purchase_skill(&id, cost);
                }
            }
            // Down to the dregs, so the routes the cab cannot reach say why.
            "driving_broke" => {
                self.begin_capture_scene("driving");
                self.game_state.fuel = 9.0;
                self.game_state.time_remaining = 14;
                self.game_state.earnings = 212;
            }
            // The same road with a hazard closing one of the routes, which
            // draws a different card entirely and had never been looked at.
            "driving_blocked" => {
                self.begin_capture_scene("driving");
                macroquad_toolkit::rng::srand(20260801);
                let blocking = self.game_data.as_ref().map(|_| {
                    let heavy = WeatherCondition {
                        weather_type: WeatherType::Thunderstorm,
                        intensity: WeatherIntensity::Heavy,
                        visibility: 25,
                        description: "Rain coming off the river in sheets".to_string(),
                        effects: Vec::new(),
                        duration: 90,
                        start_time: 0.0,
                    };
                    let mut found = Vec::new();
                    for _ in 0..256 {
                        let generated = WeatherService::generate_hazards(
                            &mut self.game_state.rng,
                            &heavy,
                            &self.game_state.time_of_day,
                            &self.game_state.season,
                            0.0,
                        );
                        if generated
                            .iter()
                            .any(|hazard| hazard.effects.route_blocked.is_some())
                        {
                            found = generated;
                            break;
                        }
                    }
                    (heavy, found)
                });
                if let Some((weather, hazards)) = blocking {
                    self.game_state.current_weather = weather;
                    self.game_state.environmental_hazards = hazards;
                }
            }
            // Mid-shift with wards in hand, so the protection readout the
            // status bar only draws when there is something to say is visible.
            "warded" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                self.game_state.earnings = 164;
                self.game_state.rides_completed = 3;
                self.game_state.time_remaining = 251;
                self.game_state.fuel = 58.0;
                self.game_state.rule_immunity_charges = 2;
                self.game_state.supernatural_protection = 1;
            }
            // The almanac with the roster at a spread of knowledge levels and
            // lore in hand, so the upgrade prices and what each next level
            // reveals are both visible in the capture.
            "almanac" => {
                self.player_stats.lore_fragments += 400;
                let roster: Vec<u32> = self
                    .game_data
                    .as_ref()
                    .map(|data| data.passengers.iter().map(|p| p.id).collect())
                    .unwrap_or_default();
                for (index, passenger_id) in roster.into_iter().enumerate() {
                    self.player_stats.mark_passenger_encountered(passenger_id);
                    // 0, 1, 2 and back round, so every tier of card is shown.
                    for _ in 0..(index % 3) {
                        let level = self
                            .player_stats
                            .get_almanac_entry(passenger_id)
                            .knowledge_level;
                        let cost = self
                            .game_data
                            .as_ref()
                            .map(|data| data.almanac.get_upgrade_cost(level + 1))
                            .unwrap_or(0);
                        self.player_stats
                            .upgrade_almanac_knowledge(passenger_id, cost);
                    }
                }
                // One passenger whose story was earned in play rather than
                // bought with lore, expanded, so the almanac honouring that is
                // visible. Selected because the backstory only shows on an
                // expanded card.
                let earned = self
                    .game_data
                    .as_ref()
                    .and_then(|data| data.passengers.first().map(|p| p.id));
                if let Some(passenger_id) = earned {
                    self.player_stats.reveal_story(passenger_id);
                    self.almanac_selected = Some(passenger_id);
                }
                self.change_screen(Screen::Almanac);
            }
            // A guideline decision with the clock running, the passenger's
            // last line on screen, and the almanac studied enough to show its
            // verdict. This screen had no scene, which is how the passenger's
            // voice went missing from it unnoticed.
            "guideline" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                if let Some(passenger) = self.game_state.current_passenger.clone() {
                    self.player_stats.mark_passenger_encountered(passenger.id);
                    let upgrades: Vec<u32> = (0..2)
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

                    // Whatever they say once they are past calm.
                    let need =
                        PassengerStateMachine::initialize(&passenger, 0.0).map(|mut need| {
                            need.stage = NeedStage::Critical;
                            need
                        });
                    let mut scene_rng = self.game_state.rng;
                    self.game_state.current_passenger_dialogue = need
                        .as_ref()
                        .and_then(|need| {
                            PassengerStateMachine::get_dialogue_for_stage(
                                &mut scene_rng,
                                &passenger,
                                need,
                            )
                        })
                        .or(self.game_state.current_passenger_dialogue.take());
                    self.game_state.rng = scene_rng;
                    self.game_state.current_passenger_need_state = need;

                    // The guideline their own exception belongs to, so the
                    // almanac verdict has something to say.
                    self.game_state.active_guideline = self.game_data.as_ref().and_then(|data| {
                        let exception_id = passenger
                            .state_profile
                            .as_ref()
                            .and_then(|profile| profile.exception_id.clone())?;
                        data.guidelines
                            .iter()
                            .find(|guideline| {
                                guideline
                                    .exceptions
                                    .iter()
                                    .any(|exception| exception.id == exception_id)
                            })
                            .cloned()
                    });
                    self.game_state.guideline_decision_start_time = Some(get_time() - 8.0);
                    self.game_state.guideline_time_remaining = 22.0;
                    self.game_state.game_phase = GamePhase::GuidelineDecision;
                }
            }
            // The briefing with hazards on the board. Clear weather generates
            // none, so the plain `briefing` scene shows the empty case and
            // this one shows the list a night is actually planned from.
            "briefing_hazards" => {
                self.start_game();
                // Hazard generation is a roll, so the scene pins the RNG and
                // takes the first draw that produces any. It still goes
                // through the real generator rather than hand-building a
                // hazard the game would never make.
                macroquad_toolkit::rng::srand(20260731);
                let hazards = self.game_data.as_ref().map(|_| {
                    let heavy = WeatherCondition {
                        weather_type: WeatherType::Thunderstorm,
                        intensity: WeatherIntensity::Heavy,
                        visibility: 30,
                        description: "Thunderheads sitting low over the river".to_string(),
                        effects: Vec::new(),
                        duration: 90,
                        start_time: 0.0,
                    };
                    let mut generated = Vec::new();
                    for _ in 0..64 {
                        generated = WeatherService::generate_hazards(
                            &mut self.game_state.rng,
                            &heavy,
                            &self.game_state.time_of_day,
                            &self.game_state.season,
                            0.0,
                        );
                        if !generated.is_empty() {
                            break;
                        }
                    }
                    (heavy, generated)
                });
                if let Some((weather, generated)) = hazards {
                    self.game_state.current_weather = weather;
                    self.game_state.environmental_hazards = generated;
                }
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

                    let mut scene_rng = self.game_state.rng;
                    let event = self.game_data.as_ref().map(|data| {
                        RideService::generate_mid_ride_event(
                            &mut scene_rng,
                            &self.game_state,
                            data,
                            &self.player_stats,
                            RouteType::Normal,
                        )
                    });
                    self.game_state.rng = scene_rng;
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
                self.overlays.pause = true;
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
                    // A cursed one, a plain one, and two that count their
                    // uses -- one fresh and one down to its last charge, so both
                    // states of the uses readout are in the capture.
                    for name in [
                        "Old Locket",
                        "Crystal Pendant",
                        "Crumpled Note",
                        "Prayer Beads",
                    ] {
                        let item = data.items.create_item(name, "Mrs. Chen", now);
                        self.game_state.inventory.push(item);
                    }
                    let mut spent = data.items.create_item("Rune Stone", "Sister Agnes", now);
                    spent.durability = Some(1);
                    self.game_state.inventory.push(spent);
                }
                self.overlays.inventory = true;
            }
            // The rules panel mid-ride, so each rule's authored reason for
            // existing is visible in the capture.
            "rules_panel" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                self.overlays.rules = true;
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
