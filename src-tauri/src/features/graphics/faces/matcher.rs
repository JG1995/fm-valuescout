//! Picks the catalog face FM27 shows for a player it generated, with "Use pre-generated images"
//! on. The rule is FM's own, read from a running FM 27.1.1: FM converts the stored hair length
//! and colour, draws a hair style from the player's Unique ID, and takes the nearest catalog
//! face of the same gender and ethnicity.

use crate::features::player::appearance::PlayerAppearance;

use super::bundle::CatalogFace;
use super::tables::{
    HeadHair, FEMALE_HAIR_SIMILARITY, FEMALE_HEAD_HAIR_V1, FEMALE_HEAD_HAIR_V2,
    MALE_FACIAL_HAIR_LENGTHS, MALE_HAIR_SIMILARITY, MALE_HEAD_HAIR,
};

/// FM's key for the head hair draw.
const HAIR_DRAW_KEY: u64 = 0x3834_8A94_ACDA_B1E1;

/// FM's weights for skin tone, hair style, hair colour and facial hair in the face distance.
const SKIN_TONE_WEIGHT: f32 = 3.0;
const HAIR_STYLE_WEIGHT: f32 = 1.0;
const HAIR_COLOUR_WEIGHT: f32 = 1.0;
const FACIAL_HAIR_WEIGHT: f32 = 0.5;

/// Portrait hair colour of each database hair colour 1-18.
const HAIR_COLOURS: [u8; 18] = [4, 5, 2, 6, 0, 3, 1, 1, 5, 5, 2, 2, 6, 6, 0, 0, 3, 3];
/// The database hair colours that are grey.
const GREY_HAIR_COLOURS: [u8; 3] = [6, 17, 18];

const MAX_ETHNICITY: u8 = 10;
const MAX_FACIAL_HAIR: u8 = 3;

/// What the face of one generated player depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FaceSubject {
    pub(crate) uid: u32,
    pub(crate) female: bool,
    /// Age in years on the snapshot's game date.
    pub(crate) age: Option<u32>,
    pub(crate) appearance: PlayerAppearance,
}

/// The six values FM matches against the catalog: gender, ethnicity, skin tone, hair style,
/// hair colour and facial hair. `None` when a stored value is one FM's rule was not read for.
pub(crate) fn face_key(subject: &FaceSubject) -> Option<[u8; 6]> {
    let appearance = subject.appearance;
    // FM parses the Unique ID as a signed 32-bit seed.
    let seed = i32::try_from(subject.uid).ok()?;
    if appearance.ethnicity > MAX_ETHNICITY || appearance.facial_hair > MAX_FACIAL_HAIR {
        return None;
    }
    let head_hair: &[HeadHair] = match (subject.female, appearance.hair_registry_version) {
        (false, 1 | 2) => &MALE_HEAD_HAIR,
        (true, 1) => &FEMALE_HEAD_HAIR_V1,
        (true, 2) => &FEMALE_HEAD_HAIR_V2,
        _ => return None,
    };
    // FM keeps medium and long hair and treats every other stored length as short.
    let hair_length = match appearance.hair_length {
        length @ (2 | 3) => length,
        _ => 1,
    };
    let hair_style = draw_hair_style(head_hair, seed, appearance.ethnicity, hair_length);
    Some([
        u8::from(subject.female),
        appearance.ethnicity,
        appearance.skin_tone,
        hair_style,
        hair_colour(subject)?,
        appearance.facial_hair,
    ])
}

/// The portrait hair colour. A generated player stored as grey shows his ethnicity's usual
/// colour until he is `30 + uid % 15` years old, unless FM marked him to keep the grey.
fn hair_colour(subject: &FaceSubject) -> Option<u8> {
    let appearance = subject.appearance;
    let mut stored = appearance.hair_colour;
    if GREY_HAIR_COLOURS.contains(&stored)
        && !appearance.keeps_grey_hair
        && subject.age? < 30 + subject.uid % 15
    {
        stored = if appearance.ethnicity == 0 { 3 } else { 5 };
    }
    HAIR_COLOURS
        .get(usize::from(stored).checked_sub(1)?)
        .copied()
}

