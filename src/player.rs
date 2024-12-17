use crate::{
    engine::{KeyState, Renderer},
    level::Bullet,
};

use self::player_states::*;

pub struct Player {
    state_machine: PlayerStateMachine,
}

impl Player {
    pub fn new() -> Self {
        Self {
            state_machine: PlayerStateMachine::Alive(PlayerState::new()),
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        self.state_machine.draw(renderer);
    }

    pub fn update(&mut self, vx: f32, vy: f32) {
        self.state_machine = self.state_machine.update().set_velocity(vx, vy);
    }

    pub fn bomb(&mut self) {
        self.state_machine = self.state_machine.transition(PlayerEvent::Bomb);
    }

    pub fn hit(&mut self) {
        self.state_machine = self.state_machine.transition(PlayerEvent::Hit);
    }

    pub fn is_collided(&self, bullet: &Bullet) -> bool {
        self.state_machine
            .context()
            .is_collided(&bullet.pos(), 10.0)
    }

    pub fn calc_velocity(keystate: &KeyState) -> (f32, f32) {
        let w = keystate.is_pressed("KeyW");
        let a = keystate.is_pressed("KeyA");
        let s = keystate.is_pressed("KeyS");
        let d = keystate.is_pressed("KeyD");
        let slow_factor = if keystate.is_pressed("KeyK") {
            0.6
        } else {
            1.0
        };
        let x_direction = match (a, d) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };
        let y_direction = match (w, s) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };

        let diag_factor = if x_direction != 0.0 && y_direction != 0.0 {
            0.71
        } else {
            1.0
        };
        let velocity_x = 6.0 * x_direction * diag_factor * slow_factor;
        let velocity_y = 6.0 * y_direction * diag_factor * slow_factor;
        (velocity_x, velocity_y)
    }
}

#[derive(Clone, Copy)]
enum PlayerStateMachine {
    Alive(PlayerState<Alive>),         // 生きている（通常状態）
    Bombing(PlayerState<Bombing>),     // ボム状態
    Reloading(PlayerState<Reloading>), // 被弾からの復帰状態
}

impl From<PlayerState<Alive>> for PlayerStateMachine {
    fn from(state: PlayerState<Alive>) -> Self {
        PlayerStateMachine::Alive(state)
    }
}

impl From<PlayerState<Bombing>> for PlayerStateMachine {
    fn from(state: PlayerState<Bombing>) -> Self {
        PlayerStateMachine::Bombing(state)
    }
}

impl From<PlayerState<Reloading>> for PlayerStateMachine {
    fn from(state: PlayerState<Reloading>) -> Self {
        PlayerStateMachine::Reloading(state)
    }
}

pub enum PlayerEvent {
    Bomb,           // ボム
    Hit,            // 被弾
    Update,         // フレームごとの更新
    Move(f32, f32), // プレイヤー速度の更新
}

impl PlayerStateMachine {
    /// 状態遷移をする。
    /// プレイヤーが適切な状態の時に適切なイベントが起こったときにのみ、状態が変わる。
    fn transition(self, event: PlayerEvent) -> Self {
        match (self, event) {
            // キー入力からの速度更新。被弾から復帰中はイベントを受け付けないことに注意。
            (PlayerStateMachine::Alive(state), PlayerEvent::Move(vx, vy)) => {
                state.set_velocity(vx, vy).into()
            }
            (PlayerStateMachine::Bombing(state), PlayerEvent::Move(vx, vy)) => {
                state.set_velocity(vx, vy).into()
            }

            // 通常状態から他の状態への移行。通常状態への復帰は特定フレーム後に自動的に行われる。
            (PlayerStateMachine::Alive(state), PlayerEvent::Bomb) => state.bomb().into(),
            (PlayerStateMachine::Alive(state), PlayerEvent::Hit) => state.hit().into(),

            // 更新処理はすべての状態に行う。
            (PlayerStateMachine::Alive(state), PlayerEvent::Update) => state.update().into(),
            (PlayerStateMachine::Bombing(state), PlayerEvent::Update) => state.update().into(),
            (PlayerStateMachine::Reloading(state), PlayerEvent::Update) => state.update().into(),

            // 他の場合は状態を変えない。
            _ => self,
        }
    }

    fn context(&self) -> &PlayerContext {
        match self {
            PlayerStateMachine::Alive(state) => &state.context(),
            PlayerStateMachine::Bombing(state) => &state.context(),
            PlayerStateMachine::Reloading(state) => &state.context(),
        }
    }

    fn update(self) -> Self {
        self.transition(PlayerEvent::Update)
    }

    fn set_velocity(self, vx: f32, vy: f32) -> Self {
        self.transition(PlayerEvent::Move(vx, vy))
    }

    fn draw(&self, renderer: &Renderer) {
        match self {
            PlayerStateMachine::Alive(state) => state.draw(renderer),
            PlayerStateMachine::Bombing(state) => state.draw(renderer),
            PlayerStateMachine::Reloading(state) => state.draw(renderer),
        }
    }
}

mod player_states {
    use std::marker::PhantomData;

    use crate::engine::{Point, Rect, Renderer};

    use super::PlayerStateMachine;
    const FLOOR: f32 = 475.0;
    const NORMAL_LOOP: u8 = 30;
    const RELOAD_TIME: u8 = 120;
    const BOMB_TIME: u8 = 60;

    #[derive(Clone, Copy)]
    pub struct PlayerState<S> {
        context: PlayerContext,
        _state: PhantomData<S>,
    }

