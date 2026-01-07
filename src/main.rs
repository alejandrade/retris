mod game;
mod tetris_shape;
mod grid;

use egor::app::App;
use game::Game;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 1048;

fn main() {
    let mut game = Game::new();
    App::new()
        .title("Egor Tetris")
        .screen_size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .resizable(false)
        .vsync(true)
        .run(move |gfx, input, timer| {
            game.update(input, timer.delta);
            game.draw(gfx, timer.delta);
        })
}
