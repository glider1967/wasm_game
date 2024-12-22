use anyhow::Result;
use engine::GameLoop;
use game::StgGame;
use wasm_bindgen::prelude::*;

#[macro_use]
mod browser;
mod engine;
mod game;
mod level;
mod math;
mod player;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    browser::spawn_local(async move {
        let game = StgGame::new();
        GameLoop::start(game)
            .await
            .expect("Could not start game loop");
    });

    Ok(())
}
