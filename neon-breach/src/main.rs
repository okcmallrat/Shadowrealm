#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use macroquad::prelude::*;
use serde::Deserialize;
const W: f32 = 1100.;
const H: f32 = 760.;
const CY: Color = Color::new(0.15, 0.95, 0.9, 1.);
const PINK: Color = Color::new(1., 0.2, 0.5, 1.);
#[derive(Deserialize)]
struct Contract {
    name: String,
    corp: String,
    boss: String,
    rooms: usize,
    hp: i32,
    story: String,
    reveal: String,
}
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Title,
    Brief,
    Room,
    Boss,
    Debrief,
    Dead,
    Won,
}
struct Bullet {
    p: Vec2,
    v: Vec2,
    enemy: bool,
    payload: bool,
    life: f32,
}
struct Enemy {
    p: Vec2,
    hp: i32,
    timer: f32,
    kind: usize,
}
struct Pickup {
    p: Vec2,
    kind: usize,
}
struct Spark {
    p: Vec2,
    v: Vec2,
    t: f32,
    c: Color,
}
struct Game {
    contracts: Vec<Contract>,
    mode: Mode,
    level: usize,
    room: usize,
    p: Vec2,
    aim: Vec2,
    hp: i32,
    shield: i32,
    weapon: i32,
    energy: i32,
    score: i32,
    high: i32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    pickups: Vec<Pickup>,
    walls: Vec<Rect>,
    ice: Vec<Rect>,
    sparks: Vec<Spark>,
    fire: f32,
    inv: f32,
    dash: f32,
    shot: f32,
    time: f32,
    boss_hp: i32,
    boss_y: f32,
    hunter: Vec2,
    toast: String,
    toast_t: f32,
    paused: bool,
    shake: f32,
}
fn circle_rect(p: Vec2, r: f32, a: Rect) -> bool {
    p.distance(vec2(p.x.clamp(a.x, a.x + a.w), p.y.clamp(a.y, a.y + a.h))) < r
}
fn text(s: &str, x: f32, y: f32, size: f32, c: Color) {
    draw_text(s, x, y, size, c);
}
fn wrap(s: &str, x: f32, mut y: f32, width: f32, size: f32, c: Color) {
    let mut line = String::new();
    for word in s.split_whitespace() {
        let test = format!("{}{} ", line, word);
        if measure_text(&test, None, size as u16, 1.).width > width {
            text(&line, x, y, size, c);
            y += size * 1.5;
            line.clear();
        }
        line.push_str(word);
        line.push(' ');
    }
    text(&line, x, y, size, c);
}
impl Game {
    fn new() -> Self {
        Self {
            contracts: serde_json::from_str(include_str!("../data/contracts.json")).unwrap(),
            mode: Mode::Title,
            level: 0,
            room: 0,
            p: vec2(90., 380.),
            aim: vec2(1., 0.),
            hp: 6,
            shield: 0,
            weapon: 0,
            energy: 0,
            score: 0,
            high: std::fs::read_to_string("neon-breach-score.txt")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            bullets: vec![],
            enemies: vec![],
            pickups: vec![],
            walls: vec![],
            ice: vec![],
            sparks: vec![],
            fire: 0.,
            inv: 0.,
            dash: 0.,
            shot: 0.,
            time: 0.,
            boss_hp: 0,
            boss_y: 380.,
            hunter: vec2(1000., 120.),
            toast: String::new(),
            toast_t: 0.,
            paused: false,
            shake: 0.,
        }
    }
    fn tell(&mut self, s: &str) {
        self.toast = s.into();
        self.toast_t = 3.;
    }
    fn burst(&mut self, p: Vec2, c: Color) {
        self.shake = 0.13;
        for _ in 0..15 {
            let a: f32 = rand::gen_range(0., std::f32::consts::TAU);
            self.sparks.push(Spark {
                p,
                v: vec2(a.cos(), a.sin()) * rand::gen_range(35., 200.),
                t: rand::gen_range(0.2, 0.65),
                c,
            });
        }
    }
    fn room(&mut self) {
        self.mode = Mode::Room;
        self.p = vec2(80., 380.);
        self.bullets.clear();
        self.enemies.clear();
        self.pickups.clear();
        self.walls.clear();
        self.inv = 1.5;
        for i in 0..4 {
            let x = 260. + i as f32 * 185.;
            let y = if (i + self.room) % 2 == 0 { 165. } else { 430. };
            self.walls.push(Rect::new(x, y, 24., 160.));
        }
        for i in 0..(5 + self.level + self.room) {
            let mut pos = vec2(rand::gen_range(400., 990.), rand::gen_range(150., 640.));
            while self.walls.iter().any(|r| circle_rect(pos, 22., *r)) {
                pos = vec2(rand::gen_range(400., 990.), rand::gen_range(150., 640.));
            }
            self.enemies.push(Enemy {
                p: pos,
                hp: if self.level > 2 { 3 } else { 2 },
                timer: 0.5 + i as f32 * 0.2,
                kind: i % 3,
            });
        }
        for i in 0..3 {
            self.pickups.push(Pickup {
                p: vec2(
                    350. + i as f32 * 250.,
                    if (i + self.room) % 2 == 0 { 610. } else { 150. },
                ),
                kind: (i + self.room + self.level) % 4,
            });
        }
        self.tell("CLEAR SECURITY // REACH THE EAST GATE");
    }
    fn boss(&mut self) {
        self.mode = Mode::Boss;
        self.p = vec2(130., 380.);
        self.bullets.clear();
        self.enemies.clear();
        self.walls.clear();
        self.pickups.clear();
        self.ice.clear();
        self.boss_hp = self.contracts[self.level].hp;
        self.hunter = vec2(1000., 130.);
        self.inv = 2.;
        self.shot = 1.;
        for col in 0..5 {
            for row in 0..15 {
                self.ice.push(Rect::new(
                    735. + col as f32 * 26.,
                    155. + row as f32 * 30.,
                    23.,
                    27.,
                ));
            }
        }
        self.tell("SHOOT OR CHEW ICE // CHARGE 12 // FIRE PAYLOAD [X]");
    }
    fn hit(&mut self) {
        if self.inv > 0. {
            return;
        }
        if self.shield > 0 {
            self.shield -= 1;
        } else {
            self.hp -= 1;
        }
        self.inv = 1.2;
        self.burst(self.p, PINK);
        if self.hp <= 0 {
            self.mode = Mode::Dead;
            self.save();
        }
    }
    fn save(&mut self) {
        if self.score > self.high {
            self.high = self.score;
            let _ = std::fs::write("neon-breach-score.txt", self.high.to_string());
        }
    }
    fn shoot(&mut self, p: Vec2, v: Vec2, enemy: bool, payload: bool) {
        self.bullets.push(Bullet {
            p,
            v,
            enemy,
            payload,
            life: 3.,
        });
    }
    fn update(&mut self, dt: f32) {
        self.time += dt;
        if is_key_pressed(KeyCode::Enter) {
            match self.mode {
                Mode::Title | Mode::Dead | Mode::Won => {
                    let high = self.high;
                    *self = Self::new();
                    self.high = high;
                    self.mode = Mode::Brief;
                }
                Mode::Brief => {
                    self.room = 0;
                    self.room();
                }
                Mode::Debrief => {
                    self.level += 1;
                    self.hp = (self.hp + 2).min(6);
                    self.mode = Mode::Brief;
                }
                _ => {}
            }
        }
        if !matches!(self.mode, Mode::Room | Mode::Boss) {
            return;
        }
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            self.paused = !self.paused;
        }
        if self.paused {
            return;
        }
        self.fire -= dt;
        self.inv -= dt;
        self.dash -= dt;
        self.shot -= dt;
        self.toast_t -= dt;
        self.shake -= dt;
        let mut d = vec2(0., 0.);
        if is_key_down(KeyCode::W) {
            d.y -= 1.
        }
        if is_key_down(KeyCode::S) {
            d.y += 1.
        }
        if is_key_down(KeyCode::A) {
            d.x -= 1.
        }
        if is_key_down(KeyCode::D) {
            d.x += 1.
        }
        let (mx, my) = mouse_position();
        let mouse = vec2(mx * W / screen_width(), my * H / screen_height());
        let mut a = vec2(0., 0.);
        for (key, v) in [
            (KeyCode::Up, vec2(0., -1.)),
            (KeyCode::Down, vec2(0., 1.)),
            (KeyCode::Left, vec2(-1., 0.)),
            (KeyCode::Right, vec2(1., 0.)),
        ] {
            if is_key_down(key) {
                a += v;
            }
        }
        if a.length() > 0. {
            self.aim = a.normalize();
        } else if is_mouse_button_down(MouseButton::Left) {
            self.aim = (mouse - self.p).normalize_or_zero();
        }
        let boost = is_key_pressed(KeyCode::LeftShift) && self.dash <= 0.;
        if boost {
            self.dash = 1.8;
            self.inv = self.inv.max(0.23);
        }
        let delta = d.normalize_or_zero() * if boost { 135. } else { 280. * dt };
        for axis in [vec2(delta.x, 0.), vec2(0., delta.y)] {
            let q = self.p + axis;
            if !self.walls.iter().any(|r| circle_rect(q, 12., *r)) {
                self.p = q;
            }
        }
        self.p.x = self.p.x.clamp(28., 1070.);
        self.p.y = self.p.y.clamp(115., 670.);
        if (a.length() > 0.
            || is_mouse_button_down(MouseButton::Left)
            || is_key_down(KeyCode::Space))
            && self.fire <= 0.
        {
            self.fire = if self.weapon >= 1 { 0.105 } else { 0.17 };
            let dirs = if self.weapon >= 2 {
                vec![-0.14_f32, 0., 0.14]
            } else {
                vec![0.]
            };
            for angle in dirs {
                let v = vec2(
                    self.aim.x * angle.cos() - self.aim.y * angle.sin(),
                    self.aim.x * angle.sin() + self.aim.y * angle.cos(),
                );
                self.shoot(self.p + v * 18., v * 680., false, false);
            }
        }
        if self.mode == Mode::Room {
            self.update_room(dt);
        } else {
            self.update_boss(dt);
        }
        let mut hits = 0;
        let mut explosions = vec![];
        let mut ice_hits = vec![];
        let mut boss_hits = 0;
        for b in &mut self.bullets {
            if b.life <= 0. {
                continue;
            }
            let old = b.p;
            b.p += b.v * dt;
            b.life -= dt;
            if b.p.x < 0. || b.p.x > W || b.p.y < 100. || b.p.y > 690. {
                b.life = 0.;
            }
            if self.walls.iter().any(|r| circle_rect(b.p, 5., *r)) {
                b.life = 0.;
            }
            if b.enemy {
                let refuge = self.mode == Mode::Boss && self.p.x > 450. && self.p.x < 565.;
                if !refuge && b.p.distance(self.p) < 16. {
                    hits += 1;
                    b.life = 0.;
                }
            } else if self.mode == Mode::Room {
                for e in self.enemies.iter_mut() {
                    if e.hp > 0 && b.p.distance(e.p) < 21. {
                        e.hp -= 1;
                        b.life = 0.;
                        if e.hp <= 0 {
                            explosions.push(e.p);
                        }
                        break;
                    }
                }
            } else {
                if let Some((i, _)) = self
                    .ice
                    .iter()
                    .enumerate()
                    .find(|(_, r)| circle_rect(b.p, if b.payload { 12. } else { 5. }, **r))
                {
                    ice_hits.push(i);
                    b.life = 0.;
                } else if b.payload
                    && old.x <= 965.
                    && b.p.x >= 945.
                    && (b.p.y - self.boss_y).abs() < 44.
                {
                    boss_hits += 1;
                    b.life = 0.;
                }
            }
        }
        self.bullets.retain(|b| b.life > 0.);
        for _ in 0..hits {
            self.hit();
        }
        for p in explosions {
            self.burst(p, PINK);
            self.score += 100;
        }
        self.enemies.retain(|e| e.hp > 0);
        ice_hits.sort_unstable();
        ice_hits.dedup();
        for i in ice_hits.into_iter().rev() {
            let r = self.ice.remove(i);
            self.energy = (self.energy + 1).min(36);
            self.score += 10;
            self.burst(vec2(r.x, r.y), CY);
        }
        if boss_hits > 0 {
            self.boss_hp -= boss_hits;
            self.score += 750 * boss_hits;
            self.burst(vec2(970., self.boss_y), YELLOW);
            if self.boss_hp <= 0 {
                self.score += 2000;
                self.mode = if self.level == 5 {
                    Mode::Won
                } else {
                    Mode::Debrief
                };
                self.save();
            }
        }
        for s in &mut self.sparks {
            s.p += s.v * dt;
            s.t -= dt;
        }
        self.sparks.retain(|s| s.t > 0.);
    }
    fn update_room(&mut self, dt: f32) {
        let mut shots = vec![];
        let mut contact = false;
        for e in &mut self.enemies {
            let dir = (self.p - e.p).normalize_or_zero();
            let speed = match e.kind {
                0 => 65.,
                1 => 115.,
                _ => 42.,
            } + self.level as f32 * 8.;
            let q = e.p + dir * speed * dt;
            if !self.walls.iter().any(|r| circle_rect(q, 18., *r)) {
                e.p = q;
            } else {
                e.p.y += (self.p.y - e.p.y).signum() * speed * dt;
            }
            e.timer -= dt;
            if e.timer < 0. {
                e.timer = 1.9 - self.level as f32 * 0.13;
                shots.push((e.p, dir * (230. + self.level as f32 * 18.)));
            }
            if e.p.distance(self.p) < 29. {
                contact = true;
            }
        }
        for (p, v) in shots {
            self.shoot(p, v, true, false);
        }
        if contact {
            self.hit();
        }
        let mut take = vec![];
        for (i, p) in self.pickups.iter().enumerate() {
            if p.p.distance(self.p) < 27. {
                take.push((i, p.kind));
            }
        }
        for (i, k) in take.into_iter().rev() {
            self.pickups.remove(i);
            match k {
                0 => {
                    self.shield = (self.shield + 2).min(6);
                    self.tell("SHIELD +2 // CARRIES INTO BOSS");
                }
                1 => {
                    self.weapon = (self.weapon + 1).min(2);
                    self.tell("WEAPON UPGRADE // CARRIES INTO BOSS");
                }
                2 => {
                    self.energy = (self.energy + 8).min(36);
                    self.tell("EXPLOIT CACHE +8 // SAVE FOR ICE CORE");
                }
                _ => {
                    self.hp = (self.hp + 2).min(6);
                    self.tell("INTEGRITY RESTORED +2");
                }
            }
            self.score += 75;
        }
        if self.enemies.is_empty() && self.p.x > 1035. && (self.p.y - 380.).abs() < 70. {
            self.room += 1;
            if self.room >= self.contracts[self.level].rooms {
                self.boss();
            } else {
                self.room();
            }
        }
    }
    fn update_boss(&mut self, dt: f32) {
        self.boss_y = 380. + (self.time * (0.7 + self.level as f32 * 0.12)).sin() * 190.;
        let rage = self.boss_hp <= self.contracts[self.level].hp / 2;
        self.hunter += (self.p - self.hunter).normalize_or_zero()
            * (90. + self.level as f32 * 12. + if rage { 35. } else { 0. })
            * dt;
        if self.hunter.distance(self.p) < 27. || self.p.distance(vec2(972., self.boss_y)) < 58. {
            self.hit();
        }
        if self.shot <= 0. {
            self.shot = if rage { 0.68 } else { 1.05 };
            let dir = (self.p - vec2(965., self.boss_y)).normalize_or_zero();
            let spread = if self.level >= 3 { 2 } else { 1 };
            for i in -spread..=spread {
                let angle = i as f32 * (0.13 + self.level as f32 * 0.025);
                let v = vec2(
                    dir.x * angle.cos() - dir.y * angle.sin(),
                    dir.x * angle.sin() + dir.y * angle.cos(),
                );
                self.shoot(
                    vec2(950., self.boss_y),
                    v * (250. + self.level as f32 * 22.),
                    true,
                    false,
                );
            }
        }
        if self.level >= 1
            && self.shot > 0.6
            && self.shot - dt <= 0.6
            && (self.level % 2 == 0 || rage)
        {
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8. + self.time;
                self.shoot(
                    vec2(965., self.boss_y),
                    vec2(a.cos(), a.sin()) * 190.,
                    true,
                    false,
                );
            }
        }
        if is_key_pressed(KeyCode::X) || is_mouse_button_pressed(MouseButton::Right) {
            if self.energy >= 12 {
                self.energy -= 12;
                self.shoot(vec2(42., self.p.y), vec2(820., 0.), false, true);
                self.tell("BACKDOOR CANNON // PAYLOAD DEPLOYED");
            } else {
                self.tell("NEED 12 ENERGY // SHOOT OR CHEW ICE");
            }
        }
        if let Some(i) = self.ice.iter().position(|r| circle_rect(self.p, 22., *r)) {
            let r = self.ice.remove(i);
            self.energy = (self.energy + 2).min(36);
            self.score += 15;
            self.burst(vec2(r.x, r.y), CY);
        }
        if self.ice.is_empty() {
            for row in 0..15 {
                self.ice
                    .push(Rect::new(810., 155. + row as f32 * 30., 23., 27.));
            }
        }
    }
    fn draw(&self) {
        clear_background(Color::from_rgba(5, 9, 22, 255));
        for x in (0..1100).step_by(40) {
            draw_line(
                x as f32,
                100.,
                x as f32,
                H,
                1.,
                Color::from_rgba(15, 30, 45, 255),
            );
        }
        for y in (100..760).step_by(40) {
            draw_line(
                0.,
                y as f32,
                W,
                y as f32,
                1.,
                Color::from_rgba(15, 30, 45, 255),
            );
        }
        if matches!(
            self.mode,
            Mode::Title | Mode::Brief | Mode::Debrief | Mode::Dead | Mode::Won
        ) {
            self.draw_card();
            return;
        }
        draw_rectangle(0., 0., W, 98., Color::from_rgba(9, 15, 31, 255));
        text("NEON / BREACH", 28., 35., 28., CY);
        text(&self.contracts[self.level].corp, 28., 65., 18., GRAY);
        text(
            &format!(
                "INTEGRITY {}   SHIELD {}   GUN {}",
                self.hp,
                self.shield,
                self.weapon + 1
            ),
            330.,
            35.,
            22.,
            WHITE,
        );
        text(
            &format!("EXPLOIT {:02}/36   SCORE {:06}", self.energy, self.score),
            330.,
            65.,
            20.,
            CY,
        );
        text(
            if self.mode == Mode::Boss {
                "ICE CORE"
            } else {
                "MATRIX RUN"
            },
            860.,
            35.,
            22.,
            PINK,
        );
        text(
            &format!(
                "{}/{}  //  ROOM {}/{}",
                self.level + 1,
                self.contracts.len(),
                (self.room + 1).min(self.contracts[self.level].rooms),
                self.contracts[self.level].rooms
            ),
            850.,
            65.,
            17.,
            GRAY,
        );
        draw_rectangle_lines(18., 105., 1064., 580., 2., CY);
        for r in &self.walls {
            draw_rectangle(
                r.x - 3.,
                r.y - 3.,
                r.w + 6.,
                r.h + 6.,
                Color::from_rgba(20, 70, 90, 255),
            );
            draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(15, 32, 48, 255));
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 1., CY);
        }
        if self.mode == Mode::Room {
            let open = self.enemies.is_empty();
            draw_rectangle(
                1048.,
                315.,
                34.,
                130.,
                if open {
                    CY
                } else {
                    Color::from_rgba(100, 30, 60, 255)
                },
            );
            text(
                if open { "EXIT >" } else { "LOCKED" },
                955.,
                305.,
                18.,
                if open { CY } else { PINK },
            );
            text(
                &format!("{} SECURITY PROGRAMS", self.enemies.len()),
                35.,
                657.,
                17.,
                GRAY,
            );
        } else {
            draw_rectangle(450., 107., 115., 576., Color::from_rgba(20, 35, 60, 255));
            for i in 0..75 {
                let x = 450. + ((i * 43) as f32 + self.time * 28.) % 115.;
                let y = 110. + ((i * 71) as f32 + self.time * 75.) % 570.;
                draw_rectangle(x, y, 3., 7., Color::from_rgba(40, 100, 120, 255));
            }
            text("REFUGE", 469., 135., 17., CY);
            for r in &self.ice {
                draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(60, 30, 80, 255));
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 1., PINK);
                draw_line(r.x + 4., r.y + 7., r.x + 17., r.y + 20., 1., PINK);
            }
            let c = if self.boss_hp <= self.contracts[self.level].hp / 2 {
                RED
            } else {
                PINK
            };
            draw_poly(972., self.boss_y, 6, 44., self.time * 45., c);
            draw_poly(972., self.boss_y, 6, 27., -self.time * 60., BLACK);
            draw_circle(972., self.boss_y, 11., YELLOW);
            text(&self.contracts[self.level].boss, 890., 124., 24., PINK);
            for i in 0..self.boss_hp {
                draw_rectangle(888. + i as f32 * 24., 135., 18., 8., PINK);
            }
            draw_poly(
                self.hunter.x,
                self.hunter.y,
                4,
                20.,
                self.time * 170.,
                ORANGE,
            );
            draw_circle(self.hunter.x, self.hunter.y, 7., BLACK);
            text(
                "HUNTER",
                self.hunter.x - 28.,
                self.hunter.y - 28.,
                14.,
                ORANGE,
            );
            text(
                if self.energy >= 12 {
                    "PAYLOAD READY [X / RMB]"
                } else {
                    "CHEW ICE TO CHARGE PAYLOAD"
                },
                35.,
                657.,
                19.,
                if self.energy >= 12 { YELLOW } else { CY },
            );
        }
        for p in &self.pickups {
            let (c, label) = match p.kind {
                0 => (SKYBLUE, "S"),
                1 => (YELLOW, "W"),
                2 => (PINK, "E"),
                _ => (GREEN, "+"),
            };
            draw_circle_lines(p.p.x, p.p.y, 19. + (self.time * 4.).sin() * 2., 2., c);
            text(label, p.p.x - 7., p.p.y + 7., 23., c);
        }
        for e in &self.enemies {
            let c = match e.kind {
                0 => PINK,
                1 => ORANGE,
                _ => PURPLE,
            };
            draw_rectangle(e.p.x - 14., e.p.y - 14., 28., 28., c);
            draw_rectangle(e.p.x - 10., e.p.y - 7., 20., 8., BLACK);
            draw_circle(e.p.x - 5., e.p.y - 3., 2., WHITE);
            draw_circle(e.p.x + 5., e.p.y - 3., 2., WHITE);
            draw_line(e.p.x - 11., e.p.y + 14., e.p.x - 17., e.p.y + 21., 3., c);
            draw_line(e.p.x + 11., e.p.y + 14., e.p.x + 17., e.p.y + 21., 3., c);
        }
        for b in &self.bullets {
            let c = if b.enemy {
                PINK
            } else if b.payload {
                YELLOW
            } else {
                CY
            };
            let r = if b.payload { 10. } else { 4. };
            draw_circle(b.p.x, b.p.y, r + 4., Color::new(c.r, c.g, c.b, 0.18));
            draw_circle(b.p.x, b.p.y, r, c);
        }
        if self.inv <= 0. || (self.time * 18.) as i32 % 2 == 0 {
            draw_triangle(
                self.p + self.aim * 19.,
                self.p - self.aim * 13. + vec2(-self.aim.y, self.aim.x) * 11.,
                self.p - self.aim * 13. - vec2(-self.aim.y, self.aim.x) * 11.,
                CY,
            );
            draw_circle(self.p.x, self.p.y, 4., WHITE);
            if self.shield > 0 {
                draw_circle_lines(self.p.x, self.p.y, 23., 1.5, SKYBLUE);
            }
        }
        for s in &self.sparks {
            draw_rectangle(
                s.p.x,
                s.p.y,
                3.,
                3.,
                Color::new(s.c.r, s.c.g, s.c.b, (s.t * 2.).min(1.)),
            );
        }
        draw_rectangle(0., 690., W, 70., Color::from_rgba(9, 15, 31, 255));
        text(
            "WASD MOVE   ARROWS / MOUSE FIRE   SHIFT DASH   X PAYLOAD   P PAUSE",
            28.,
            741.,
            17.,
            GRAY,
        );
        if self.toast_t > 0. {
            text(&self.toast, 28., 716., 19., YELLOW);
        }
        if self.paused {
            draw_rectangle(0., 98., W, 592., Color::new(0., 0., 0., 0.8));
            text("CONNECTION SUSPENDED", 320., 365., 32., CY);
            text("P / ESC TO RESUME", 419., 410., 22., WHITE);
        }
    }
    fn draw_card(&self) {
        text("WISP // ENCRYPTED CONNECTION", 90., 110., 20., CY);
        draw_line(90., 130., 1010., 130., 2., CY);
        let(title,body,button)=match self.mode {Mode::Title=>("NEON / BREACH","You are a Decker in a city owned by HELIX. Run the Matrix, steal what they hide, and burn through the ICE guarding each corporate core.","ENTER // JACK IN"),Mode::Brief=>(self.contracts[self.level].name.as_str(),self.contracts[self.level].story.as_str(),"ENTER // ACCEPT CONTRACT"),Mode::Debrief=>("CONTRACT COMPLETE",self.contracts[self.level].reveal.as_str(),"ENTER // NEXT CONTRACT"),Mode::Won=>("THE CITY IS AWAKE",self.contracts[self.level].reveal.as_str(),"ENTER // NEW RUN"),_=>("CONNECTION LOST","Your deck is fried. The city keeps its secrets tonight. Start a fresh run and bring more power-ups to the ICE fight.","ENTER // RETRY CAMPAIGN")};
        text(
            title,
            90.,
            214.,
            if title.len() > 25 { 37. } else { 54. },
            WHITE,
        );
        wrap(
            body,
            90.,
            275.,
            890.,
            25.,
            Color::from_rgba(180, 200, 215, 255),
        );
        if self.mode == Mode::Title {
            text("01  RUN THE MATRIX", 90., 407., 24., CY);
            wrap("Fast arena combat. Dodge security programs, collect upgrades, and clear each east exit.",90.,443.,430.,20.,GRAY);
            text("02  BREAK THE ICE", 590., 407., 24., PINK);
            wrap("Shoot or chew a gap in the wall. Spend 12 energy to fire a horizontal payload from the left edge at your height. Hit the moving core through the gap.",590.,443.,420.,20.,GRAY);
            text(
                "6 CONTRACTS  /  6 BOSSES  /  ONE MEGACORP",
                90.,
                565.,
                20.,
                WHITE,
            );
        } else {
            text(
                &format!("SCORE {:06}     BEST {:06}", self.score, self.high),
                90.,
                450.,
                24.,
                CY,
            );
            if self.mode == Mode::Brief {
                wrap("Collect S shields, W weapons, E exploit caches and + repairs. All upgrades carry into the boss fight. Refuge blocks bullets; it cannot stop the hunter.",90.,510.,880.,22.,GRAY);
            }
        }
        text(button, 90., 645., 28., YELLOW);
        text(
            "WASD MOVE  /  ARROWS OR MOUSE AIM + FIRE  /  SHIFT DASH",
            90.,
            699.,
            18.,
            GRAY,
        );
        text(
            "SPACE FIRE  /  X OR RIGHT MOUSE PAYLOAD  /  P PAUSE",
            90.,
            727.,
            18.,
            GRAY,
        );
    }
}
fn config() -> Conf {
    Conf {
        window_title: "NEON / BREACH".into(),
        window_width: 1100,
        window_height: 760,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}
async fn verify_game() {
    std::fs::create_dir_all("verification").unwrap();
    let mut g = Game::new();
    g.draw();
    next_frame().await;
    g.draw();
    for level in 0..6 {
        g.level = level;
        g.hp = 6;
        g.shield = 4;
        g.weapon = 2;
        g.energy = 24;
        for room in 0..g.contracts[level].rooms {
            g.room = room;
            g.room();
            assert!(g.enemies.len() >= 5);
            assert_eq!(g.pickups.len(), 3);
            // Live simulation checks enemy movement and projectiles under a real graphics context.
            for _ in 0..30 {
                g.update(1. / 60.);
                g.draw();
                next_frame().await;
            }
            if level == 0 && room == 0 {
                g.draw();
            }
            g.enemies.clear();
            g.p = vec2(1060., 380.);
            g.update(1. / 60.);
            assert!(matches!(g.mode, Mode::Room | Mode::Boss));
        }
        assert!(g.mode == Mode::Boss);
        assert_eq!(g.weapon, 2);
        assert_eq!(g.shield, 4);
        g.draw();
        next_frame().await;
        if level == 0 {
            g.draw();
        }
        // Normal weapons cannot hurt a core; payloads are stopped by intact ICE.
        let hp = g.boss_hp;
        g.shoot(vec2(940., g.boss_y), vec2(820., 0.), false, false);
        g.update(1. / 60.);
        assert_eq!(g.boss_hp, hp);
        let n = g.ice.len();
        g.shoot(vec2(720., 170.), vec2(820., 0.), false, true);
        g.update(1. / 30.);
        assert_eq!(g.boss_hp, hp);
        assert!(g.ice.len() < n);
        // Open a corridor, then verify repeated payload hits defeat each boss.
        g.ice.clear();
        // Keep a remote tile so the depleted-wall regeneration does not close the test corridor.
        g.ice.push(Rect::new(810., 155., 23., 27.));
        for _ in 0..hp {
            g.shoot(vec2(940., g.boss_y), vec2(820., 0.), false, true);
            g.update(1. / 60.);
        }
        assert!(if level == 5 {
            g.mode == Mode::Won
        } else {
            g.mode == Mode::Debrief
        });
    }
    g.draw();
    next_frame().await;
    g.draw();
    println!("VERIFIED: 23 Matrix rooms, 6 boss transitions, power-up persistence, ICE blocking, core weapon rules, campaign ending.");
}
#[macroquad::main(config)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    if std::env::args().any(|a| a == "--verify") {
        verify_game().await;
        return;
    }
    let mut g = Game::new();
    let preview = std::env::args().find(|a| a.starts_with("--preview="));
    if let Some(ref p) = preview {
        if p.ends_with("room") {
            g.room();
        } else if p.ends_with("boss") {
            g.shield = 4;
            g.weapon = 2;
            g.energy = 24;
            g.boss();
        }
    }
    let mut frames = 0;
    loop {
        let dt = get_frame_time().min(0.033);
        if preview.is_none() {
            g.update(dt);
        }
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., W, H));
        camera.zoom.y = 2. / H;
        if g.shake > 0. && !g.paused {
            camera.target += vec2(rand::gen_range(-3., 3.), rand::gen_range(-3., 3.));
        }
        set_camera(&camera);
        g.draw();
        set_default_camera();
        frames += 1;
        if let Some(ref p) = preview {
            if frames == 45 {
                let name = p.trim_start_matches("--preview=");
                get_screen_data().export_png(&format!("verification/{}.png", name));
                return;
            }
        }
        next_frame().await;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn campaign_has_six_escalating_contracts() {
        let g = Game::new();
        assert_eq!(g.contracts.len(), 6);
        assert_eq!(g.contracts[5].boss, "CROWN");
        for c in g.contracts {
            assert!(c.rooms >= 3);
            assert!(c.hp >= 3);
        }
    }
    #[test]
    fn circle_collision() {
        assert!(circle_rect(
            vec2(9., 15.),
            2.,
            Rect::new(10., 10., 10., 10.)
        ));
        assert!(!circle_rect(
            vec2(2., 2.),
            2.,
            Rect::new(10., 10., 10., 10.)
        ));
    }
    #[test]
    fn powerups_survive_transition() {
        let mut g = Game::new();
        g.shield = 4;
        g.weapon = 2;
        g.energy = 24;
        g.boss();
        assert_eq!((g.shield, g.weapon, g.energy), (4, 2, 24));
        assert_eq!(g.ice.len(), 75);
        assert_eq!(g.boss_hp, 3);
    }
}
