use crate::{light_disc, CY};
use macroquad::prelude::*;

pub const DEATH_DURATION: f32 = 1.8;

/// Pose state follows resolved movement, rather than held keys or the global clock.
pub struct Avatar {
    pub facing: Vec2,
    pub motion: Vec2,
    pub phase: f32,
    pub attack: f32,
    pub dash: f32,
    pub death: f32,
}
impl Default for Avatar {
    fn default() -> Self {
        Self {
            facing: vec2(1., 0.),
            motion: Vec2::ZERO,
            phase: 0.,
            attack: 0.,
            dash: 0.,
            death: 0.,
        }
    }
}
pub fn octant(v: Vec2) -> Vec2 {
    let angle =
        (v.y.atan2(v.x) / std::f32::consts::FRAC_PI_4).round() * std::f32::consts::FRAC_PI_4;
    vec2(angle.cos(), angle.sin())
}
impl Avatar {
    pub fn advance(&mut self, displacement: Vec2, aim: Option<Vec2>, dt: f32) {
        self.attack = (self.attack - dt).max(0.);
        self.dash = (self.dash - dt).max(0.);
        self.motion = displacement.normalize_or_zero();
        // Large dash steps never whip the walk cycle around multiple times.
        self.phase += displacement.length().min(9.) * 0.17;
        if let Some(v) = aim.filter(|v| v.length_squared() > 0.001) {
            self.facing = octant(v);
        } else if displacement.length_squared() > 0.001 {
            self.facing = octant(displacement);
        }
    }
    pub fn draw(&self, p: Vec2, time: f32) {
        if self.dash > 0. {
            for i in 1..=4 {
                let q = p - self.motion * i as f32 * 9.;
                draw_ellipse(
                    q.x,
                    q.y - 1.,
                    11.,
                    23.,
                    0.,
                    Color::new(CY.r, CY.g, CY.b, 0.12 / i as f32),
                );
            }
        }
        let front = self.facing.y > -0.35;
        let profile = self.facing.x.abs();
        let width = 9. - profile * 3.;
        let moving = self.motion.length_squared() > 0.01;
        let stride = if moving { self.phase.sin() * 6. } else { 0. };
        let bob = if moving {
            (self.phase * 2.).cos() * 1.2
        } else {
            (time * 2.5).sin() * 0.35
        };
        let p = p + vec2(0., bob);
        let ink = Color::from_rgba(10, 24, 39, 255);
        let suit = Color::from_rgba(198, 225, 230, 255);
        let circuit = if front { CY } else { SKYBLUE };
        draw_ellipse(p.x, p.y + 23., 16., 4., 0., Color::new(0., 0., 0., 0.5));
        // Travel controls feet independently from aim: forward, reverse and strafing cycles.
        for side in [-1., 1.] {
            let hip = p + vec2(side * width * 0.45, 5.);
            let swing = stride * side;
            let knee = p + vec2(side * width * 0.65, 12.) + self.motion * swing * 0.5;
            let foot = p
                + vec2(side * width * 0.8, 21.)
                + self.motion * swing
                + vec2(
                    0.,
                    -(self.phase.cos() * side).max(0.) * if moving { 3. } else { 0. },
                );
            draw_line(hip.x, hip.y, knee.x, knee.y, 7., ink);
            draw_line(knee.x, knee.y, foot.x, foot.y, 6., ink);
            draw_line(hip.x, hip.y, knee.x, knee.y, 4., suit);
            draw_line(knee.x, knee.y, foot.x, foot.y, 3., suit);
            draw_line(hip.x, hip.y, foot.x, foot.y, 1., circuit);
            draw_line(foot.x - 3., foot.y, foot.x + 3., foot.y, 3., circuit);
        }
        let arm = |side: f32| {
            let shoulder = p + vec2(side * (width + 1.), -8.);
            let swing = self.motion * (-stride * side * 0.6);
            let hand = if self.attack > 0. && side == 1. {
                p + self.facing * (23. - self.attack * 22.) + vec2(0., -6.)
            } else {
                p + vec2(side * (width + 4.), 6.) + swing
            };
            draw_line(shoulder.x, shoulder.y, hand.x, hand.y, 6., ink);
            draw_line(shoulder.x, shoulder.y, hand.x, hand.y, 3.5, suit);
            draw_line(shoulder.x, shoulder.y, hand.x, hand.y, 1., circuit);
            if side == 1. {
                light_disc(hand, 5., time * 8., CY);
            }
        };
        // Far arm first, body next, near arm last gives side poses depth.
        arm(-1.);
        draw_rectangle(p.x - width, p.y - 11., width * 2., 19., ink);
        draw_rectangle_lines(p.x - width, p.y - 11., width * 2., 19., 2., suit);
        for side in [-1., 1.] {
            draw_line(
                p.x + side * width * 0.6,
                p.y - 9.,
                p.x + side * width * 0.6,
                p.y + 6.,
                1.5,
                circuit,
            );
        }
        draw_line(p.x - width, p.y + 4., p.x + width, p.y + 4., 2., circuit);
        if front {
            draw_circle_lines(p.x + self.facing.x * 2., p.y - 4., 3., 1.5, WHITE);
        } else {
            light_disc(p + vec2(0., -3.), 5., 0., SKYBLUE);
        }
        arm(1.);
        let head = p + vec2(self.facing.x * 2., -19.);
        draw_ellipse(head.x, head.y, 8. - profile * 1.5, 9., 0., ink);
        draw_ellipse_lines(head.x, head.y, 8. - profile * 1.5, 9., 0., 2., suit);
        if front {
            let visor = head + vec2(self.facing.x * 3., self.facing.y * 2.);
            draw_line(
                visor.x - 5. + profile * 2.,
                visor.y,
                visor.x + 5. - profile * 2.,
                visor.y,
                3.,
                CY,
            );
        } else {
            draw_line(head.x, head.y - 7., head.x, head.y + 5., 2., SKYBLUE);
        }
        draw_line(head.x, head.y - 8., head.x, head.y - 5., 2., CY);
    }
    pub fn draw_death(&self, p: Vec2, time: f32) {
        let t = self.death;
        if t < 0.3 {
            self.draw(p + vec2((t * 130.).sin() * 3., 0.), time);
            draw_circle(p.x, p.y - 4., 27., Color::new(0.6, 1., 1., (0.3 - t) * 1.8));
        }
        // Stable per-cell velocities: fragments follow continuous paths, never random flicker.
        let age = (t - 0.18).max(0.);
        let alpha = (1. - age / (DEATH_DURATION - 0.18)).clamp(0., 1.);
        for row in 0..13 {
            for col in -3_i32..=3 {
                let y = row as f32 * 4. - 27.;
                let x = col as f32 * 4.;
                let silhouette = if row < 4 {
                    col.abs() <= 1
                } else if row < 9 {
                    col.abs() <= 2
                } else {
                    col.abs() == 1 || col.abs() == 2
                };
                if !silhouette {
                    continue;
                }
                let seed = (row * 17 + col * 31) as f32;
                let velocity = vec2(
                    (seed * 1.7).sin() * 70.,
                    -35. - (seed * 0.7).cos().abs() * 65.,
                );
                let q = p + vec2(x, y) + velocity * age + vec2(0., age * age * 48.);
                let c = if (row + col) % 3 == 0 { WHITE } else { CY };
                draw_rectangle(q.x, q.y, 3., 3., Color::new(c.r, c.g, c.b, alpha));
            }
        }
        if t < 0.65 {
            draw_circle_lines(
                p.x,
                p.y - 3.,
                10. + t * 95.,
                2.,
                Color::new(CY.r, CY.g, CY.b, 1. - t / 0.65),
            );
        }
        if t > 0.4 {
            draw_line(
                p.x - 24.,
                p.y + 25.,
                p.x + 24.,
                p.y + 25.,
                1.,
                Color::new(CY.r, CY.g, CY.b, alpha),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eight_distinct_facings_and_idle_retention() {
        let mut a = Avatar::default();
        for i in 0..8 {
            let t = i as f32 * std::f32::consts::FRAC_PI_4;
            let v = vec2(t.cos(), t.sin());
            a.advance(v * 4., None, 0.016);
            assert!(a.facing.distance(v) < 0.001);
            a.advance(Vec2::ZERO, None, 0.016);
            assert!(a.facing.distance(v) < 0.001);
        }
    }
    #[test]
    fn strafe_uses_travel_for_feet_and_aim_for_body() {
        let mut a = Avatar::default();
        a.advance(vec2(-4., 0.), Some(vec2(0., -1.)), 0.016);
        assert!(a.motion.distance(vec2(-1., 0.)) < 0.001);
        assert!(a.facing.distance(vec2(0., -1.)) < 0.001);
        let phase = a.phase;
        a.advance(Vec2::ZERO, None, 0.016);
        assert_eq!(phase, a.phase);
        assert_eq!(a.motion, Vec2::ZERO);
    }
}
