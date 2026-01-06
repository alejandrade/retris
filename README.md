# retris

A Tetris game built with Rust, wgpu, SDL2, and Kira.

## What it does

A Tetris implementation with a fixed timestep game loop and VSync rendering.

## How to run

```bash
cargo run
```

## Controls

- **Arrow keys**: Move the cube
- **Mouse click**: Move cube to click position
- **Close window**: Quit

## Tech stack

- `wgpu` - Graphics rendering
- `SDL2` - Window and input handling
- `Kira` - Audio (initialized, not used yet)
- Fixed timestep game loop pattern
