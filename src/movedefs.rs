/*
    Moves:
    The effect of a move on permutation is defined using a vector of indices. The index in position n means that piece in position n moves
    to that index. For example, if the move vector for corner permutation is [1, 2, 0, 3, 4, 5], then the corner in the UBL position moves to
    the UBR position, the corner in the UBR postion moves to UF, and the corner in UF moves to UBL. The D corners do not move.

    The effect of a move on orientation is defined as a vector of booleans. Orientation is applied after permutation, and each element of
    the vector indicates whether or not the piece that arrives in that position is flipped relative to it's starting orientation.
*/

// Corners:
//      0    1    2   3   4   5
//      UBL  UBR  UF  DB  DR  DL
// Edges:
//      0   1   2   3    4    5    6    7    8    9   10  11
//      UB  UR  UL  BLB  BLL  BLD  BRR  BRB  BRD  FL  FR  FD
// Up centres:
//      0    1    2   3    4    5     6    7     8    9   10   11
//      UBL  UBR  UF  BLU  BLF  BLBR  BRU  BRBL  BRF  FU  FBR  FBL
// Down Centres:
//      0   1   2   3   4   5   6   7   8   9   10  11
//      BR  BL  BD  RL  RB  RD  LB  LR  LD  DL  DR  DB

use std::fmt;
use std::borrow::Cow;

pub const NUM_FACES: usize = 8;

pub const NUM_CORNERS: usize = 6;
pub const NUM_EDGES: usize = 12;
pub const NUM_CENTRES: usize = 12;




const RAW_TURN_U: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [2,0,1,3,4,5],
    corner_orientation: [0b000000],
    edges: [2,0,1,3,4,5,6,7,8,9,10,11],
    up_centres: [2,0,1,3,4,5,6,7,8,9,10,11],
    down_centres: [6,7,2,0,1,5,3,4,8,9,10,11],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};
const RAW_TURN_F: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,1,5,3,2,4],
    corner_orientation: [0b001001],
    edges: [0,1,2,3,4,5,6,7,8,11,9,10],
    up_centres: [0,1,2,3,4,5,6,7,8,11,9,10],
    down_centres: [0,1,2,8,4,7,6,9,10,5,3,11],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};
const RAW_TURN_BL: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [3,1,2,5,4,0],
    corner_orientation: [0b100100],
    edges: [0,1,2,5,3,4,6,7,8,9,10,11],
    up_centres: [0,1,2,5,3,4,6,7,8,9,10,11],
    down_centres: [0,11,9,3,4,5,2,7,1,6,10,8],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};
const RAW_TURN_BR: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,4,2,1,3,5],
    corner_orientation: [0b010010],
    edges: [0,1,2,3,4,5,8,6,7,9,10,11],
    up_centres: [0,1,2,3,4,5,8,6,7,9,10,11],
    down_centres: [5,1,4,3,10,11,6,7,8,9,2,0],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};

// Down Centres:
//      0   1   2   3   4   5   6   7   8   9   10  11
//      BR  BL  BD  RL  RB  RD  LB  LR  LD  DL  DR  DB
const RAW_TURN_L: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [5,1,0,3,4,2],
    corner_orientation: [0b101000],
    edges: [0,1,4,3,9,5,6,7,8,2,10,11],
    up_centres: [4,1,3,11,9,5,6,7,8,0,10,2],
    down_centres: [0,1,2,3,4,5,8,6,7,9,10,11],
    triple_centres: [0,3,2,9,4,5,6,7,8,1,10,11],
};
const RAW_TURN_R: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,2,4,3,1,5],
    corner_orientation: [0b011000],
    edges: [0,10,2,3,4,5,1,7,8,9,6,11],
    up_centres: [0,9,10,3,4,5,2,7,1,8,6,11],
    down_centres: [0,1,2,5,3,4,6,7,8,9,10,11],
    triple_centres: [10,1,2,3,4,5,6,0,8,9,7,11],
};
const RAW_TURN_B: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [1,3,2,0,4,5],
    corner_orientation: [0b110000],
    edges: [7,1,2,0,4,5,6,3,8,9,10,11],
    up_centres: [6,7,2,1,4,0,5,3,8,9,10,11],
    down_centres: [2,0,1,3,4,5,6,7,8,9,10,11],
    triple_centres: [0,1,2,3,6,5,11,7,8,9,10,4],
};
const RAW_TURN_D: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,1,2,4,5,3],
    corner_orientation: [0b000000],
    edges: [0,1,2,3,4,8,6,7,11,9,10,5],
    up_centres: [0,1,2,3,7,8,6,10,11,9,4,5],
    down_centres: [0,1,2,3,4,5,6,7,8,11,9,10],
    triple_centres: [0,1,8,3,4,2,6,7,5,9,10,11],
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    U,
    F,
    BL,
    BR,
    L,
    R,
    B,
    D,
}

