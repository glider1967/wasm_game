use std::{cell::RefCell, collections::HashMap, f64::consts::PI, rc::Rc};

use crate::{
    browser::{self, window, LoopClosure},
    math::{Point, Rect},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::mpsc::{unbounded, UnboundedReceiver};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, KeyboardEvent};

#[async_trait(?Send)]
pub trait Game {
    async fn initialize(&self) -> Result<Box<dyn Game>>;
    fn update(&mut self, keystate: &KeyState);
    fn draw(&self, renderer: &Renderer);
}

const FRAME_SIZE: f32 = 1.0 / 60.0 * 1000.0;
pub struct GameLoop {
    last_frame: f64,
    accumulated_delta: f32,
}
type SharedLoopClosure = Rc<RefCell<Option<LoopClosure>>>;

impl GameLoop {
    pub async fn start(game: impl Game + 'static) -> Result<()> {
        let mut keyevent_receiver = prepare_input()?;
        let mut game = game.initialize().await?;
        let mut game_loop = GameLoop {
            last_frame: browser::now()?,
            accumulated_delta: 0.0,
        };

        let renderer = Renderer {
            context: browser::context()?,
        };
        renderer.init();

        let f: SharedLoopClosure = Rc::new(RefCell::new(None));
        let g: SharedLoopClosure = f.clone();

        let mut keystate = KeyState::new();
        *g.borrow_mut() = Some(browser::create_raf_closure(move |perf: f64| {
            process_input(&mut keystate, &mut keyevent_receiver);
            game_loop.accumulated_delta += (perf - game_loop.last_frame) as f32;
            while game_loop.accumulated_delta > FRAME_SIZE {
                game.update(&keystate);
                game_loop.accumulated_delta -= FRAME_SIZE;
            }
            game_loop.last_frame = perf;
            game.draw(&renderer);

            let _ = browser::request_animation_frame(f.borrow().as_ref().unwrap());
        }));

        browser::request_animation_frame(
            g.borrow()
                .as_ref()
                .ok_or_else(|| anyhow!("GameLoop: Loop is None"))?,
        )?;
        Ok(())
    }
}

pub struct Renderer {
    context: CanvasRenderingContext2d,
}

impl Renderer {
    pub fn init(&self) {
        self.context.set_line_width(2.0);
    }

    pub fn clear(&self, rect: &Rect) {
        self.context.clear_rect(
            rect.x.into(),
            rect.y.into(),
            rect.width.into(),
            rect.height.into(),
        );
    }

    pub fn draw_rect(&self, rect: &Rect) {
        self.context.stroke_rect(
            rect.x.into(),
            rect.y.into(),
            rect.width.into(),
            rect.height.into(),
        );
    }

    #[allow(dead_code)]
    pub fn draw_line(&self, start: &Point, end: &Point) {
        self.context.begin_path();
        self.context.move_to(start.x.into(), start.y.into());
        self.context.line_to(end.x.into(), end.y.into());
        self.context.close_path();
        self.context.stroke();
    }

    #[allow(dead_code)]
    pub fn draw_triangle(&self, p1: &Point, p2: &Point, p3: &Point) {
        self.context.begin_path();
        self.context.move_to(p1.x.into(), p1.y.into());
        self.context.line_to(p2.x.into(), p2.y.into());
        self.context.line_to(p3.x.into(), p3.y.into());
        self.context.close_path();
        self.context.stroke();
    }

    pub fn draw_circle(&self, center: &Point, radius: f32) {
        self.context.begin_path();
        let _ = self.context.arc(
            center.x.into(),
            center.y.into(),
            radius.into(),
            0.0,
            2.0 * PI,
        );
        self.context.close_path();
        self.context.stroke();
    }

    pub fn set_color(&self, str: &str) {
        self.context.set_stroke_style(&JsValue::from_str(str));
    }
}

enum KeyPress {
    KeyUp(KeyboardEvent),
    KeyDown(KeyboardEvent),
}

pub struct KeyState {
    pressed_keys: HashMap<String, KeyboardEvent>,
}

impl KeyState {
    pub fn new() -> Self {
        KeyState {
            pressed_keys: HashMap::new(),
        }
    }

    pub fn is_pressed(&self, code: &str) -> bool {
        self.pressed_keys.contains_key(code)
    }

    fn set_pressed(&mut self, code: &str, event: KeyboardEvent) {
        self.pressed_keys.insert(code.into(), event);
    }

    fn set_released(&mut self, code: &str) {
        self.pressed_keys.remove(code);
    }
}

// ブラウザからのキー入力のレシーバーを作る
fn prepare_input() -> Result<UnboundedReceiver<KeyPress>> {
    let (keydown_sender, keyevent_receiver) = unbounded();
    let keydown_sender = Rc::new(RefCell::new(keydown_sender));
    let keyup_sender = keydown_sender.clone();

    let onkeydown = browser::closure_wrap(Box::new(move |keycode: KeyboardEvent| {
        let _ = keydown_sender
            .borrow_mut()
            .start_send(KeyPress::KeyDown(keycode));
    }) as Box<dyn FnMut(KeyboardEvent)>);
    let onkeyup = browser::closure_wrap(Box::new(move |keycode: KeyboardEvent| {
        let _ = keyup_sender
            .borrow_mut()
            .start_send(KeyPress::KeyUp(keycode));
    }) as Box<dyn FnMut(KeyboardEvent)>);

    window()?.set_onkeydown(Some(onkeydown.as_ref().unchecked_ref()));
    window()?.set_onkeyup(Some(onkeyup.as_ref().unchecked_ref()));

    onkeydown.forget();
    onkeyup.forget();

    Ok(keyevent_receiver)
}

fn process_input(state: &mut KeyState, keyevent_receiver: &mut UnboundedReceiver<KeyPress>) {
    loop {
        match keyevent_receiver.try_next() {
            Ok(None) => break,
            Err(_) => break,
            Ok(Some(evt)) => match evt {
                KeyPress::KeyUp(evt) => state.set_released(&evt.code()),
                KeyPress::KeyDown(evt) => state.set_pressed(&evt.code(), evt),
            },
        }
    }
}
