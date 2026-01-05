# Rust Coding Standards for Nightmare Shift

**Engine**: Macroquad + macroquad-toolkit  
**Language**: Rust  
**Genre**: Horror Taxi Driving Survival Game

This document defines the coding standards for the Nightmare Shift project. Its goal is to maintain long-term sanity for a narrative-driven game with complex passenger state machines and rule systems. The night shift may be terrifying, but the code should be calm.

These standards prioritize:  
- Readability over cleverness  
- Data-driven design over hardcoded values  
- Clean state management for passenger interactions  
- Modular services for game logic  
- A clear mental model for game phases and transitions  

## 1. Core Philosophy

### 1.1 Write for Maintainability
This is a single-player horror game with complex passenger behaviors and rule systems. Code should be easy to debug and extend.  
- Prefer obvious, straightforward code  
- Avoid hidden state or side effects  
- If a junior Rust developer can understand the flow, you are doing it right.

### 1.2 Consistency Beats Preference
If a pattern already exists in the codebase, follow it even if you dislike it. A consistent codebase is more valuable than a perfect one.

### 1.3 Data-Driven Design
All game constants, balance values, passenger data, rules, guidelines, and static data should be defined in JSON files under `assets/`. Load this data at startup using Serde for easy balancing and iteration without recompiling code. Avoid hardcoding values in Rust code; reference loaded data structures instead.

### 1.4 No Unused Code
- Remove unused variables, fields, and functions immediately
- Never suppress unused warnings with `_` prefixes on struct fields
- If a field is unused, delete it - don't mark it as unused
- Parameter prefixes with `_` are acceptable only when required by trait signatures

## 2. Project Structure Rules

### 2.1 Module Responsibilities
Each module/subdirectory owns a single conceptual domain:

**Root Level:**
- `main.rs` – Entry point, game loop, phase transitions, and high-level coordination

**Subdirectories:**
- `data/` – Data structures and JSON loading
  - Type definitions for passengers, rules, guidelines, items
  - Constants and configuration structures
  - Environment types (weather, time, seasons)

- `engine/` – Game logic services (stateless where possible)
  - `game_engine.rs` – Core game calculations (fare, violations)
  - `passenger_service.rs` – Passenger selection and management
  - `passenger_state_machine.rs` – Need level progression
  - `guideline_engine.rs` – Tell detection and guideline evaluation
  - `route_service.rs` – Route cost calculations
  - `item_service.rs` – Item drops and effects
  - `weather_service.rs` – Weather generation and effects
  - `effects.rs` – Visual effects (particles, screen shake, transitions)

- `state/` – Game state management
  - `game_state.rs` – Current shift state (resources, passengers, phase)
  - `player_stats.rs` – Persistent player progression
  - `persistence.rs` – Save/load functionality

- `ui/` – User interface components
  - `core.rs` – Base UI utilities and styling
  - `components.rs` – Reusable UI widgets (cards, bars, panels)
  - Uses macroquad-toolkit for buttons and interactions

- `screens/` – Screen-specific rendering (if separated from main.rs)

**Cross-Domain Rules:**
- ❌ UI must never mutate game state directly
- ❌ Engine services should be stateless - receive state, return results
- ❌ Data module has no knowledge of engine or UI
- ✅ All domains can read from `data/` types
- ✅ State mutations happen only in main.rs via clearly defined actions

### 2.2 File Size Guideline
- Target: 200–400 lines per file
- Soft limit: 600 lines
- Hard limit: 800 lines (main.rs excepted for game loop complexity)
- If a file grows beyond this, split by responsibility.

### 2.3 Folder Structure

```
nightmare_shift/
├── Cargo.toml              # Project manifest
├── CODE_STANDARDS.md       # This file
├── src/
│   ├── main.rs             # Entry point, game loop, screen rendering
│   ├── data/               # Data types and loading
│   │   ├── mod.rs          # Re-exports all data types
│   │   ├── loader.rs       # JSON deserialization
│   │   ├── constants.rs    # Game constants structures
│   │   ├── passenger.rs    # Passenger types and tells
│   │   ├── rules.rs        # Rule and guideline types
│   │   ├── inventory.rs    # Item types and effects
│   │   ├── environment.rs  # Weather, time, seasons
│   │   ├── location.rs     # Location data
│   │   └── skill_tree.rs   # Skill progression types
│   ├── engine/             # Game logic services
│   │   ├── mod.rs          # Re-exports
│   │   ├── game_engine.rs  # Core calculations
│   │   ├── passenger_service.rs
│   │   ├── passenger_state_machine.rs
│   │   ├── guideline_engine.rs
│   │   ├── route_service.rs
│   │   ├── item_service.rs
│   │   ├── weather_service.rs
│   │   └── effects.rs      # Visual effects
│   ├── state/              # State management
│   │   ├── mod.rs
│   │   ├── game_state.rs   # Current shift state
│   │   ├── player_stats.rs # Persistent progression
│   │   └── persistence.rs  # Save/load
│   ├── ui/                 # UI components
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   └── components.rs
│   └── screens/            # Screen renderers (optional)
├── assets/                 # Game data
│   ├── constants.json      # Balance values
│   ├── passengerData.json  # Passenger definitions
│   ├── shiftRulesData.json # Rule definitions
│   ├── guidelineData.json  # Guideline exceptions
│   ├── locationData.json   # Pickup/destination data
│   ├── skillTreeData.json  # Skill progression
│   └── localization/       # Text strings
└── .gitignore
```

