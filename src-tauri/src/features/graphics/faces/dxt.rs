//! DXT5 (BC3) decoding to RGBA.

/// Decodes a `width` x `height` DXT5 image whose rows run bottom-up, as Unity stores them,
/// into top-down RGBA. `None` when the dimensions are not multiples of four or `data` is short.
pub(super) fn decode_dxt5_bottom_up(data: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    if width == 0 || height == 0 || width % 4 != 0 || height % 4 != 0 {
        return None;
    }
    let blocks = (width / 4).checked_mul(height / 4)?;
    let data = data.get(..blocks.checked_mul(16)?)?;
    let mut rgba = vec![0; width * height * 4];
    for (index, block) in data.chunks_exact(16).enumerate() {
        let (block_x, block_y) = (index % (width / 4) * 4, index / (width / 4) * 4);
        let alphas = alpha_palette(block[0], block[1]);
        let alpha_bits = u64::from_le_bytes([
            block[2], block[3], block[4], block[5], block[6], block[7], 0, 0,
        ]);
        let colours = colour_palette(
            u16::from_le_bytes([block[8], block[9]]),
            u16::from_le_bytes([block[10], block[11]]),
        );
        let colour_bits = u32::from_le_bytes([block[12], block[13], block[14], block[15]]);
        for pixel in 0..16 {
            let (x, y) = (block_x + pixel % 4, block_y + pixel / 4);
            let colour = colours[(colour_bits >> (2 * pixel)) as usize & 3];
            let alpha = alphas[(alpha_bits >> (3 * pixel)) as usize & 7];
            let at = ((height - 1 - y) * width + x) * 4;
            rgba[at..at + 4].copy_from_slice(&[colour[0], colour[1], colour[2], alpha]);
        }
    }
    Some(rgba)
}

fn alpha_palette(first: u8, second: u8) -> [u8; 8] {
    let (a, b) = (u16::from(first), u16::from(second));
    let mut palette = [first, second, 0, 0, 0, 0, 0, 255];
    let steps = if first > second { 7 } else { 5 };
    for step in 1..steps {
        palette[usize::from(step) + 1] = (((steps - step) * a + step * b) / steps) as u8;
    }
    palette
}

fn colour_palette(first: u16, second: u16) -> [[u8; 3]; 4] {
    let expand = |colour: u16| {
        let (r, g, b) = (colour >> 11, (colour >> 5) & 63, colour & 31);
        [
            ((r * 255 + 15) / 31) as u8,
            ((g * 255 + 31) / 63) as u8,
            ((b * 255 + 15) / 31) as u8,
        ]
    };
    let (a, b) = (expand(first), expand(second));
    let mix =
        |a: u8, b: u8, of_a: u16| ((of_a * u16::from(a) + (3 - of_a) * u16::from(b)) / 3) as u8;
    [
        a,
        b,
        [mix(a[0], b[0], 2), mix(a[1], b[1], 2), mix(a[2], b[2], 2)],
        [mix(a[0], b[0], 1), mix(a[1], b[1], 1), mix(a[2], b[2], 1)],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One block: alpha 255 and 0 with the first pixel opaque and the rest clear; red and blue
    /// with pixel 0 red, pixel 1 blue, pixel 2 two thirds red, pixel 3 one third red.
    fn block() -> [u8; 16] {
        let mut block = [0; 16];
        block[0] = 255;
        block[1] = 0;
        // Alpha indexes: pixel 0 uses 0 (255); every other pixel uses 1 (0).
        let alpha_bits: u64 = (0..16).fold(0, |bits, pixel| {
            bits | (u64::from(pixel != 0) << (3 * pixel))
        });
        block[2..8].copy_from_slice(&alpha_bits.to_le_bytes()[..6]);
        block[8..10].copy_from_slice(&0xF800u16.to_le_bytes());
        block[10..12].copy_from_slice(&0x001Fu16.to_le_bytes());
        block[12] = 0b1110_0100;
        block
    }

    #[test]
    fn a_block_decodes_with_its_rows_flipped() {
        let rgba = decode_dxt5_bottom_up(&block(), 4, 4).unwrap();

        // The block's first row is the image's last row.
        let last_row = &rgba[3 * 16..];
        assert_eq!(&last_row[0..4], &[255, 0, 0, 255]);
        assert_eq!(&last_row[4..8], &[0, 0, 255, 0]);
        assert_eq!(&last_row[8..12], &[170, 0, 85, 0]);
        assert_eq!(&last_row[12..16], &[85, 0, 170, 0]);
        // The other rows use colour index 0 and the clear alpha.
        assert_eq!(&rgba[0..4], &[255, 0, 0, 0]);
    }

    #[test]
    fn short_data_and_odd_sizes_are_rejected() {
        assert_eq!(decode_dxt5_bottom_up(&block()[..15], 4, 4), None);
        assert_eq!(decode_dxt5_bottom_up(&block(), 4, 6), None);
        assert_eq!(decode_dxt5_bottom_up(&block(), 0, 4), None);
    }
}
