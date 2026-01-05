# Rust Coding Standards for Fracture Command

**Engine**: Macroquad  
**Language**: Rust  
**Genre**: Strategic War Simulation with Networking

This document defines the coding standards for the Fracture Command project. Its goal is not academic purity, but long-term sanity for a complex real-time strategy game with networking. The battlefield may be chaotic, but the code should not be.

These standards prioritize:  
- Readability over cleverness  
- Determinism over performance  
- Network reliability over shortcuts  
- Data-driven design over hardcoded values  
- A clear mental model for multiplayer synchronization  

## 1. Core Philosophy

### 1.1 Write for Multiplayer Maintenance
This is a networked real-time strategy game where both clients must stay perfectly synchronized. Code should be easy to debug across network boundaries.  
- Prefer deterministic, obvious code  
- Avoid hidden state or side effects  
- Profile only when networking demands it  
- If a junior Rust developer can understand the synchronization, you are doing it right.

### 1.2 Consistency Beats Preference
If a pattern already exists in the codebase, follow it even if you dislike it. A consistent codebase is more valuable than a perfect one.

### 1.3 Data-Driven Design
All game constants, balance values, unit stats, doctrines, and static data should be defined in JSON files under `assets/`. Load this data at startup using Serde for easy modding, balancing, and iteration without recompiling code. Avoid hardcoding values in Rust code; reference loaded data structures instead.

## 2. Project Structure Rules

### 2.1 Module Responsibilities
Each module/subdirectory owns a single conceptual domain:

**Root Level:**
- `main.rs` – Entry point only: initialization, game loop, high-level coordination

**Subdirectories:**
- `game/` – Game state ownership and coordination
  - Game state struct and public API
  - Victory conditions, state transitions
  - Orchestrates simulation, input, and rendering

- `simulation/` – Game simulation (deterministic, network-ready)
  - Autonomous unit AI and squad behavior
  - Combat resolution and damage calculation
  - Sector control mechanics

- `rendering/` – Visual output (side-effect free)
  - World rendering (units, sectors, battlefield)
  - UI panels, HUD, menus
  - Camera management

- `input/` – Input handling and command processing
  - Keyboard and mouse input
  - Translates input to game commands
  - Camera controls

- `network/` – Peer-to-peer networking (Phase 3)
  - Connection management
  - Packet sending/receiving
  - Synchronization logic

**Shared:**
- `types.rs` – Shared data structures, enums
- `config.rs` – JSON data loading and configuration management

**Cross-Domain Rules:**
- ❌ Rendering must never mutate game state
- ❌ Network must never render
- ❌ Input emits commands, doesn't directly mutate state
- ❌ Simulation has no knowledge of input or rendering
- ✅ All domains can read from `types.rs` and `config.rs`

### 2.2 File Size Guideline
- Target: 200–400 lines per file
- Hard limit: 800 lines
- If a file grows beyond this, split by responsibility.

### 2.3 Folder Structure

**Philosophy**: Use subdirectories to organize related code, but avoid deep nesting. One level of subdirectories is usually sufficient. Two levels maximum, only when absolutely justified.

```
fracture/
├── Cargo.toml              # Project manifest
├── CLAUDE.md               # AI assistant guidance
├── CODE_STANDARDS.md       # This file
├── design.md               # Game design document
├── implementation_plan.md  # Implementation roadmap
├── src/
│   ├── main.rs            # Entry point, initialization
│   ├── game/              # Game state and coordination
│   │   ├── mod.rs         # Main game state, public API
│   │   └── state.rs       # State management helpers (if needed)
│   ├── simulation/        # Unit AI, combat, doctrines
│   │   ├── mod.rs         # Simulation coordinator
│   │   ├── ai.rs          # Autonomous unit AI
│   │   └── combat.rs      # Combat resolution
│   ├── rendering/         # Drawing and visual effects
│   │   ├── mod.rs         # Rendering coordinator
│   │   ├── world.rs       # World rendering (units, sectors)
│   │   └── ui.rs          # UI panels and HUD
│   ├── input/             # Input handling
│   │   └── mod.rs         # Input processing
│   ├── network/           # Networking (Phase 3)
│   │   └── mod.rs         # Network manager
│   ├── types.rs           # Shared data structures
│   └── config.rs          # JSON data loading and configuration management
├── assets/                 # Game assets
│   ├── data/             # JSON configuration and game data files
│   ├── textures/
│   ├── sounds/
│   └── fonts/
└── tests/                  # Integration tests
    └── simulation_tests.rs
```

