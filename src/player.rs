use crate::{
    engine::{KeyState, Point, Rect, Renderer},
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
        if self.state_machine.context().is_shielded {
            renderer.set_color("blue");
        } else {
            renderer.set_color("red");
        }
        let center = &Point {
            x: self.state_machine.context().position.x,
            y: self.state_machine.context().position.y,
        };
        renderer.draw_rect(&Rect {
            x: center.x - 10.0,
            y: center.y + 10.0,
            width: 20.0,
            height: -20.0 - self.state_machine.context().frame as f32,
        });

        renderer.draw_circle(center, 3.0);
    }

    pub fn update(&mut self, vx: f32, vy: f32) {
        self.state_machine = self.state_machine.update().set_velocity(vx, vy);
    }

    pub fn bomb(&mut self) {
        self.state_machine = self.state_machine.transition(PlayerEvent::Bomb);
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
    Alive(PlayerState<Alive>),
    Bombing(PlayerState<Bombing>),
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

pub enum PlayerEvent {
    Bomb,
    Update,
    Move(f32, f32),
}

impl PlayerStateMachine {
    fn transition(self, event: PlayerEvent) -> Self {
        match (self, event) {
            (PlayerStateMachine::Alive(state), PlayerEvent::Move(vx, vy)) => {
                state.set_velocity(vx, vy).into()
            }
            (PlayerStateMachine::Bombing(state), PlayerEvent::Move(vx, vy)) => {
                state.set_velocity(vx, vy).into()
            }
            (PlayerStateMachine::Alive(state), PlayerEvent::Bomb) => state.bomb().into(),
            (PlayerStateMachine::Alive(state), PlayerEvent::Update) => state.update().into(),
            (PlayerStateMachine::Bombing(state), PlayerEvent::Update) => state.update().into(),
            _ => self,
        }
    }

    fn context(&self) -> &PlayerContext {
        match self {
            PlayerStateMachine::Alive(state) => &state.context(),
            PlayerStateMachine::Bombing(state) => &state.context(),
        }
    }

    fn update(self) -> Self {
        self.transition(PlayerEvent::Update)
    }

    fn set_velocity(self, vx: f32, vy: f32) -> Self {
        self.transition(PlayerEvent::Move(vx, vy))
    }
}

mod player_states {
    use std::marker::PhantomData;

    use crate::engine::Point;

    use super::PlayerStateMachine;
    const FLOOR: f32 = 475.0;

    #[derive(Clone, Copy)]
    pub struct PlayerState<S> {
        context: PlayerContext,
        _state: PhantomData<S>,
    }

    #[derive(Clone, Copy)]
    pub struct PlayerContext {
        pub frame: u8,
        pub position: Point,
        pub velocity: Point,
        pub is_shielded: bool,
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

        fn set_shield(mut self, shield: bool) -> Self {
            self.is_shielded = shield;
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
                    is_shielded: false,
                },
                _state: PhantomData,
            }
        }

        pub fn bomb(self) -> PlayerState<Bombing> {
            PlayerState {
                context: self.context.reset_frame().set_shield(true),
                _state: PhantomData,
            }
        }

        pub fn update(mut self) -> Self {
            self.context = self.context.update(29);
            self
        }
    }

    impl PlayerState<Bombing> {
        pub fn update(mut self) -> BombingEndState {
            self.context = self.context.update(60);

            if self.context.frame >= 60 {
                BombingEndState::Complete(self.end_bomb())
            } else {
                BombingEndState::Sliding(self)
            }
        }

        pub fn end_bomb(self) -> PlayerState<Alive> {
            PlayerState {
                context: self.context.reset_frame().set_shield(false),
                _state: PhantomData,
            }
        }
    }

    pub enum BombingEndState {
        Complete(PlayerState<Alive>),
        Sliding(PlayerState<Bombing>),
    }

    impl From<BombingEndState> for PlayerStateMachine {
        fn from(value: BombingEndState) -> Self {
            match value {
                BombingEndState::Complete(state) => state.into(),
                BombingEndState::Sliding(state) => state.into(),
            }
        }
    }
}