## 3. Naming Conventions

### 3.1 General Rules
- Types: PascalCase  
- Functions & variables: snake_case  
- Constants: SCREAMING_SNAKE_CASE  
- Modules: snake_case  

Names should describe what the thing is, not how it works.

Good examples:  
```rust
Passenger  
PassengerNeedState  
calculate_fare  
check_rule_violation  
spawn_rain_particles  
```

Bad examples:  
```rust
do_thing  
temp2  
handle_stuff  
p  // use passenger instead
```

### 3.2 Boolean Naming
Booleans should read like facts:  
```rust
is_supernatural  
can_accept_ride  
has_backstory_unlocked  
should_end_shift  
```  
Avoid `flag`, `value`, or `state` in names.

### 3.3 Service Naming
Engine services follow a naming pattern:
- `*Service` for stateless helpers (`PassengerService`, `RouteService`)
- `*Engine` for complex stateless processors (`GameEngine`, `GuidelineEngine`)
- `*StateMachine` for state progressions (`PassengerStateMachine`)

## 4. Functions & Methods

### 4.1 Function Size
- Target: 20–50 lines  
- Absolute max: 100 lines  
- If a function needs scrolling, it probably needs refactoring.

### 4.2 Single Responsibility
Each function should answer one question or perform one action.

Bad:  
```rust
// Calculates fare, updates reputation, drops items, checks achievements  
fn complete_ride() { ... }  
```

Good:  
```rust
fn calculate_fare() -> u32 { ... }  
fn update_passenger_reputation() { ... }  
fn check_item_drop() -> Option<Item> { ... }  
fn check_achievements() { ... }  
```

### 4.3 Argument Count
- Prefer ≤ 3 parameters  
- If more are needed, use a struct or reference to state  
- Services should take `&GameState` or `&ConstantsData` rather than many individual fields

### 4.4 Return Types
- Use `Option<T>` for potentially missing values  
- Use custom result structs for complex outcomes (e.g., `RuleViolationResult`)
- Avoid returning multiple values via tuple; create a named struct instead

## 5. Data & State Management

### 5.1 Game State Ownership
- `GameState` owns the current shift state  
- `PlayerStats` owns persistent progression  
- Mutation happens through methods on `Game` struct in main.rs  
- Services return results; they don't mutate state directly  

### 5.2 Prefer Plain Data
Use structs with clear fields. Avoid overly clever enums with embedded logic unless they model a real state machine.  

Game data should be:  
- Serializable (Serde-friendly for save/load)  
- Easy to debug and inspect  
- Immutable after loading from JSON  

### 5.3 Data-Driven Design
- All game balance, passenger stats, rules, and configuration in JSON under `assets/`
- Load data at application startup; data is embedded at compile time
- Use structs that mirror JSON structure for type safety
- Never hardcode magic numbers; reference loaded config data

### 5.4 Enums for Game Phases
Use enums to model distinct game states:
```rust
pub enum GamePhase {
    Loading,
    MainMenu,
    Briefing,
    Waiting,
    RideRequest,
    Driving,
    Interaction,
    GuidelineDecision,
    DropOff,
    GameOver,
    Success,
}
```

## 6. Error Handling

### 6.1 Prefer Option Over Panics
- `panic!` is acceptable only for truly unrecoverable states  
- Missing passengers or items should return `None`, not panic  
- Use:  
  - `Option<T>` for potentially missing values  
  - `Result<T, E>` for fallible I/O operations (save/load)  
  - Graceful degradation for missing data  

### 6.2 Logging Over Silent Failures
Use `eprintln!` for error conditions that should be visible during development but shouldn't crash the game.

## 7. UI Code (Macroquad-Toolkit)

### 7.1 UI Is Dumb
UI code:  
- Reads game state  
- Returns actions/intents  
- It should never contain game logic.  

Bad:  
```rust
// Calculating fare inside a button handler  
fn on_accept_button() { calculate_fare(); }  
```

Good:  
```rust
// Button returns UiAction::AcceptRide
// main.rs handles the action and calculations
fn draw_ride_request() -> Option<UiAction> { ... }
```

### 7.2 Action Pattern
UI components return `Option<UiAction>` to signal user intent:
```rust
pub enum UiAction {
    StartGame,
    AcceptRide,
    DeclineRide,
    SelectRoute(RouteType),
    UseItem(usize),
    // etc.
}
```