**Key Principles:**

1. **Use Subdirectories for Domain Grouping**
   - Group related modules into subdirectories (e.g., `rendering/`, `simulation/`)
   - Each subdirectory represents a clear conceptual domain
   - Use `mod.rs` as the public interface for each subdirectory

2. **Avoid Deep Nesting** (Max 2 levels)
   - ✅ Good: `src/rendering/ui.rs`
   - ✅ Good: `src/simulation/combat.rs`
   - ❌ Bad: `src/rendering/ui/panels/commander/panel.rs`
   - ❌ Bad: `src/simulation/ai/units/squad/orders/advance.rs`

3. **When to Use Subdirectories**
   - A domain has 2+ related modules (e.g., `rendering/world.rs` + `rendering/ui.rs`)
   - A single file exceeds 500 lines and can be logically split
   - Clear separation of concerns within a domain

4. **When NOT to Use Subdirectories**
   - Single-file modules (just use `src/types.rs`, not `src/types/mod.rs`)
   - Premature organization (don't create subdirs "for the future")
   - If a subdirectory would only contain `mod.rs`

5. **File Naming Within Subdirectories**
   - `mod.rs` - Public interface and re-exports
   - Descriptive names for implementation files (`combat.rs`, `ui.rs`, `ai.rs`)
   - Avoid redundant names (`src/rendering/rendering.rs` ❌, use `src/rendering/mod.rs` ✅)

6. **Refactoring Trigger**
   - If `mod.rs` exceeds 400 lines, split into sibling files
   - If a subdirectory has >5 files, consider if it needs one more level (rarely)
   - If you're creating a 3rd level, stop and reconsider the architecture

## 3. Naming Conventions

### 3.1 General Rules
- Types: PascalCase  
- Functions & variables: snake_case  
- Constants: SCREAMING_SNAKE_CASE  
- Modules: snake_case  

Names should describe what the thing is, not how it works.

Good examples:  
```rust
UnitSquad  
calculate_supply_cost  
apply_doctrine_effect  
send_input_packet  
```

Bad examples:  
```rust
do_thing  
temp2  
handle_stuff  
```

### 3.2 Boolean Naming
Booleans should read like facts:  
```rust
is_alive  
can_deploy_unit  
has_supply_line  
is_doctrine_active  
```  
Avoid `flag`, `value`, or `state` in names.

## 4. Functions & Methods

### 4.1 Function Size
- Target: 20–50 lines  
- Absolute max: 100 lines  
- If a function needs scrolling, it probably needs refactoring.

### 4.2 Single Responsibility
Each function should answer one question or perform one action.

Bad:  
```rust
// Updates unit position, checks collisions, applies damage, sends network sync  
fn process_unit_tick() { ... }  
```

Good:  
```rust
fn update_unit_position() { ... }  
fn check_unit_collisions() { ... }  
fn apply_combat_damage() { ... }  
fn sync_unit_state() { ... }  
```

### 4.3 Argument Count
- Prefer ≤ 3 parameters  
- If more are needed, use a struct  
- This improves readability and future extensibility.

## 5. Data & State Management

### 5.1 Game State Ownership
- There should be a single authoritative game state  
- Mutation happens through well-defined systems  
- Network synchronization requires immutable state transitions.

### 5.2 Prefer Plain Data
Use structs with clear fields. Avoid overly clever enums with embedded logic unless they model a real-world state machine.  

Game data should be:  
- Serializable (Serde-friendly for networking)  
- Deterministic (same inputs produce same outputs)  
- Easy to debug and inspect  

### 5.3 Data-Driven Design
- All game balance, unit stats, doctrines, and static configuration should be stored in JSON files under `assets/data/`.
- Load data at application startup using Serde; avoid runtime file I/O for performance.
- Use structs that mirror JSON structure for type safety.
- This enables modding, easy balancing tweaks, and reduces code recompilation.
- Never hardcode magic numbers or balance values in code; always reference loaded config data.  

## 6. Error Handling

### 6.1 Prefer Result Over Panics
- `panic!` is acceptable only for truly unrecoverable states  
- Network disconnections should never panic  
- Use:  
  - `Result<T, E>` for fallible operations  
  - Graceful degradation for network errors  

### 6.2 Custom Error Types
For domain errors (networking, simulation), define small error enums instead of using strings.

## 7. UI Code (Macroquad)

### 7.1 UI Is Dumb
UI code:  
- Reads game state  
- Sends intent (commands)  
- It should never contain simulation logic.  

Bad:  
```rust
// Calculating supply cost inside a button handler  
fn on_deploy_button() { calculate_supply_cost(); }  
```

Good:  
```rust
// Button emits Command::DeployUnit  
fn on_deploy_button() { emit(Command::DeployUnit); }  
// Simulation handles consequences  
```

### 7.2 Deterministic Rendering
Rendering must be deterministic and free of side effects. No mutation during draw calls. Network state should not affect rendering directly.

## 8. Simulation & Time

### 8.1 Explicit Ticks
All time progression must be explicit:  
- No hidden updates in getters  
- No background mutation  
- Game tick logic should live in one clearly named function.

### 8.2 Determinism First
Randomness must be:  
- Seeded identically on both clients  
- Centralized in simulation module  
- This ensures network synchronization and replay consistency.

## 9. Networking Standards

### 9.1 Packet Design
- Packets should be small and focused  
- Use enums for packet types  
- Include sequence numbers for ordering  
- Prefer unreliable UDP for inputs, reliable for critical state

### 9.2 Synchronization
- Send inputs, not full state  
- Both clients must produce identical results from same inputs  
- Handle late packets with interpolation, not rollback  
- Document all synchronization points

### 9.3 Threading
- Network runs in separate Tokio thread  
- Use channels for communication  
- Never block the main game loop  

## 10. Comments & Documentation

### 10.1 Comment Why, Not What
Code already explains what it does. Comments should explain why it exists.

Good:  
```rust
// Supply caps prevent overwhelming the renderer with too many units  
fn enforce_supply_limit() { ... }  
```

Bad:  
```rust
// Limit supply to 100  
fn limit_supply() { ... }  
```

### 10.2 Module-Level Docs
Each module should contain a short `//!` comment explaining its purpose and boundaries.

## 11. Formatting & Tooling

### 11.1 rustfmt
- Always use `cargo fmt`  
- Never fight the formatter  

### 11.2 Clippy
- Run `cargo clippy` regularly  
- Fix warnings unless intentionally ignored  
- Document any `#[allow]` with a comment.

### 11.3 Variable Shadowing
- Avoid variable shadowing (hiding). Do not declare a new variable with the same name as an existing one in the same scope.  
- Unused variables must trigger a warning. Never suppress unused variable warnings with `_` prefixes or `#[allow(unused_variables)]`.

## 12. Testing Guidelines

### 12.1 What to Test
Focus tests on:  
- Simulation calculations (combat, supply, doctrines)  
- Network packet serialization/deserialization  
- Deterministic behavior  
- UI and rendering generally do not need unit tests.

### 12.2 Test Style
- Tests should read like rules  
- Avoid complex setups  
- If a test is hard to write, the code is probably too tangled.

## 13. Final Rule
If a piece of code feels fragile, confusing, or brittle, it probably is. Refactor early. Leave the battlefield better than you found it.