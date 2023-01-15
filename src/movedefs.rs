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

use phf::{phf_map, Map};

pub struct Turn {
    pub corners: [u8; 6],
    pub corner_orientation: u8,
    pub edges: [u8; 12],
    pub up_centres: [u8; 12],
    pub down_centres: [u8; 12],
    pub triple_centres: [u8; 12],
}

impl Turn {
    pub fn get(face: &str) -> &Self {
        RAW_MOVE_LOOKUP.get(face)
        .expect("Face should be a valid face")
    }
}

static RAW_MOVE_LOOKUP: Map<&'static str, Turn> = phf_map! {
    "U" => Turn {
        corners: [2,0,1,3,4,5],
        corner_orientation: 0b000000,
        edges: [2,0,1,3,4,5,6,7,8,9,10,11],
        up_centres: [2,0,1,3,4,5,6,7,8,9,10,11],
        down_centres: [6,7,2,0,1,5,3,4,8,9,10,11],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },    
    "F" => Turn {
        corners: [0,1,5,3,2,4],
        corner_orientation: 0b001001,
        edges: [0,1,2,3,4,5,6,7,8,11,9,10],
        up_centres: [0,1,2,3,4,5,6,7,8,11,9,10],
        down_centres: [0,1,2,8,4,7,6,9,10,5,3,11],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
    "BL" => Turn {
        corners: [3,1,2,5,4,0],
        corner_orientation: 0b100100,
        edges: [0,1,2,5,3,4,6,7,8,9,10,11],
        up_centres: [0,1,2,5,3,4,6,7,8,9,10,11],
        down_centres: [0,11,9,3,4,5,2,7,1,6,10,8],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
    "BR" => Turn {
        corners: [0,4,2,1,3,5],
        corner_orientation: 0b010010,
        edges: [0,1,2,3,4,5,8,6,7,9,10,11],
        up_centres: [0,1,2,3,4,5,8,6,7,9,10,11],
        down_centres: [5,1,4,3,10,11,6,7,8,9,2,0],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
    "L" => Turn {
        corners: [5,1,0,3,4,2],
        corner_orientation: 0b101000,
        edges: [0,1,4,3,9,5,6,7,8,2,10,11],
        up_centres: [4,1,3,11,9,5,6,7,8,0,10,2],
        down_centres: [0,1,2,3,4,5,8,6,7,9,10,11],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
    "R" => Turn {
        corners: [0,2,4,3,1,5],
        corner_orientation: 0b011000,
        edges: [0,10,2,3,4,5,1,7,8,9,6,11],
        up_centres: [0,9,10,3,4,5,2,7,1,8,6,11],
        down_centres: [0,1,2,5,3,4,6,7,8,9,10,11],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
    "B" => Turn {
        corners: [1,3,2,0,4,5],
        corner_orientation: 0b110000,
        edges: [7,1,2,0,4,5,6,3,8,9,10,11],
        up_centres: [6,7,2,1,4,0,5,3,8,9,10,11],
        down_centres: [2,0,1,3,4,5,6,7,8,9,10,11],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
    "D" => Turn {
        corners: [0,1,2,4,5,3],
        corner_orientation: 0b000000,
        edges: [0,1,2,3,4,8,6,7,11,9,10,5],
        up_centres: [0,1,2,3,7,8,6,10,11,9,4,5],
        down_centres: [0,1,2,3,4,5,6,7,8,11,9,10],
        triple_centres: [0,1,2,3,4,5,6,7,8,9,10,11],
    },
};
