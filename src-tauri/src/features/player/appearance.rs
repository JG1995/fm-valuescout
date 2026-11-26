//! A newgen's stored appearance: the values FM keeps in the person record and the bridge dump
//! carries as `players[].appearance`. `players.appearance` stores them packed into one integer.

/// The stored appearance of a player FM generated. FM's own conversions (hair length, hair
/// colour) are not applied here; `features::graphics` applies them when it picks a face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerAppearance {
    pub ethnicity: u8,
    pub skin_tone: u8,
    pub hair_length: u8,
    pub hair_colour: u8,
    pub facial_hair: u8,
    pub hair_registry_version: u8,
    /// FM keeps this player's grey hair even while he is young.
    pub keeps_grey_hair: bool,
}

impl PlayerAppearance {
    /// Length of the dump's `appearance` array.
    pub const DUMP_LEN: usize = 7;

    /// Reads the dump's array: ethnicity, skin tone, hair length, hair colour, facial hair,
    /// hair registry version, and the grey-hair flag (0 or 1).
    pub fn from_dump(values: [u8; Self::DUMP_LEN]) -> Option<Self> {
        let [ethnicity, skin_tone, hair_length, hair_colour, facial_hair, hair_registry_version, keeps_grey_hair] =
            values;
        if keeps_grey_hair > 1 {
            return None;
        }
        Some(Self {
            ethnicity,
            skin_tone,
            hair_length,
            hair_colour,
            facial_hair,
            hair_registry_version,
            keeps_grey_hair: keeps_grey_hair == 1,
        })
    }

    /// One byte per value in the order of [`Self::from_dump`], lowest byte first.
    pub fn pack(self) -> i64 {
        i64::from(self.ethnicity)
            | i64::from(self.skin_tone) << 8
            | i64::from(self.hair_length) << 16
            | i64::from(self.hair_colour) << 24
            | i64::from(self.facial_hair) << 32
            | i64::from(self.hair_registry_version) << 40
            | i64::from(self.keeps_grey_hair) << 48
    }

    /// `None` for a value [`Self::pack`] cannot have produced.
    pub fn unpack(packed: i64) -> Option<Self> {
        if !(0..1 << 49).contains(&packed) {
            return None;
        }
        let byte = |shift: u32| (packed >> shift) as u8;
        Some(Self {
            ethnicity: byte(0),
            skin_tone: byte(8),
            hair_length: byte(16),
            hair_colour: byte(24),
            facial_hair: byte(32),
            hair_registry_version: byte(40),
            keeps_grey_hair: packed >> 48 == 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_packed_appearance_unpacks_to_the_same_values() {
        let appearance = PlayerAppearance::from_dump([10, 19, 3, 18, 2, 2, 1]).unwrap();

        assert_eq!(appearance.pack(), 0x0001_0202_1203_130A);
        assert_eq!(
            PlayerAppearance::unpack(appearance.pack()),
            Some(appearance)
        );
    }

    #[test]
    fn values_outside_the_format_are_rejected() {
        assert_eq!(PlayerAppearance::from_dump([0, 0, 0, 0, 0, 0, 2]), None);
        assert_eq!(PlayerAppearance::unpack(-1), None);
        assert_eq!(PlayerAppearance::unpack(1 << 49), None);
    }
}
