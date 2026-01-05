# Rust Game Migration Guide: Web to Macroquad

This guide outlines the process of migrating React/PHP/MySQL web games to a standalone Rust application using `macroquad` and `macroquad-toolkit`.

## 1. Architecture Shift

| Feature | Web Architecture (Old) | Rust Architecture (New) |
| :--- | :--- | :--- |
| **Frontend** | React (DOM, CSS, State) | Macroquad (Canvas, Immediate Mode UI) |
| **Backend** | PHP (Server-side logic) | Rust (Internal Game Logic) |
| **Database** | MySQL (Server-based) | SQLite (Relational) or JSON (Simple) |
| **Styling** | CSS (Declarative) | Rust Code (Imperative/Constants) |
| **Data** | JSON API Responses | Rust Structs + JSON/SQLite |

## 2. Project Setup

Create a new Rust project:

```bash
cargo new my_game
cd my_game
```

Add dependencies to `Cargo.toml`. Note that `macroquad-toolkit` is assumed to be a local path in the monorepo.

```toml
[package]
name = "my_game"
version = "0.1.0"
edition = "2021"

[dependencies]
macroquad = "0.4"
macroquad-toolkit = { path = "../macroquad-toolkit" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.29", features = ["bundled"] } # Optional: for complex data
rand = "0.8"

[profile.release]
opt-level = 3
lto = true
```

### Entry Point (`src/main.rs`)

Macroquad requires a specific configuration function.

```rust
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "My Game".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(ORANGE);
        draw_text("Hello, World!", 20.0, 20.0, 30.0, DARKGRAY);
        next_frame().await;
    }
}
```

## 3. Deployment & Web Standards

Your game must be buildable for both Windows and Web (WASM).

### Build Script (`publish.ps1`)

Every game MUST have a `publish.ps1` in its root. This script handles:
1.  Building release targets (`cargo build --release` and `--target wasm32-unknown-unknown`).
2.  Packaging assets.
3.  Deploying to the XAMPP preview server or Production.

**Copy the standard script from `frontier/publish.ps1`**, ensuring you update the paths if necessary.

### Web Template (`index.html`)

For WebGL builds, you need an `index.html` that loads the WASM and provides the game container.
**Critical**: You must load `mq_js_bundle.js` (standard Miniquad loader).

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>My Game | Web Hatchery</title>
    <!-- Link shared CSS for consistent styling -->
    <link rel="stylesheet" href="../shared.css">
    <style>
        .game-container canvas {
            image-rendering: pixelated; /* Essential for pixel art */
        }
    </style>
</head>
<body>
    <div class="game-page">
        <canvas id="glcanvas" tabindex="1"></canvas>
    </div>
    
    <!-- Load Miniquad -->
    <script src="mq_js_bundle.js"></script>
    <script>
        // Load the specific WASM file for this game
        load("my_game.wasm");
    </script>
</body>
</html>
```

## 4. UI Migration (React -> Macroquad)

**Concept**: React uses a **Tree of Components** with CSS layout (Flexbox/Grid). Macroquad uses a **Game Loop** where you draw everything every frame based on X/Y coordinates.

### Layout Strategy

Since you lose CSS Flexbox, you must calculate positions manually. Create helper constants or functions.

**React (CSS):**
```css
.container { display: flex; justify-content: center; }
.button { margin: 10px; }
```

**Rust (Macroquad):**
```rust
// Define layout constants
const SCREEN_CENTER_X: f32 = screen_width() / 2.0;
const BUTTON_WIDTH: f32 = 200.0;
const PADDING: f32 = 20.0;

// In your draw loop
let start_x = SCREEN_CENTER_X - BUTTON_WIDTH / 2.0;
let mut current_y = 100.0;

if button(start_x, current_y, BUTTON_WIDTH, 50.0, "Start Game") {
   // Handle click
}
current_y += 50.0 + PADDING; // Manual margin
```

## 5. Persistence Strategy (MySQL -> SQLite/JSON)

### Option A: JSON (Recommended for Save Files)
For single-player game states (like `frontier` or `scrapyard`), serializing the entire `GameState` struct to JSON is preferred.

```rust
use std::fs;

// Save
let json = serde_json::to_string_pretty(&game_state)?;
fs::write("save_slot_0.json", json)?;

// Load
let data = fs::read_to_string("save_slot_0.json")?;
let state: GameState = serde_json::from_str(&data)?;
```

### Option B: SQLite (Recommended for Complex Data)
If you are migrating a complex database (e.g., thousands of item definitions, user stats) that was previously in MySQL, usage of `rusqlite` is appropriate.

```rust
use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn init() -> Result<Self> {
        let conn = Connection::open("game_data.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS player_stats (
                id INTEGER PRIMARY KEY,
                gold INTEGER NOT NULL
            )",
            (),
        )?;
        Ok(Self { conn })
    }
}
```

## 6. Future Image Prompts Strategy

To manage the transition from "no images" to "generated images", use a central JSON catalog.

### 1. The Prompt Catalog (`assets/image_prompts.json`)
Maintain a JSON file that acts as the source of truth for all graphical assets. **Important**: `width` and `height` must be divisible by 16.

```json
{
  "player_idle": {
    "prompt": "A futuristic space marine standing idle, pixel art style, high contrast",
    "filename": "player_idle.png",
    "width": 64,
    "height": 64
  }
}
```

### 2. Rust Implementation
Your `AssetManager` can load this JSON at startup to know which assets *should* exist.

```rust
#[derive(serde::Deserialize)]
struct AssetDef {
    filename: String,
    width: u32,
    height: u32,
}
// Load logic...
```

**Workflow:**
1.  **Define**: Add new needed assets to `image_prompts.json` with a description.
2.  **Develop**: Game uses the placeholder immediately because the file is missing.
3.  **Generate**: Use the prompts in the JSON to create images.
4.  **Provide**: Place the generated images in `assets/`. Game automatically picks them up on next run.

## 7. Migration Checklist

1.  **Define Structs**: specific types for your game entities (Player, Enemy, Item).
2.  **Implementation**:
    *   Initialize `macroquad::main`.
    *   Set up `publish.ps1` from the standard template.
    *   Create `index.html` referring to the correct WASM file.
3.  **Port Logic**: Translate PHP logic (calculations, rules) into Rust functions.
4.  **Build UI**: Rebuild the React interface using `macroquad-toolkit` calls.
5.  **Connect Data**: Wire the UI buttons to modify the Structs/DB.
