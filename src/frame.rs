//! Nonblocking equivalents of the wait routines at $bd2 and $bb6.
//! A host scheduler must drive the VBlank count; these do not run game updates.
#[derive(Debug, Default)]
pub struct FrameSync {
    /// $fff9ec, incremented by the level-6 interrupt at $1872.
    pub vblank_count: u32,
    /// $fff9f4. The ordinary wait aliases its low word at $fff9f6.
    pub previous_sample: u32,
    /// $ffdf88, incremented once on entering $bd2, not on each spin.
    pub wait_entries: u16,
}
impl FrameSync {
    pub fn vblank(&mut self) {
        self.vblank_count = self.vblank_count.wrapping_add(1);
    }
    pub fn begin_wait(&mut self) {
        self.wait_entries = self.wait_entries.wrapping_add(1);
    }
    /// Poll $bde..$bf0. Calling repeatedly does not increment wait_entries.
    pub fn poll_frame(&mut self) -> bool {
        let current = self.vblank_count as u16;
        if current == self.previous_sample as u16 {
            return false;
        }
        self.previous_sample = (self.previous_sample & 0xffff0000) | u32::from(current);
        true
    }
    /// Poll $bb6..$bd0: compare the full counter with its low bit cleared.
    pub fn poll_even_frame(&mut self) -> bool {
        let current = self.vblank_count & !1;
        if current == self.previous_sample {
            return false;
        }
        self.previous_sample = current;
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_wait_only_consumes_a_changed_sample() {
        let mut f = FrameSync::default();
        f.begin_wait();
        assert!(!f.poll_frame());
        assert!(!f.poll_frame());
        assert_eq!(f.wait_entries, 1);
        f.vblank();
        assert!(f.poll_frame());
        assert!(!f.poll_frame());
    }
    #[test]
    fn word_wrap_and_shared_marker_match_memory_alias() {
        let mut f = FrameSync {
            vblank_count: 0xffff,
            previous_sample: 0x1234ffff,
            wait_entries: 0xffff,
        };
        f.begin_wait();
        assert_eq!(f.wait_entries, 0);
        f.vblank();
        assert!(f.poll_frame());
        assert_eq!(f.previous_sample, 0x12340000);
        assert!(f.poll_even_frame());
        assert_eq!(f.previous_sample, 0x10000);
    }
    #[test]
    fn even_gate_samples_pairs_including_counter_wrap() {
        let mut f = FrameSync::default();
        f.vblank();
        assert!(!f.poll_even_frame());
        f.vblank();
        assert!(f.poll_even_frame());
        f.vblank();
        assert!(!f.poll_even_frame());
        f.vblank_count = u32::MAX;
        f.previous_sample = u32::MAX - 1;
        f.vblank();
        assert!(f.poll_even_frame());
        assert_eq!(f.previous_sample, 0);
    }
}