pub enum TurnEffectType {
    Corner,
    CornerPermutation,
    CornerOrientation,
    EdgeInFace,
    EdgeAcrossFaces,
    UpCentre,
    DownCentre,
    TripleCentre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTurn {
    corner_full_state: [u8; 7],
    pub corner_permutation: [u8; 6],
    pub corner_orientation: [u8; 1],
    pub edges: [u8; 12],
    pub up_centres: [u8; 12],
    pub down_centres: [u8; 12],
    pub triple_centres: [u8; 12],
}

#[derive(Clone, Copy)]
pub struct Turn {
    pub face: Face,
    pub invert: bool
}

impl Face {
    pub fn get_up_faces() -> [Self; 4] {
        [Self::U, Self::F, Self::BL, Self::BR]
    }

    pub fn get_down_faces() -> [Self; 4] {
        [Self::L, Self::R, Self::B, Self::D]
    }

    pub fn get_all_faces() -> [Self; 8] {
        [Self::U, Self::F, Self::BL, Self::BR, Self::L, Self::R, Self::B, Self::D]
    }

    pub fn get_raw_turn(self) -> &'static RawTurn {
        RawTurn::get(self)
    }

    pub fn get_primary_face(&self) -> Self {
        match self {
            Self::U => Self::U,
            Self::F => Self::F,
            Self::BL => Self::BL,
            Self::BR => Self::BR,
            Self::L => Self::BR,
            Self::R => Self::BL,
            Self::B => Self::F,
            Self::D => Self::U,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::U => b'U',
            Self::F => b'F',
            Self::BL => b'P',
            Self::BR => b'S',
            Self::L => b'L',
            Self::R => b'R',
            Self::B => b'B',
            Self::D => b'D',
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            b'U' => Self::U,
            b'F' => Self::F,
            b'P' => Self::BL,
            b'S' => Self::BR,
            b'L' => Self::L,
            b'R' => Self::R,
            b'B' => Self::B,
            b'D' => Self::D,
            _ => panic!("Unrecognised byte")
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::U => 0,
            Self::F => 1,
            Self::BL => 2,
            Self::BR => 3,
            Self::L => 4,
            Self::R => 5,
            Self::B => 6,
            Self::D => 7,
        }
    }

    pub fn from_index(value: usize) -> Self {
        match value {
            0 => Self::U,
            1 => Self::F,
            2 => Self::BL,
            3 => Self::BR,
            4 => Self::L,
            5 => Self::R,
            6 => Self::B,
            7 => Self::D,
            _ => panic!("Unrecognised index")
        }
    }
}

impl RawTurn {
    pub fn get(face: Face) -> &'static Self {
        match face {
            Face::U => &RAW_TURN_U,
            Face::F => &RAW_TURN_F,
            Face::BL => &RAW_TURN_BL,
            Face::BR => &RAW_TURN_BR,
            Face::L => &RAW_TURN_L,
            Face::R => &RAW_TURN_R,
            Face::B => &RAW_TURN_B,
            Face::D => &RAW_TURN_D
        }
    }

    fn get_for_faces(faces: &[Face]) -> Vec<&'static Self> {
        let mut result: Vec<&'static Self> = vec![];
        for face in faces {
            result.push(Self::get(*face));
        }
        result
    }

    pub fn get_all() -> Vec<&'static Self> {
        Self::get_for_faces(&Face::get_all_faces())
    }

    pub fn get_for_up_faces() -> Vec<&'static Self> {
        Self::get_for_faces(&Face::get_up_faces())
    }

    pub fn get_for_down_faces() -> Vec<&'static Self> {
        Self::get_for_faces(&Face::get_down_faces())
    }

    fn get_corner_full_state(&self) -> Cow<[u8]> {
        let mut state = Vec::with_capacity(NUM_CORNERS + 1);
        state.extend_from_slice(&self.corner_permutation);
        state.extend_from_slice(&self.corner_permutation);
        Cow::Owned(state)
    }

    pub fn get_effect(&self, effect_type: TurnEffectType) -> Cow<[u8]> {
        match effect_type {
            TurnEffectType::Corner => self.get_corner_full_state(),
            TurnEffectType::CornerPermutation => Cow::Borrowed(&self.corner_permutation),
            TurnEffectType::CornerOrientation => Cow::Borrowed(&self.corner_orientation),
            TurnEffectType::EdgeInFace => Cow::Borrowed(&self.edges),
            TurnEffectType::EdgeAcrossFaces => Cow::Borrowed(&self.edges),
            TurnEffectType::UpCentre => Cow::Borrowed(&self.up_centres),
            TurnEffectType::DownCentre => Cow::Borrowed(&self.down_centres),
            TurnEffectType::TripleCentre => Cow::Borrowed(&self.triple_centres),
        }
    }
}

