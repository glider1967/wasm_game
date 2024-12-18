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
                    None,
                    None,
                ),
                Bullet::new(
                    Point { x: 300.0, y: 50.0 },
                    Point { x: 0.0, y: 4.0 },
                    None,
                    Some(vec![(60, BulletEvent::SetAcc(Point { x: 0.05, y: 0.02 }))]),
                ),
                Bullet::new(
                    Point { x: 400.0, y: 300.0 },
                    Point { x: -4.0, y: 4.0 },
                    None,
                    None,
                ),
                Bullet::new(
                    Point { x: 400.0, y: 300.0 },
                    Point { x: -4.0, y: -4.0 },
                    None,
                    None,
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
    frame: u16,                              // 弾が生成されてからの経過フレーム
    pos: Point,                              // 位置
    vel: Point,                              // 速度
    acc: Option<Point>,                      // 加速度
    events: Option<Vec<(u16, BulletEvent)>>, // 弾に起こる変化の列（タイミング、イベント）
    next_event: usize,                       // 次に起こるイベント番号
    next_event_at: Option<u16>,              // 次に起こるイベントのフレーム
}

impl Bullet {
    pub fn new(
        pos: Point,
        vel: Point,
        acc: Option<Point>,
        events: Option<Vec<(u16, BulletEvent)>>,
    ) -> Self {
        let next_event_at = match &events {
            Some(events) => Some(events[0].0),
            None => None,
        };

        Self {
            frame: 0,
            pos,
            vel,
            acc,
            events,
            next_event: 0,
            next_event_at,
        }
    }

    pub fn update(&mut self) {
        self.frame += 1;

        if let Some(acc) = self.acc {
            self.vel.x += acc.x;
            self.vel.y += acc.y;
        };

        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;

        if Some(self.frame) == self.next_event_at {
            let event = &self.events.as_ref().unwrap()[self.next_event].1;

            match event {
                BulletEvent::RotateVel(_) => {}
                BulletEvent::SetVel(vel) => {
                    self.vel = vel.clone();
                }
                BulletEvent::SetAcc(acc) => {
                    self.acc = Some(acc.clone());
                }
            }

            self.next_event += 1;
            let next_event = self.events.as_ref().unwrap().get(self.next_event);
            self.next_event_at = next_event.map(|event| event.0);
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

#[derive(Clone)]
pub enum BulletEvent {
    RotateVel(f32),
    SetVel(Point),
    SetAcc(Point),
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
                None,
                None,
            ));
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        renderer.set_color("pink");
        renderer.draw_circle(&self.pos, 20.0);
    }
}