/// FM's `HairRegistry.GetRandomHairIndex`: a weighted draw among the styles that allow the
/// ethnicity and have exactly the hair length, in table order. No such style gives style 0.
fn draw_hair_style(table: &[HeadHair], seed: i32, ethnicity: u8, hair_length: u8) -> u8 {
    let candidates = || {
        table.iter().enumerate().filter(move |(_, hair)| {
            hair.ethnicities >> ethnicity & 1 == 1 && hair.length == hair_length
        })
    };
    // FM sums the chances and scales the roll in single precision, then walks them in double.
    let total: f32 = candidates().map(|(_, hair)| hair.chance).sum();
    let target = f64::from(roll(seed) * total);
    let mut reached = 0.0;
    let mut style = 0;
    for (index, hair) in candidates() {
        style = index;
        reached += f64::from(hair.chance);
        if reached >= target {
            break;
        }
    }
    style as u8
}

/// FM's `DeterministicRNG(seed).GetFloat(key)`: a stateless hash of the seed, in `0.0..1.0`.
fn roll(seed: i32) -> f32 {
    let state = i64::from(seed) as u64;
    let mut hash = 0x9E37_79B9_7F4A_7C15_u64
        .wrapping_add(state >> 2)
        .wrapping_add((state << 6).wrapping_add(HAIR_DRAW_KEY));
    hash ^= state;
    hash ^= hash >> 32;
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    (hash >> 40) as f32 * (1.0 / 16_777_216.0)
}

/// FM's `FindBestPortrait`: the first catalog face of the same gender and ethnicity with the
/// smallest distance. The arithmetic follows FM's single-precision order, so ties fall the same.
pub(crate) fn best_face(key: [u8; 6], faces: &[CatalogFace]) -> Option<usize> {
    let female = key[0] == 1;
    let squared = |difference: i32, weight: f32| difference as f32 * weight * difference as f32;
    let difference = |index: usize, face: &[u8; 6]| i32::from(key[index]) - i32::from(face[index]);
    let mut best: Option<(usize, f32)> = None;
    for (index, face) in faces.iter().enumerate() {
        let face = &face.properties;
        if face[0] != key[0] || face[1] != key[1] {
            continue;
        }
        let facial_hair_length = if female {
            0
        } else {
            match MALE_FACIAL_HAIR_LENGTHS.get(usize::from(face[5])) {
                Some(length) => i32::from(*length),
                None => continue,
            }
        };
        let alike = if female {
            similarity_of(&FEMALE_HAIR_SIMILARITY, key[3], face[3])
        } else {
            similarity_of(&MALE_HAIR_SIMILARITY, key[3], face[3])
        };
        let mut distance = squared(difference(2, face), SKIN_TONE_WEIGHT);
        distance += (1.0 - alike) * HAIR_STYLE_WEIGHT;
        distance += squared(difference(4, face), HAIR_COLOUR_WEIGHT);
        distance += squared(i32::from(key[5]) - facial_hair_length, FACIAL_HAIR_WEIGHT);
        if !best.is_some_and(|(_, smallest)| distance >= smallest) {
            best = Some((index, distance));
        }
    }
    best.map(|(index, _)| index)
}