impl Turn {
    pub fn new(face: Face, invert: bool) -> Self {
        Self {
            face,
            invert,
         }
    }

    pub fn get_allowed_turns_for_faces(faces: &[Face]) -> Vec<Self> {
        let mut turns = Vec::new();
        for face in faces {
            turns.push(Self::new(*face, false));
            turns.push(Self::new(*face, true));
        }
        turns
    }

    pub fn get_all_turns() -> Vec<Self> {
        Self::get_allowed_turns_for_faces(&Face::get_all_faces())
    }

    pub fn get_up_turns() -> Vec<Self> {
        Self::get_allowed_turns_for_faces(&Face::get_up_faces())
    }

    pub fn get_down_turns() -> Vec<Self> {
        Self::get_allowed_turns_for_faces(&Face::get_down_faces())
    }
}

impl fmt::Debug for Turn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let invert_symbol = if self.invert {
            "'"
        } else {
            ""
        };
        write!(f, "{:?}{}", self.face, invert_symbol)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_faces() {
        let turns = Face::get_all_faces();
        let expected = [Face::U, Face::F, Face::BL, Face::BR, Face::L, Face::R, Face::B, Face::D];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_up_faces() {
        let turns = Face::get_up_faces();
        let expected = [Face::U, Face::F, Face::BL, Face::BR];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_down_faces() {
        let turns = Face::get_down_faces();
        let expected = [Face::L, Face::R, Face::B, Face::D];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_single_turn() {
        assert_eq!(RawTurn::get(Face::U), &RAW_TURN_U);
        assert_eq!(RawTurn::get(Face::F), &RAW_TURN_F);
        assert_eq!(RawTurn::get(Face::R), &RAW_TURN_R);
        assert_eq!(RawTurn::get(Face::L), &RAW_TURN_L);
        assert_eq!(RawTurn::get(Face::B), &RAW_TURN_B);
        assert_eq!(RawTurn::get(Face::D), &RAW_TURN_D);
        assert_eq!(RawTurn::get(Face::BR), &RAW_TURN_BR);
        assert_eq!(RawTurn::get(Face::BL), &RAW_TURN_BL);
    }

    #[test]
    fn test_get_single_turn_for_face() {
        assert_eq!(Face::U.get_raw_turn(), &RAW_TURN_U);
        assert_eq!(Face::F.get_raw_turn(), &RAW_TURN_F);
        assert_eq!(Face::R.get_raw_turn(), &RAW_TURN_R);
        assert_eq!(Face::L.get_raw_turn(), &RAW_TURN_L);
        assert_eq!(Face::B.get_raw_turn(), &RAW_TURN_B);
        assert_eq!(Face::D.get_raw_turn(), &RAW_TURN_D);
        assert_eq!(Face::BR.get_raw_turn(), &RAW_TURN_BR);
        assert_eq!(Face::BL.get_raw_turn(), &RAW_TURN_BL);
    }

    #[test]
    fn test_get_opposing_faces() {
        assert_eq!(Face::U.get_primary_face(), Face::U);
        assert_eq!(Face::D.get_primary_face(), Face::U);
        assert_eq!(Face::F.get_primary_face(), Face::F);
        assert_eq!(Face::B.get_primary_face(), Face::F);
        assert_eq!(Face::BL.get_primary_face(), Face::BL);
        assert_eq!(Face::R.get_primary_face(), Face::BL);
        assert_eq!(Face::BR.get_primary_face(), Face::BR);
        assert_eq!(Face::L.get_primary_face(), Face::BR);
    }

    #[test]
    fn test_get_all_raw_turns() {
        let turns = RawTurn::get_all();
        let expected = vec![&RAW_TURN_U, &RAW_TURN_F, &RAW_TURN_BL, &RAW_TURN_BR, &RAW_TURN_L, &RAW_TURN_R, &RAW_TURN_B, &RAW_TURN_D];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_up_raw_turns() {
        let turns = RawTurn::get_for_up_faces();
        let expected = vec![&RAW_TURN_U, &RAW_TURN_F, &RAW_TURN_BL, &RAW_TURN_BR];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_down_raw_turns() {
        let turns = RawTurn::get_for_down_faces();
        let expected = vec![&RAW_TURN_L, &RAW_TURN_R, &RAW_TURN_B, &RAW_TURN_D];
        assert_eq!(turns, expected);
    }

    fn assert_turn_list_contains_each_face_both_directions(turns: &[Turn], faces: &[Face]) {
        assert_eq!(turns.len(), faces.len() * 2);
        for face in faces {
            let mut seen_flags: u8 = 0b00;
            for turn in turns {
                if turn.face == *face {
                    let mask = 1 << (turn.invert as u8);
                    assert_eq!(seen_flags & mask, 0);
                    seen_flags |= mask;
                }
            }
            assert_eq!(seen_flags, 0b11);
        }
    }

    #[test]
    fn test_debug_turn() {
        assert_eq!(format!("{:?}", Turn::new(Face::U, true)), "U'");
        assert_eq!(format!("{:?}", Turn::new(Face::U, false)), "U");
        assert_eq!(format!("{:?}", Turn::new(Face::B, true)), "B'");
        assert_eq!(format!("{:?}", Turn::new(Face::B, false)), "B");
        assert_eq!(format!("{:?}", Turn::new(Face::BR, true)), "BR'");
        assert_eq!(format!("{:?}", Turn::new(Face::BR, false)), "BR");
    }

    #[test]
    fn test_get_all_turns() {
        let turns = Turn::get_all_turns();
        assert_turn_list_contains_each_face_both_directions(&turns, &Face::get_all_faces())
    }

    #[test]
    fn test_get_up_turns() {
        let turns = Turn::get_up_turns();
        assert_turn_list_contains_each_face_both_directions(&turns, &Face::get_up_faces())
    }

    #[test]
    fn test_face_to_and_from_byte() {
        let mut seen_bytes = Vec::new();
        for face in Face::get_all_faces() {
            let byte = face.to_byte();
            let converted_face = Face::from_byte(byte);

            assert_eq!(face, converted_face);
            for seen in &seen_bytes {
                assert_ne!(*seen, byte);
            }
            seen_bytes.push(byte);
        }
    }

    #[test]
    fn test_get_turn_effect_type() {
        assert_eq!(RAW_TURN_U.get_effect(TurnEffectType::CornerPermutation).as_ref(), &RAW_TURN_U.corner_permutation);
        assert_eq!(RAW_TURN_U.get_effect(TurnEffectType::CornerOrientation).as_ref(), &RAW_TURN_U.corner_orientation);
        assert_eq!(RAW_TURN_U.get_effect(TurnEffectType::EdgeInFace).as_ref(), &RAW_TURN_U.edges);
        assert_eq!(RAW_TURN_U.get_effect(TurnEffectType::EdgeAcrossFaces).as_ref(), &RAW_TURN_U.edges);
        assert_eq!(RAW_TURN_U.get_effect(TurnEffectType::UpCentre).as_ref(), &RAW_TURN_U.up_centres);
        assert_eq!(RAW_TURN_U.get_effect(TurnEffectType::TripleCentre).as_ref(), &RAW_TURN_U.triple_centres);

        assert_eq!(RAW_TURN_D.get_effect(TurnEffectType::CornerPermutation).as_ref(), &RAW_TURN_D.corner_permutation);
        assert_eq!(RAW_TURN_D.get_effect(TurnEffectType::CornerOrientation).as_ref(), &RAW_TURN_D.corner_orientation);
        assert_eq!(RAW_TURN_D.get_effect(TurnEffectType::EdgeInFace).as_ref(), &RAW_TURN_D.edges);
        assert_eq!(RAW_TURN_D.get_effect(TurnEffectType::EdgeAcrossFaces).as_ref(), &RAW_TURN_D.edges);
        assert_eq!(RAW_TURN_D.get_effect(TurnEffectType::UpCentre).as_ref(), &RAW_TURN_D.up_centres);
        assert_eq!(RAW_TURN_D.get_effect(TurnEffectType::TripleCentre).as_ref(), &RAW_TURN_D.triple_centres);
    }
}