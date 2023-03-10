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

use std::borrow::Cow;

pub const NUM_CORNERS: usize = 6;
pub const NUM_EDGES: usize = 12;
pub const NUM_CENTRES: usize = 12;


const TURN_U: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [2,0,1,3,4,5],
    corner_orientation: [0b000000],
    edges: [2,0,1,3,4,5,6,7,8,9,10,11],
    up_centres: [2,0,1,3,4,5,6,7,8,9,10,11],
    down_centres: [6,7,2,0,1,5,3,4,8,9,10,11],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};   
const TURN_F: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,1,5,3,2,4],
    corner_orientation: [0b001001],
    edges: [0,1,2,3,4,5,6,7,8,11,9,10],
    up_centres: [0,1,2,3,4,5,6,7,8,11,9,10],
    down_centres: [0,1,2,8,4,7,6,9,10,5,3,11],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};
const TURN_BL: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [3,1,2,5,4,0],
    corner_orientation: [0b100100],
    edges: [0,1,2,5,3,4,6,7,8,9,10,11],
    up_centres: [0,1,2,5,3,4,6,7,8,9,10,11],
    down_centres: [0,11,9,3,4,5,2,7,1,6,10,8],
    triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
};
const TURN_BR: RawTurn = RawTurn {
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
const TURN_L: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [5,1,0,3,4,2],
    corner_orientation: [0b101000],
    edges: [0,1,4,3,9,5,6,7,8,2,10,11],
    up_centres: [4,1,3,11,9,5,6,7,8,0,10,2],
    down_centres: [0,1,2,3,4,5,8,6,7,9,10,11],
    triple_centres: [0,9,2,1,4,5,6,7,8,3,10,11],
};
const TURN_R: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,2,4,3,1,5],
    corner_orientation: [0b011000],
    edges: [0,10,2,3,4,5,1,7,8,9,6,11],
    up_centres: [0,9,10,3,4,5,2,7,1,8,6,11],
    down_centres: [0,1,2,5,3,4,6,7,8,9,10,11],
    triple_centres: [7,1,2,3,4,5,6,10,8,9,0,11],
};
const TURN_B: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [1,3,2,0,4,5],
    corner_orientation: [0b110000],
    edges: [7,1,2,0,4,5,6,3,8,9,10,11],
    up_centres: [6,7,2,1,4,0,5,3,8,9,10,11],
    down_centres: [2,0,1,3,4,5,6,7,8,9,10,11],
    triple_centres: [0,1,2,3,11,5,4,7,8,9,10,6],
};
const TURN_D: RawTurn = RawTurn {
    corner_full_state: [0; 7],
    corner_permutation: [0,1,2,4,5,3],
    corner_orientation: [0b000000],
    edges: [0,1,2,3,4,8,6,7,11,9,10,5],
    up_centres: [0,1,2,3,7,8,6,10,11,9,4,5],
    down_centres: [0,1,2,3,4,5,6,7,8,11,9,10],
    triple_centres: [0,5,2,3,4,8,6,7,1,9,10,11],
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

    pub fn turn(self) -> &'static RawTurn {
        RawTurn::get(self)
    }
}

impl RawTurn {
    pub fn get(face: Face) -> &'static Self {
        match face {
            Face::U => &TURN_U,
            Face::F => &TURN_F,
            Face::BL => &TURN_BL,
            Face::BR => &TURN_BR,
            Face::L => &TURN_L,
            Face::R => &TURN_R,
            Face::B => &TURN_B,
            Face::D => &TURN_D
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
            TurnEffectType::TripleCentre => Cow::Borrowed(&self.triple_centres),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_single_turn() {
        assert_eq!(RawTurn::get(Face::U), &TURN_U);
        assert_eq!(RawTurn::get(Face::F), &TURN_F);
        assert_eq!(RawTurn::get(Face::R), &TURN_R);
        assert_eq!(RawTurn::get(Face::L), &TURN_L);
        assert_eq!(RawTurn::get(Face::B), &TURN_B);
        assert_eq!(RawTurn::get(Face::D), &TURN_D);
        assert_eq!(RawTurn::get(Face::BR), &TURN_BR);
        assert_eq!(RawTurn::get(Face::BL), &TURN_BL);
    }

    #[test]
    fn test_get_single_turn_for_face() {
        assert_eq!(Face::U.turn(), &TURN_U);
        assert_eq!(Face::F.turn(), &TURN_F);
        assert_eq!(Face::R.turn(), &TURN_R);
        assert_eq!(Face::L.turn(), &TURN_L);
        assert_eq!(Face::B.turn(), &TURN_B);
        assert_eq!(Face::D.turn(), &TURN_D);
        assert_eq!(Face::BR.turn(), &TURN_BR);
        assert_eq!(Face::BL.turn(), &TURN_BL);
    }

    #[test]
    fn test_get_all_turns() {
        let turns = RawTurn::get_all();
        let expected = vec![&TURN_U, &TURN_F, &TURN_BL, &TURN_BR, &TURN_L, &TURN_R, &TURN_B, &TURN_D];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_up_turns() {
        let turns = RawTurn::get_for_up_faces();
        let expected = vec![&TURN_U, &TURN_F, &TURN_BL, &TURN_BR];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_down_turns() {
        let turns = RawTurn::get_for_down_faces();
        let expected = vec![&TURN_L, &TURN_R, &TURN_B, &TURN_D];
        assert_eq!(turns, expected);
    }

    #[test]
    fn test_get_turn_effect_type() {
        assert_eq!(TURN_U.get_effect(TurnEffectType::CornerPermutation).as_ref(), &TURN_U.corner_permutation);
        assert_eq!(TURN_U.get_effect(TurnEffectType::CornerOrientation).as_ref(), &TURN_U.corner_orientation);
        assert_eq!(TURN_U.get_effect(TurnEffectType::EdgeInFace).as_ref(), &TURN_U.edges);
        assert_eq!(TURN_U.get_effect(TurnEffectType::EdgeAcrossFaces).as_ref(), &TURN_U.edges);
        assert_eq!(TURN_U.get_effect(TurnEffectType::UpCentre).as_ref(), &TURN_U.up_centres);
        assert_eq!(TURN_U.get_effect(TurnEffectType::TripleCentre).as_ref(), &TURN_U.triple_centres);

        assert_eq!(TURN_D.get_effect(TurnEffectType::CornerPermutation).as_ref(), &TURN_D.corner_permutation);
        assert_eq!(TURN_D.get_effect(TurnEffectType::CornerOrientation).as_ref(), &TURN_D.corner_orientation);
        assert_eq!(TURN_D.get_effect(TurnEffectType::EdgeInFace).as_ref(), &TURN_D.edges);
        assert_eq!(TURN_D.get_effect(TurnEffectType::EdgeAcrossFaces).as_ref(), &TURN_D.edges);
        assert_eq!(TURN_D.get_effect(TurnEffectType::UpCentre).as_ref(), &TURN_D.up_centres);
        assert_eq!(TURN_D.get_effect(TurnEffectType::TripleCentre).as_ref(), &TURN_D.triple_centres);
    }
}