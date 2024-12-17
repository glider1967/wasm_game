use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::{
    engine::{Game, KeyState, Rect, Renderer},
    level::Level,
};

pub enum StgGame {
    Loading,
    Loaded(Level),
}

impl StgGame {
    pub fn new() -> Self {
        StgGame::Loading
    }
}

#[async_trait(?Send)]
impl Game for StgGame {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            StgGame::Loading => Ok(Box::new(StgGame::Loaded(Level::new()))),
            StgGame::Loaded(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let StgGame::Loaded(level) = self {
            level.update(keystate);
        }
    }

    fn draw(&self, renderer: &Renderer) {
        let whole_canvas = Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        };

        renderer.clear(&whole_canvas);

        if let StgGame::Loaded(level) = self {
            renderer.set_color("gray");
            renderer.draw_rect(&Rect {
                x: 50.0,
                y: 30.0,
                width: 500.0,
                height: 540.0,
            });
            level.draw(renderer);
        }
    }
}
