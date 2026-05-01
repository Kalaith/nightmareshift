# Nightmare Shift 🚕👻

**Nightmare Shift** is a horror-themed taxi driving survival game built with Rust and the Macroquad game engine.

Drive supernatural passengers through the night, follow mysterious rules, and try to survive until dawn. Every passenger has a story, and every route has its risks.

## 🎮 How to Play

You are a taxi driver working the graveyard shift in a city where the supernatural is the norm. Your goal is to pick up passengers, follow their specific rules, and get them to their destination safely.

### Controls

| Action | Key / Input |
|dev|dev|
| **Accept / Continue** | <kbd>SPACE</kbd> |
| **Decline / Back** | <kbd>ESC</kbd> |
| **Select Route** | <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd> |
| **Interact** | <kbd>Mouse Click</kbd> |

## 🛠️ Development

This project is built using:
- **Language:** [Rust](https://www.rust-lang.org/)
- **Engine:** [Macroquad](https://github.com/not-fl3/macroquad)
- **Toolkit:** Custom `macroquad-toolkit`

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) v1.70+
- Basic development tools for your platform (e.g., build-essential on Linux, MSVC on Windows)

### Running Locally

To run the game natively on your machine:

```bash
cargo run
```

### Automated Playtest Bot

To smoke-test the gameplay loop with the built-in bot:

```powershell
.\scripts\run-playtest-bot.ps1 -Shifts 3 -Strategy coverage
```

To sweep increasing almanac unlocks for the learned bot:

```powershell
.\scripts\run-bot-almanac-sweep.ps1 -Strategy learned -FreshStats
```

See `docs/playtest-bot.md` for strategies and command-line options.

### Building for Web (WASM)

To build the game for the web:

1.  Ensure you have the wasm32 target installed:
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
2.  Use the provided publish script (Windows):
    ```powershell
    ./publish.ps1
    ```
    Or manually build:
    ```bash
    cargo build --target wasm32-unknown-unknown --release
    ```

## 📂 Project Structure

- `src/`: Core game source code
- `assets/`: Game assets (images, data, etc.)
- `index.html`: Web entry point for the WASM build
- `nightmare_shift.wasm`: Compiled game binary (generated)

## 📜 License

See the LICENSE file for details.
