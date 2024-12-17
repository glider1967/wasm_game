use std::cell::RefCell;

use crate::{
    engine::{KeyState, Point, Renderer},
    player::Player,
};

pub struct Level {
    player: Player,
    bullets: RefCell<Vec<Bullet>>,
}

impl Level {
    pub fn new() -> Self {
        Level {
            player: Player::new(),
            bullets: RefCell::new(vec![
                Bullet::new(&Point { x: 50.0, y: 50.0 }, &Point { x: 5.0, y: 5.0 }),
                Bullet::new(&Point { x: 100.0, y: 50.0 }, &Point { x: 0.0, y: 2.0 }),
                Bullet::new(&Point { x: 400.0, y: 300.0 }, &Point { x: -4.0, y: 4.0 }),
                Bullet::new(&Point { x: 400.0, y: 300.0 }, &Point { x: -4.0, y: -4.0 }),
            ]),
        }
    }

    pub fn update(&mut self, keystate: &KeyState) {
        let (vx, vy) = Player::calc_velocity(keystate);
        self.player.update(vx, vy);

        if keystate.is_pressed("KeyJ") {
            self.player.bomb();
        }

        for bullet in self.bullets.borrow_mut().iter_mut() {
            bullet.update();
        }

        // 画面外に飛んで行った弾を消す
        self.bullets
            .borrow_mut()
            .retain(|bullet| bullet.in_canvas());

        // プレイヤーと敵弾の衝突判定
        for bullet in self.bullets.borrow().iter() {
            if self.player.is_collided(bullet) {
                self.player.bomb();
            }
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        self.player.draw(renderer);
        for bullet in self.bullets.borrow().iter() {
            bullet.draw(renderer);
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pos: Point,
    vel: Point,
}

impl Bullet {
    pub fn new(pos: &Point, vel: &Point) -> Self {
        Self {
            pos: pos.clone(),
            vel: vel.clone(),
        }
    }

    pub fn update(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
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