    #[derive(Clone, Copy)]
    pub struct PlayerContext {
        frame: u8,
        position: Point,
        velocity: Point,
    }

    impl PlayerContext {
        fn update(mut self, frame_count: u8) -> Self {
            if self.frame < frame_count {
                self.frame += 1;
            } else {
                self.frame = 0;
            }
            self.position.x += self.velocity.x;
            self.position.y += self.velocity.y;

            if self.position.x < 50.0 {
                self.position.x = 50.0;
            }
            if self.position.x > 550.0 {
                self.position.x = 550.0;
            }
            if self.position.y < 30.0 {
                self.position.y = 30.0;
            }
            if self.position.y > 570.0 {
                self.position.y = 570.0;
            }

            self
        }

        fn reset_frame(mut self) -> Self {
            self.frame = 0;
            self
        }

        pub fn is_collided(&self, point: &Point, radius: f32) -> bool {
            let dx = point.x - self.position.x;
            let dy = point.y - self.position.y;
            let distance = dx * dx + dy * dy;
            let r = radius + 3.0;
            distance < r * r
        }
    }

    #[derive(Clone, Copy)]
    pub struct Alive;
    #[derive(Clone, Copy)]
    pub struct Bombing;
    #[derive(Clone, Copy)]
    pub struct Reloading;

    impl<S> PlayerState<S> {
        pub fn context(&self) -> &PlayerContext {
            &self.context
        }

        pub fn set_velocity(mut self, vx: f32, vy: f32) -> Self {
            self.context.velocity.x = vx;
            self.context.velocity.y = vy;
            self
        }
    }

    impl PlayerState<Alive> {
        pub fn new() -> Self {
            PlayerState {
                context: PlayerContext {
                    frame: 0,
                    position: Point { x: 300.0, y: FLOOR },
                    velocity: Point { x: 0.0, y: 0.0 },
                },
                _state: PhantomData,
            }
        }

        pub fn bomb(self) -> PlayerState<Bombing> {
            PlayerState {
                context: self.context.reset_frame(),
                _state: PhantomData,
            }
        }

        pub fn hit(self) -> PlayerState<Reloading> {
            PlayerState {
                context: self.context.reset_frame(),
                _state: PhantomData,
            }
        }

        pub fn update(mut self) -> Self {
            self.context = self.context.update(NORMAL_LOOP);
            self
        }

        pub fn draw(&self, renderer: &Renderer) {
            renderer.set_color("red");
            let center = &Point {
                x: self.context.position.x,
                y: self.context.position.y,
            };
            renderer.draw_rect(&Rect {
                x: center.x - 10.0,
                y: center.y + 10.0,
                width: 20.0,
                height: -20.0 - self.context.frame as f32,
            });

            renderer.draw_circle(center, 3.0);
        }
    }

    impl PlayerState<Bombing> {
        pub fn update(mut self) -> BombingEndState {
            self.context = self.context.update(BOMB_TIME);

            // `BOMB_TIME`経過したら通常状態へ。そうでないならまだボム中。
            if self.context.frame >= BOMB_TIME {
                BombingEndState::Complete(self.end_bomb())
            } else {
                BombingEndState::Bombing(self)
            }
        }

        pub fn end_bomb(self) -> PlayerState<Alive> {
            PlayerState {
                context: self.context.reset_frame(),
                _state: PhantomData,
            }
        }

        pub fn draw(&self, renderer: &Renderer) {
            renderer.set_color("blue");
            let center = &Point {
                x: self.context.position.x,
                y: self.context.position.y,
            };
            renderer.draw_rect(&Rect {
                x: center.x - 10.0,
                y: center.y + 10.0,
                width: 20.0,
                height: -20.0 - self.context.frame as f32,
            });

            renderer.draw_circle(center, 3.0);
        }
    }

    pub enum BombingEndState {
        Complete(PlayerState<Alive>),
        Bombing(PlayerState<Bombing>),
    }

    impl PlayerState<Reloading> {
        pub fn update(mut self) -> ReloadEndState {
            self.context = self.context.update(RELOAD_TIME);

            // `RELOAD_TIME`経過したら通常状態へ。そうでないならまだ復帰中。
            if self.context.frame >= RELOAD_TIME {
                ReloadEndState::Complete(self.end_reload())
            } else {
                ReloadEndState::Reloading(self)
            }
        }

        pub fn end_reload(self) -> PlayerState<Alive> {
            PlayerState {
                context: self.context.reset_frame(),
                _state: PhantomData,
            }
        }

        pub fn draw(&self, renderer: &Renderer) {
            renderer.set_color("yellow");
            let center = &Point {
                x: 300.0,
                y: FLOOR + (RELOAD_TIME - self.context.frame) as f32,
            };
            renderer.draw_rect(&Rect {
                x: center.x - 10.0,
                y: center.y + 10.0,
                width: 20.0,
                height: -20.0,
            });

            renderer.draw_circle(center, 3.0);
        }
    }

    pub enum ReloadEndState {
        Complete(PlayerState<Alive>),
        Reloading(PlayerState<Reloading>),
    }

    impl From<BombingEndState> for PlayerStateMachine {
        fn from(value: BombingEndState) -> Self {
            match value {
                BombingEndState::Complete(state) => state.into(),
                BombingEndState::Bombing(state) => state.into(),
            }
        }
    }

    impl From<ReloadEndState> for PlayerStateMachine {
        fn from(value: ReloadEndState) -> Self {
            match value {
                ReloadEndState::Complete(state) => state.into(),
                ReloadEndState::Reloading(state) => state.into(),
            }
        }
    }
}
