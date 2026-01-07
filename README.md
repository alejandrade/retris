# retris

A Tetris game built with Rust and the [egor](https://github.com/wick3dr0se/egor) 2D graphics engine.

## Features

- Classic Tetris gameplay with 5 pieces (I, O, T, L, S)
- 10×20 game grid with spawn area above the visible playfield
- Piece rotation with wall kick mechanics
- Line clearing when rows are completed
- Collision detection and piece locking
- Fixed screen size (640×1048) with non-resizable window
- VSync rendering for smooth gameplay

## How to run

### Native

```bash
cargo run
```

### Web (WASM)

```bash
trunk serve
```

Then open your browser to the URL shown (typically `http://localhost:8080`).

## Controls

- **Left Arrow** / **Right Arrow**: Move piece horizontally
- **Down Arrow**: Soft drop (accelerate piece downward)
- **Space**: Rotate piece clockwise (with wall kick)
- **Close window**: Quit

## Game Mechanics

- Pieces spawn at the top of the grid and fall automatically
- Pieces lock in place when they can no longer move down
- Completed horizontal lines are cleared automatically
- Game uses a fixed timestep loop for consistent physics

## Tech stack

- [`egor`](https://github.com/wick3dr0se/egor) - Cross-platform 2D graphics engine (supports native and WASM)
- `rand` - Random number generation for piece selection
- Fixed timestep game loop pattern
