//! Video routines translated from the supplied ROM; no console emulation.
use crate::RomError;
/// ROM $1088: decrement each nonzero Genesis color channel by one level.
/// Non-color bits are cleared by the original AND instructions.
pub fn fade_out_word(mut color: u16) -> u16 {
    for (keep, mask, step) in [
        (0xeee0, 0x000e, 2),
        (0xee0e, 0x00e0, 0x20),
        (0xe0ee, 0x0e00, 0x200),
    ] {
        let preserved = color & keep;
        let channel = color & mask;
        color = preserved | if channel == 0 { 0 } else { channel - step };
    }
    color
}
/// ROM $10d6: increment channels that differ from target. This routine assumes
/// an ascending fade; it does NOT clamp or move downward when above target.
/// Wrapping and intermediate masks intentionally match the 68000 word arithmetic.
pub fn fade_in_word(mut color: u16, target: u16) -> u16 {
    for (keep, mask, step) in [
        (0xeee0, 0x000e, 2),
        (0xee0e, 0x00e0, 0x20),
        (0xe0ee, 0x0e00, 0x200),
    ] {
        let preserved = color & keep;
        let channel = color & mask;
        color = preserved
            | if channel == target & mask {
                channel
            } else {
                channel.wrapping_add(step)
            };
    }
    color
}
pub fn fade_out(palette: &mut [u16; 64]) {
    for c in palette {
        *c = fade_out_word(*c);
    }
}
pub fn fade_in(palette: &mut [u16; 64], target: &[u16; 64]) {
    for (color, goal) in palette.iter_mut().zip(target) {
        *color = fade_in_word(*color, *goal);
    }
}
/// The ROM's 59 monochrome glyphs are expanded to 4-bit tiles at $39c..$3ca.
/// Returns index 0/1 pixels in row-major order (ASCII $20..$5a).
pub fn boot_font(rom: &[u8]) -> Result<Vec<[u8; 64]>, RomError> {
    // Restrict fixed offsets to the identified game revision's header.
    if rom.get(0x180..0x18e) != Some(b"GM MK-1352 -00".as_slice()) {
        return Err(RomError(
            "boot font offsets require Shadowrun GM MK-1352 -00",
        ));
    }
    let bytes = rom
        .get(0x4ee..0x6c6)
        .ok_or(RomError("truncated boot font"))?;
    Ok(bytes
        .chunks_exact(8)
        .map(|rows| {
            let mut tile = [0; 64];
            for y in 0..8 {
                for x in 0..8 {
                    tile[y * 8 + x] = (rows[y] >> (7 - x)) & 1;
                }
            }
            tile
        })
        .collect())
}
/// Reconstruct the region-error screen using the boot font and text in the ROM.
/// Produces a 320x224 RGB raster, with no emulator or external graphics library.
/// This previews the mismatch screen even on a supported region.
pub fn region_screen(rom: &[u8]) -> Result<Vec<u8>, RomError> {
    let font = boot_font(rom)?;
    let mut rgb = vec![0; 320 * 224 * 3];
    let mut draw = |address: usize, y: usize| -> Result<(), RomError> {
        let x = *rom.get(address).ok_or(RomError("missing boot string"))? as usize;
        let text = rom
            .get(address + 1..)
            .ok_or(RomError("missing boot string"))?;
        let end = text
            .iter()
            .position(|b| *b == 0)
            .ok_or(RomError("unterminated boot string"))?;
        if x + end > 40 || y >= 28 {
            return Err(RomError("boot text outside screen"));
        }
        for (i, c) in text[..end].iter().enumerate() {
            let tile = font
                .get(
                    c.checked_sub(32)
                        .ok_or(RomError("unsupported font character"))?
                        as usize,
                )
                .ok_or(RomError("unsupported font character"))?;
            for row in 0..8 {
                for col in 0..8 {
                    let offset = ((y * 8 + row) * 320 + (x + i) * 8 + col) * 3;
                    rgb[offset..offset + 3].fill(tile[row * 8 + col] * 255);
                }
            }
        }
        Ok(())
    };
    draw(0x484, 8)?;
    let mut y = 8;
    let regions = rom
        .get(0x1f0..0x200)
        .ok_or(RomError("missing region header"))?;
    for (index, region) in regions.iter().enumerate() {
        if *region == b' ' {
            break;
        }
        let address = match region {
            b'J' => 0x4ae,
            b'U' => 0x4bf,
            b'E' => 0x4cd,
            _ => continue,
        };
        if index != 0 && regions.get(index + 1) == Some(&b' ') {
            y += 1;
            draw(0x4a1, y)?;
        }
        y += 1;
        draw(address, y)?;
    }
    draw(0x4a4, y + 1)?;
    Ok(rgb)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fade_out_all_valid_colors_matches_independent_channel_arithmetic() {
        for r in 0u16..8 {
            for g in 0u16..8 {
                for b in 0u16..8 {
                    let input = r * 2 + g * 32 + b * 512;
                    let expected = r.saturating_sub(1) * 2
                        + g.saturating_sub(1) * 32
                        + b.saturating_sub(1) * 512;
                    assert_eq!(fade_out_word(input), expected);
                }
            }
        }
    }
    #[test]
    fn fade_in_reaches_every_target_from_black_in_seven_frames() {
        for target in 0u16..4096 {
            let target = target & 0xeee;
            let mut c = 0;
            for _ in 0..7 {
                c = fade_in_word(c, target);
            }
            assert_eq!(c, target);
            assert_eq!(fade_in_word(c, target), target);
        }
    }
    #[test]
    fn fade_in_preserves_original_non_clamping_behavior() {
        assert_eq!(fade_in_word(0x0004, 0), 0x0006);
        // Overflow of red is removed by the next channel mask.
        assert_eq!(fade_in_word(0x000e, 0), 0);
    }
    #[test]
    fn full_palette_fades_to_black() {
        let mut colors = [0xeee; 64];
        for _ in 0..7 {
            fade_out(&mut colors);
        }
        assert_eq!(colors, [0; 64]);
        fade_in(&mut colors, &[0xeee; 64]);
        assert_eq!(colors, [0x222; 64]);
    }
    #[test]
    fn truncated_or_wrong_revision_font_is_rejected() {
        assert!(boot_font(&[]).is_err());
        assert!(region_screen(&vec![0; 0x700]).is_err());
    }
}