### 7.3 Component Organization
- `core.rs` – Color schemes, fonts, base styling  
- `components.rs` – Reusable widgets (StatusBar, PassengerCard, etc.)  
- Each component is a pure function: `fn draw_thing(state: &State) -> Option<UiAction>`

### 7.4 Macroquad-Toolkit Usage

This project uses `macroquad-toolkit` for common UI patterns. Import via `use ui::*;` which re-exports all toolkit modules.

**Available Modules:**
- `ui::button()` – Standard clickable button (fires on release)
- `ui::button_on_press()` – Button that fires on mouse down
- `ui::button_styled()` – Button with custom styling
- `ui::panel()` – Draws a panel with optional title
- `ui::progress_bar()` – Progress indicator
- `ui::colors::dark::*` – Standard dark theme colors
- `ui::input::*` – Mouse/keyboard input helpers

**Button Click Semantics:**
```rust
// Standard button - fires on mouse RELEASE (safer, allows cancel)
if button(x, y, w, h, "Accept Ride") {
    return UiAction::AcceptRide;
}

// Press button - fires on mouse DOWN (instant feedback)
if button_on_press(x, y, w, h, "Emergency", &style) {
    // Immediate action
}
```

**Color Palette:**
```rust
use macroquad_toolkit::colors::dark;

clear_background(dark::BACKGROUND);  // Standard background
draw_rectangle(x, y, w, h, dark::PANEL);  // Panel color
draw_text("Hello", x, y, 20.0, dark::TEXT);  // Text color
// Also: dark::ACCENT, dark::POSITIVE, dark::WARNING, dark::NEGATIVE
```

**Input Helpers:**
```rust
use ui::input::*;

if is_hovered(x, y, w, h) { /* Mouse over area */ }
if was_clicked(x, y, w, h) { /* Left click released on area */ }
if was_pressed(x, y, w, h) { /* Left click pressed on area */ }
```

## 8. Deployment & Web Standards

### 8.1 Required Files
Every game must have these files for deployment:
- `publish.ps1` – Build and deploy script
- `index.html` – WebGL host page

### 8.2 Build Targets
The game must build for:
- **Windows**: `cargo build --release`
- **Web/WASM**: `cargo build --release --target wasm32-unknown-unknown`

### 8.3 WebGL Requirements
The `index.html` must:
- Load `mq_js_bundle.js` (Miniquad loader)
- Call `load("nightmare_shift.wasm")`
- Include canvas with `id="glcanvas"`
- Use `image-rendering: pixelated` for pixel art

## 9. Game Phases & Transitions

### 9.1 Clear Phase Model
The game uses explicit phases:
1. **MainMenu** → Start game
2. **Briefing** → Display shift rules
3. **Waiting** → Between passengers, can refuel
4. **RideRequest** → Accept/decline passenger
5. **Driving** → Route selection
6. **Interaction** → Passenger dialogue
7. **GuidelineDecision** → Follow/break guideline choice
8. **DropOff** → Ride completion summary
9. **GameOver/Success** → End of shift

### 9.2 Transition Clarity
Phase transitions should be explicit and obvious in code:
```rust
// Clear: one function, one transition
fn start_shift(&mut self) {
    self.game_state.game_phase = GamePhase::Waiting;
    self.screen = Screen::Game;
}
```

## 10. Comments & Documentation

### 10.1 Comment Why, Not What
Code already explains what it does. Comments should explain why it exists.

Good:  
```rust
// Supernatural passengers ignore normal rule violations  
fn check_rule_violation() { ... }  
```

Bad:  
```rust
// Check if rule is violated  
fn check_rule_violation() { ... }  
```

### 10.2 Module-Level Docs
Each module should contain a short `//!` comment explaining its purpose:
```rust
//! Passenger state machine for need level progression.
```

## 11. Formatting & Tooling

### 11.1 rustfmt
- Always use `cargo fmt`  
- Never fight the formatter  

### 11.2 Clippy
- Run `cargo clippy` regularly  
- Fix warnings unless intentionally ignored  
- Document any `#[allow]` with a comment

### 11.3 Variable Shadowing
- Avoid variable shadowing (hiding)
- Do not declare a new variable with the same name as an existing one in the same scope

### 11.4 Unused Code
- Remove unused variables immediately
- Remove unused struct fields immediately  
- Never use `_` prefix on struct fields to suppress warnings
- `_` prefix on function parameters is acceptable when required by API

## 12. Testing Guidelines

### 12.1 What to Test
Focus tests on:  
- Fare calculations  
- Rule violation detection  
- Passenger selection logic  
- State machine transitions  
- JSON data loading  
- UI and rendering generally do not need unit tests.

### 12.2 Test Style
- Tests should read like rules  
- Avoid complex setups  
- If a test is hard to write, the code is probably too tangled.

## 13. Final Rule

If a piece of code feels fragile, confusing, or brittle, it probably is. Refactor early. Leave the night shift code calmer than you found it.