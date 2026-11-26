//! LZ4 block decoding for Unity bundle blocks. The input is a game or mod file, so every length
//! and offset is checked and a malformed block is an error, never a panic.

/// Decodes `src` into `out` (cleared first) and stops once `out` holds `wanted` bytes or the
/// block ends. `Err` means the block is malformed or would exceed `limit` bytes.
pub(super) fn decode(src: &[u8], out: &mut Vec<u8>, wanted: usize, limit: usize) -> Result<(), ()> {
    out.clear();
    let mut at = 0;
    while at < src.len() && out.len() < wanted {
        let token = src[at];
        at += 1;
        let literals = length(src, &mut at, usize::from(token >> 4))?;
        let end = at
            .checked_add(literals)
            .filter(|end| *end <= src.len())
            .ok_or(())?;
        if out.len() + literals > limit {
            return Err(());
        }
        out.extend_from_slice(&src[at..end]);
        at = end;
        if at == src.len() {
            break;
        }
        let offset = match src.get(at..at + 2) {
            Some(bytes) => usize::from(u16::from_le_bytes([bytes[0], bytes[1]])),
            None => return Err(()),
        };
        at += 2;
        let matched = length(src, &mut at, usize::from(token & 15))? + 4;
        if offset == 0 || offset > out.len() || out.len() + matched > limit {
            return Err(());
        }
        let start = out.len() - offset;
        if offset >= matched {
            out.extend_from_within(start..start + matched);
        } else {
            // The match overlaps its own output, so it repeats the last `offset` bytes.
            for index in 0..matched {
                out.push(out[start + index]);
            }
        }
    }
    Ok(())
}

fn length(src: &[u8], at: &mut usize, nibble: usize) -> Result<usize, ()> {
    let mut total = nibble;
    if nibble == 15 {
        loop {
            let byte = *src.get(*at).ok_or(())?;
            *at += 1;
            total = total.checked_add(usize::from(byte)).ok_or(())?;
            if byte != 255 {
                break;
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded(src: &[u8], wanted: usize, limit: usize) -> Result<Vec<u8>, ()> {
        let mut out = Vec::new();
        decode(src, &mut out, wanted, limit).map(|()| out)
    }

    #[test]
    fn literals_and_an_overlapping_match_decode() {
        // "ab", then a match of 6 at offset 2 ("ababab"), then the literal "c".
        let block = [0x22, b'a', b'b', 0x02, 0x00, 0x10, b'c'];

        assert_eq!(decoded(&block, usize::MAX, 64), Ok(b"ababababc".to_vec()));
    }

    #[test]
    fn decoding_stops_once_the_wanted_bytes_are_there() {
        let block = [0x22, b'a', b'b', 0x02, 0x00, 0x10, b'c'];

        assert_eq!(decoded(&block, 4, 64), Ok(b"abababab".to_vec()));
    }

    #[test]
    fn long_lengths_use_the_extra_bytes() {
        let mut block = vec![0xF0, 5];
        block.extend([b'x'; 20]);

        assert_eq!(decoded(&block, usize::MAX, 64), Ok(vec![b'x'; 20]));
    }

    #[test]
    fn malformed_blocks_are_errors() {
        // Literal run longer than the input.
        assert_eq!(decoded(&[0x50, b'a'], usize::MAX, 64), Err(()));
        // Match offset before the start of the output.
        assert_eq!(decoded(&[0x10, b'a', 0x05, 0x00], usize::MAX, 64), Err(()));
        // Zero offset.
        assert_eq!(decoded(&[0x10, b'a', 0x00, 0x00], usize::MAX, 64), Err(()));
        // Output larger than the limit.
        assert_eq!(
            decoded(&[0x1F, b'a', 0x01, 0x00, 0xFF, 0x00], usize::MAX, 64),
            Err(())
        );
        // Missing match offset.
        assert_eq!(decoded(&[0x10, b'a', 0x01], usize::MAX, 64), Err(()));
    }
}