/// The similarity of two hair styles; 0 when FM's matrix does not cover one of them.
fn similarity_of<const N: usize>(matrix: &[[f32; N]], first: u8, second: u8) -> f32 {
    matrix
        .get(usize::from(first))
        .and_then(|row| row.get(usize::from(second)))
        .copied()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The catalog of the bundle the ground truth was captured with (FM 27.1.1 with the
    /// Photoreal Faces V2 mod): six bytes per face, in catalog order.
    fn catalog() -> Vec<CatalogFace> {
        let bytes = zstd::decode_all(&include_bytes!("fixtures/catalog.bin.zst")[..]).unwrap();
        bytes
            .chunks_exact(6)
            .map(|face| CatalogFace::for_test(face.try_into().unwrap()))
            .collect()
    }

    /// Players FM showed a face for on 2026-11-26, one per line: uid, female, age, the seven
    /// appearance values in dump order, and the index of the catalog face FM had loaded.
    fn ground_truth() -> Vec<(FaceSubject, usize)> {
        include_str!("fixtures/faces.csv")
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| {
                let values: Vec<u32> = line
                    .split(',')
                    .map(|value| value.parse().unwrap())
                    .collect();
                let appearance: [u8; 7] = std::array::from_fn(|index| values[3 + index] as u8);
                let subject = FaceSubject {
                    uid: values[0],
                    female: values[1] == 1,
                    age: Some(values[2]),
                    appearance: PlayerAppearance::from_dump(appearance).unwrap(),
                };
                (subject, values[10] as usize)
            })
            .collect()
    }

    fn subject(appearance: [u8; 7]) -> FaceSubject {
        FaceSubject {
            uid: 2_002_233_856,
            female: false,
            age: Some(16),
            appearance: PlayerAppearance::from_dump(appearance).unwrap(),
        }
    }

    #[test]
    fn every_captured_player_gets_the_face_fm_showed() {
        let catalog = catalog();
        let people = ground_truth();
        assert!(people.len() > 120);

        for (subject, expected) in people {
            let face = face_key(&subject).and_then(|key| best_face(key, &catalog));
            assert_eq!(face, Some(expected), "uid {}", subject.uid);
        }
    }

    #[test]
    fn stored_hair_lengths_other_than_medium_and_long_count_as_short() {
        let short = face_key(&subject([0, 4, 1, 12, 0, 2, 0]));

        assert_eq!(face_key(&subject([0, 4, 4, 12, 0, 2, 0])), short);
        assert_eq!(face_key(&subject([0, 4, 0, 12, 0, 2, 0])), short);
        assert_ne!(face_key(&subject([0, 4, 3, 12, 0, 2, 0])), short);
    }

    #[test]
    fn grey_hair_on_a_young_generated_player_becomes_his_ethnicitys_colour() {
        // uid % 15 is 1, so the grey shows from age 31.
        let young = subject([0, 2, 1, 18, 0, 2, 0]);
        let old = FaceSubject {
            age: Some(31),
            ..young
        };
        let kept = subject([0, 2, 1, 18, 0, 2, 1]);
        let other_ethnicity = subject([3, 2, 1, 17, 0, 2, 0]);
        let unknown_age = FaceSubject { age: None, ..young };

        assert_eq!(face_key(&young).unwrap()[4], 2);
        assert_eq!(face_key(&old).unwrap()[4], 3);
        assert_eq!(face_key(&kept).unwrap()[4], 3);
        assert_eq!(face_key(&other_ethnicity).unwrap()[4], 0);
        assert_eq!(face_key(&unknown_age), None);
    }

    #[test]
    fn values_fm_was_not_read_for_give_no_face() {
        // No hair colour, an ethnicity and a facial hair value FM does not have, an unknown
        // hair registry version, and a Unique ID above FM's signed seed.
        assert_eq!(face_key(&subject([0, 4, 1, 0, 0, 2, 0])), None);
        assert_eq!(face_key(&subject([0, 4, 1, 19, 0, 2, 0])), None);
        assert_eq!(face_key(&subject([11, 4, 1, 12, 0, 2, 0])), None);
        assert_eq!(face_key(&subject([0, 4, 1, 12, 4, 2, 0])), None);
        assert_eq!(face_key(&subject([0, 4, 1, 12, 0, 3, 0])), None);
        let high_uid = FaceSubject {
            uid: u32::MAX,
            ..subject([0, 4, 1, 12, 0, 2, 0])
        };
        assert_eq!(face_key(&high_uid), None);
    }

    #[test]
    fn only_faces_of_the_same_gender_and_ethnicity_are_candidates() {
        let faces = [
            CatalogFace::for_test([1, 0, 4, 1, 2, 1]),
            CatalogFace::for_test([0, 1, 4, 1, 2, 0]),
            CatalogFace::for_test([0, 0, 9, 1, 2, 0]),
            CatalogFace::for_test([0, 0, 5, 1, 2, 0]),
            CatalogFace::for_test([0, 0, 3, 1, 2, 0]),
        ];

        // Skin tones 5 and 3 are equally near; the first one wins.
        assert_eq!(best_face([0, 0, 4, 1, 2, 0], &faces), Some(3));
        assert_eq!(best_face([0, 2, 4, 1, 2, 0], &faces), None);
    }
}
