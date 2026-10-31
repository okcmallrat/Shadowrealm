//! Native input state matching ROM $195c..$19f0. A frontend supplies held buttons;
//! no Genesis port polling or secondary-CPU synchronization is needed on a PC.
pub const UP: u8 = 1;
pub const DOWN: u8 = 2;
pub const LEFT: u8 = 4;
pub const RIGHT: u8 = 8;
pub const B: u8 = 16;
pub const C: u8 = 32;
pub const A: u8 = 64;
pub const START: u8 = 128;
/// ROM $19d2, indexed by the four direction bits. $ffff means no valid heading.
pub const DIRECTIONS: [u16; 16] = [
    0xffff, 0, 0x8000, 0xffff, 0xc000, 0xe000, 0xa000, 0xffff, 0x4000, 0x2000, 0x6000, 0xffff,
    0xffff, 0xffff, 0xffff, 0xffff,
];
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Controller {
    pub held: u8,
    pub pressed: u8,
}
impl Controller {
    pub fn update(&mut self, held: u8) {
        // Original: new & (old ^ new), followed by LSL.W #8 and OR held.
        self.pressed = held & !self.held;
        self.held = held;
    }
    pub fn packed(self) -> u16 {
        u16::from(self.pressed) << 8 | u16::from(self.held)
    }
    pub fn direction(self) -> Option<u16> {
        let heading = DIRECTIONS[(self.held & 15) as usize];
        if heading == 0xffff {
            None
        } else {
            Some(heading)
        }
    }
}
/// ROM $19f2/$1a48 combines two active-low pad samples (TH low, TH high).
/// Kept for reference/traces; native frontends can supply held bits directly.
pub fn decode_pad_samples(th_low: u8, th_high: u8) -> u8 {
    !((th_low.wrapping_shl(2) & 0xc0) | (th_high & 0x3f))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn held_buttons_only_press_once_and_repress_after_release() {
        let mut c = Controller::default();
        c.update(UP | START);
        assert_eq!(c.packed(), 0x8181);
        assert_eq!(c.direction(), Some(0));
        c.update(UP | START);
        assert_eq!(c.packed(), 0x0081);
        c.update(0);
        assert_eq!(c.pressed, 0);
        c.update(START);
        assert_eq!(c.pressed, START);
        assert_eq!(c.direction(), None);
    }
    #[test]
    fn opposed_directions_are_invalid_and_diagonals_match_rom() {
        let mut c = Controller::default();
        for bits in [UP | DOWN, LEFT | RIGHT, UP | DOWN | LEFT] {
            c.update(bits);
            assert_eq!(c.direction(), None);
        }
        c.update(UP | RIGHT);
        assert_eq!(c.direction(), Some(0x2000));
        c.update(DOWN | LEFT);
        assert_eq!(c.direction(), Some(0xa000));
    }
    #[test]
    fn every_button_combination_round_trips_hardware_samples() {
        for held in 0..=255u8 {
            let active_low = !held;
            let low = (active_low >> 2) & 0x30;
            let high = active_low & 0x3f;
            assert_eq!(decode_pad_samples(low, high), held);
        }
    }
}
