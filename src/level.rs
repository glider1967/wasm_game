use std::iter::Peekable;

use crate::{
    engine::{KeyState, Point, Renderer},
    player::Player,
};

pub struct Level {
    player: Player,
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
}

impl Level {
    pub fn new() -> Self {
        Level {
            player: Player::new(),
            enemies: vec![Enemy::new(
                Point { x: 300.0, y: 50.0 },
                Point { x: 0.0, y: 0.0 },
            )],
            bullets: vec![
                Bullet::new(
                    Point { x: 50.0, y: 50.0 },
                    Point { x: 5.0, y: 5.0 },
                    Point { x: 0.0, y: 0.0 },
                    vec![],
                ),
                Bullet::new(
                    Point { x: 300.0, y: 50.0 },
                    Point { x: 0.0, y: 4.0 },
                    Point { x: 0.0, y: 0.0 },
                    vec![
                        BulletEvent {
                            at: 60,
                            event_ty: BulletEventType::SetAcc(Point { x: 0.05, y: 0.02 }),
                        },
                        BulletEvent {
                            at: 80,
                            event_ty: BulletEventType::SetVel(Point { x: -0.3, y: 0.0 }),
                        },
                    ],
                ),
                Bullet::new(
                    Point { x: 400.0, y: 300.0 },
                    Point { x: -4.0, y: 4.0 },
                    Point { x: 0.0, y: 0.0 },
                    vec![],
                ),
                Bullet::new(
                    Point { x: 400.0, y: 300.0 },
                    Point { x: -4.0, y: -4.0 },
                    Point { x: 0.0, y: 0.0 },
                    vec![],
                ),
            ],
        }
    }

    pub fn update(&mut self, keystate: &KeyState) {
        let (vx, vy) = Player::calc_velocity(keystate);
        self.player.update(vx, vy);

        if keystate.is_pressed("KeyJ") {
            self.player.bomb();
        }

        for enemy in self.enemies.iter_mut() {
            enemy.update(&mut self.bullets);
        }

        for bullet in self.bullets.iter_mut() {
            bullet.update();
        }

        // 画面外に飛んで行った弾を消す
        self.bullets.retain(|bullet| bullet.in_canvas());

        // プレイヤーと敵弾の衝突判定
        for bullet in self.bullets.iter() {
            if self.player.is_collided(bullet) {
                self.player.hit();
            }
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        self.player.draw(renderer);
        for enemy in self.enemies.iter() {
            enemy.draw(renderer);
        }
        for bullet in self.bullets.iter() {
            bullet.draw(renderer);
        }
    }
}

#[derive(Clone)]
pub struct Bullet {
    frame: u16,                // 弾が生成されてからの経過フレーム
    pos: Point,                // 位置
    vel: Point,                // 速度
    acc: Point,                // 加速度
    events: Vec<BulletEvent>,  // 弾に起こる変化の列（タイミング、イベント）
    next_event: Option<usize>, // 次に起こるイベント番号
}

impl Bullet {
    pub fn new(pos: Point, vel: Point, acc: Point, events: Vec<BulletEvent>) -> Self {
        Self {
            frame: 0,
            pos,
            vel,
            acc,
            next_event: if events.is_empty() { None } else { Some(0) },
            events,
        }
    }

    pub fn update(&mut self) {
        self.frame += 1;

        self.vel.x += self.acc.x;
        self.vel.y += self.acc.y;

        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;

        if let Some(next_event) = self.next_event {
            let event = &self.events[next_event];

            if event.at != self.frame {
                return;
            }

            match event.event_ty {
                BulletEventType::RotateVel(_) => {}
                BulletEventType::SetVel(vel) => {
                    self.vel = vel.clone();
                }
                BulletEventType::SetAcc(acc) => {
                    self.acc = acc.clone();
                }
            }

            self.next_event = if next_event == self.events.len() - 1 {
                None
            } else {
                Some(next_event + 1)
            };
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        renderer.set_color("black");
        renderer.draw_circle(&self.pos, 10.0);
    }

    pub fn in_canvas(&self) -> bool {
        self.pos.x <= 550.0 && self.pos.x >= 50.0 && self.pos.y <= 570.0 && self.pos.y >= 30.0
    }

    pub fn pos(&self) -> Point {
        self.pos
    }
}

#[derive(Clone, Copy)]
pub enum BulletEventType {
    RotateVel(f32),
    SetVel(Point),
    SetAcc(Point),
}

#[derive(Clone, Copy)]
pub struct BulletEvent {
    at: u16,
    event_ty: BulletEventType,
}

struct Enemy {
    frame: u16, // 敵が生成されてからの経過フレーム
    pos: Point, // 位置
    vel: Point, // 速度
}

impl Enemy {
    pub fn new(pos: Point, vel: Point) -> Self {
        Self { frame: 0, pos, vel }
    }

    pub fn update(&mut self, bullets: &mut Vec<Bullet>) {
        self.frame += 1;

        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;

        if self.frame == 180 {
            bullets.push(Bullet::new(
                Point { x: 100.0, y: 100.0 },
                Point { x: 0.0, y: 0.0 },
                Point { x: 0.0, y: 0.0 },
                vec![],
            ));
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        renderer.set_color("pink");
        renderer.draw_circle(&self.pos, 20.0);
    }
}
