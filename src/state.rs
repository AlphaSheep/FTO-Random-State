
/*
    PIECE DEFINITIONS:
    The orientation and face moves match those used in Ben Streeter's document: 
    https://docs.google.com/document/d/e/2PACX-1vTDL7-XvpNrhIc2Q_1nHfeJyG7tIazgBCq88PE8ahqIbvPb3LPQsM3_vsdqX6y8sxte1n5jGk2J3c5V/pub
    The puzzle is oriented with one face flat on the table, with the triangle pointing away from you. This is labelled the D face. 
    The other horizontal face points toward you and is labelled the U face. There are four visible faces, clockwise U, R, F, and L.
    The other faces are D opposite U, B opposite F, BL opposite R, and BR opposite L.
    The eight possible moves are clockwise turns of corresponding faces.

    Corner permutation:
    NOTE: We make a distinction between up corners (corners which have a sticker belonging on the U face) and down corners (with a sticker 
    belonging on the D face) for conveinience, although mechanically these corners are interchangable and are not tehnically different 
    piece types. This allows us to label corners with two faces - either U or D, and the face of the sticker that can be interchanged 
    with either U or D. This has advantages for defining orientation.

    We assign an ordering to the corners as follows:
         0    1    2   3   4   5
         UBL  UBR  UF  DB  DR  DL
    Note that the permutation coordinate only needs to encode the position of 4 corners as the remaining two can be deduced as the corners
    must always have even parity.    

    There are 6!/2 = 360 possible corner permutations.
    
    Corner Orientation:
    Corner orientation is defined differently for up corners and down corners.
        - Up corners on the U face: Oriented if the U sticker points up.
        - Up corners on the D face: Oriented if the U sticker points clockwise when viewing the puzzle from above.
        - Down corners on the U face: Oriented if the D sticker points clockwise when viewing the puzzle from above.
        - Down corners on the D face: Oriented if the D sticker points down.

    There are (2^6)/2 = 32 possible combinations of corner orientation.

    Edge permutation:
    We label edges with the two faces that the stickers belong to, preferring to use the up face first. Since there is no orientation of edges, 
    we only need to track permutation.

    We assign an ordering to the edges as follows:
         0   1   2   3    4    5    6    7    8    9   10  11
         UB  UR  UL  BLB  BLL  BLD  BRR  BRB  BRD  FL  FR  FD
    Note that again the permutation coordinate only needs to encode the position of 10 edges as edges must always have even parity.

    There are 12!/2  = 239_500_800 edge possible edge permutations.

    Centre permutation:
    We distinguish between two centre types: up centres (that can be move to the U face) and down centres (which can be moved to the D face). 
    Unlike corners, these are mechanically two different piece types, and cannot be interchanged. They are also not unique. There are 12 centres 
    of each type, grouped in four sets of three. We label centres with just the face that they belong on, noting that this is not a unique label 
    (all three centres that belong on a face are interchangable have the same label). We label all possible positions uniquely, however, using 
    two faces, the first being the face on which the centre is, and the second being the opposite face on the corner that the centre is touching.
    See the labels and ordering below if this is confusing.
    
    We assign a number based on the face where these pieces belong:
    Up centres:
        0  1   2   3
        U  BL  BR  F
    Down centres:
        0  1  2  3
        B  R  L  D
    We assign an ordering for postions: 
    Up centres:
        0    1    2   3    4    5     6    7     8    9   10   11
        UBL  UBR  UF  BLU  BLF  BLBR  BRU  BRBL  BRF  FU  FBR  FBL 
    Down centres:
        0   1   2   3   4   5   6   7   8   9   10  11
        BR  BL  BD  RL  RB  RD  LB  LR  LD  DL  DR  DB   

    There are 12!/(3!^4) = 369_600 permutations for up centers, and the same number for down centres.

    Additional piece types:
    Hex edge permutation:
    We define a hex as three edges and three up centres arranged to form a hexagon. Hexes are special as they are not broken by 
    up face turns. There are four hexes that may be formed. Every edge is part of a hex, and we define the hex edge permutation 
    identically to the up centre permutation, that is, by treating all three edges with the same up face as if they were interchangable.

    Hex orientation:
    Hex orientation is only meaningful when the hex edge permutation is correctly ordered.

    We define the hex orientation of hex n is defined as (p[3n+1] - p[3n]) mod 3, where p[i] is the full edge permutation 
    (note: not hex edge permutation) of edge i, and n is 0, 1, 2, or 3. A hex orientation of 0 is not possible if the hex is 
    formed from a single colour, so hex orientation can only take a value of 1 or 2. If a hex orientation is 1, then full edge
    permutation may be solved with at most a single turn of each up face, and no down face turns. If the hex orientation is 2, 
    then solving edge permutation requires multiple down face turns.

    Triple centre permutation:
    We define a triple as a corner and the two down centres touching it. As with hexes, triples are not affected by up face turns.
    The triple centre permutation is defined similarly to the down centre permutation, except the order is not fixed, but follows 
    corners as they move. This means that a single turn of a down face will only affect the permutation of triple centres that are not 
    physically moved by the turn, as the corner that those centres were attached to moves.
*/

use crate::movedefs::{Face, RawTurn};
use crate::movetables::MoveTables;


#[derive(Clone, Copy)]
pub struct RawState {
    pub corners: [u8; 6],
    pub corner_orientation: u8,
    pub edges: [u8; 12],
    pub up_centres: [u8; 12],
    pub down_centres: [u8; 12],
}

impl RawState {
    pub fn new(corners: &[u8], corner_orientation: u8, edges: &[u8], up_centres: &[u8], down_centres: &[u8]) -> Self {
        Self {
            corners: corners.try_into().unwrap(), 
            corner_orientation, 
            edges: edges.try_into().unwrap(), 
            up_centres: up_centres.try_into().unwrap(), 
            down_centres: down_centres.try_into().unwrap(),
        }
    }

    pub fn solved() -> Self {
        let corners: [u8; 6] = [0,1,2,3,4,5];
        let corner_orientation: u8 = 0b000000;
        let edges: [u8; 12] = [0,1,2,3,4,5,6,7,8,9,10,11];
        let up_centres: [u8; 12] = [0,0,0,3,3,3,6,6,6,9,9,9];
        let down_centres: [u8; 12] = [0,0,0,3,3,3,6,6,6,9,9,9];

        Self::new(&corners, corner_orientation, &edges, &up_centres, &down_centres)
    }

    pub fn apply(&mut self, m: &RawTurn) {
        apply_raw_permutation(&mut self.corners, &m.corner_permutation);
        apply_orientation(&mut self.corner_orientation, &m.corner_permutation, &m.corner_orientation[0]);
        apply_raw_permutation(&mut self.edges, &m.edges);
        apply_raw_permutation(&mut self.up_centres, &m.up_centres);
        apply_raw_permutation(&mut self.down_centres, &m.down_centres);
    }
}

pub fn apply_raw_permutation<T>(state: &mut [T], effect: &[u8]) 
where T: Copy + Clone
{
    let orig_state: Vec<T> = state.to_vec();
    
    for i in 0..effect.len() {
        state[i] = orig_state[effect[i] as usize];
    }
}

pub fn apply_orientation(state: &mut u8, perm_effect: &[u8], orient_effect: &u8) {
    let mut flip_state = flip_num_to_bool_array(state);
    apply_raw_permutation::<bool>(&mut flip_state, perm_effect);
    *state = flip_bool_array_to_num(&flip_state) ^ orient_effect;
}

pub fn apply_full_corner(state: &mut [u8], perm_effect: &[u8], orient_effect: &[u8]) {
    apply_raw_permutation(state, perm_effect);
    let mut orientation = orient_effect[0];
    let mut first_flip: u8 = 0;
    for i in (1..state.len()).rev() {
        let flip = orientation % 2;
        orientation /= 2;
        first_flip ^= flip;
        state[i] ^= flip;
    }
    state[0] ^= first_flip;
}

pub fn flip_num_to_bool_array(state: &u8) -> [bool; 6] {
    let mut flips: [bool; 6] = [false, false, false, false, false, false];
    let mut remaining_flips = *state;
    for i in (0..6).rev() {
        flips[i] = (remaining_flips % 2) == 1;
        remaining_flips /= 2;
    }
    flips
}

fn flip_bool_array_to_num(state: &[bool]) -> u8 {
    let mut num: u8 = 0;
    for flipped in state {
        num *= 2;
        if *flipped {
            num += 1;
        }
    }
    num
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[false, false, false, false, false, false], 0)]
    #[test_case(&[false, false, false, false, false, true], 1)]
    #[test_case(&[true, true, false, false, false, false], 0b110000)]
    #[test_case(&[true, true, true, true, true, true], 0b111111)]
    fn test_flip_bool_array_to_num(input: &[bool], expected: u8) {
        assert_eq!(flip_bool_array_to_num(input), expected);
    }

    #[test_case(&0, [false, false, false, false, false, false])]
    #[test_case(&1, [false, false, false, false, false, true])]
    #[test_case(&0b110000, [true, true, false, false, false, false])]
    #[test_case(&0b111111, [true, true, true, true, true, true])]
    fn test_flip_num_to_bool_array(input: &u8, expected: [bool; 6]) {
        assert_eq!(flip_num_to_bool_array(input), expected);
    }

    #[test_case([0,1,2,3,4,5], &[0,1,2,3,4,5], &[0,1,2,3,4,5])]
    #[test_case([0,1,2,3,4,5], &[1,2,3,4,5,0], &[1,2,3,4,5,0])]
    #[test_case([1,2,3,4,5,0], &[0,1,2,3,4,5], &[1,2,3,4,5,0])]
    #[test_case([3,2,1,4,5,0], &[5,3,4,2,0,1], &[0,4,5,1,3,2])]
    fn test_apply_permutation(start_state: [u8; 6], permutation: &[u8], expected: &[u8]) {
        let mut state : [u8;6] = start_state;
        apply_raw_permutation::<u8>(&mut state, permutation);
        assert_eq!(state, expected);
    }

    #[test_case(0, &[0,1,2,3,4,5], &0, 0)]
    #[test_case(0, &[0,1,2,3,4,5], &0b000101, 0b000101)]
    #[test_case(0, &[0,1,2,3,4,5], &0b111001, 0b111001)]
    #[test_case(0, &[0,1,2,5,3,4], &0b010001, 0b010001)]
    #[test_case(0b010001, &[0,1,2,5,3,4], &0b000000, 0b010100)]
    #[test_case(0b010001, &[0,1,2,5,3,4], &0b010001, 0b000101)]
    fn test_apply_orientation(start_state: u8, permutation_effect: &[u8], orientation_effect: &u8, expected: u8) {
        let mut state : u8 = start_state;
        apply_orientation(&mut state, permutation_effect, orientation_effect);
        assert_eq!(state, expected);
    }

    #[test_case([0,2,4,6,8,10], &[0,1,2,3,4,5], &[0], &[0,2,4,6,8,10])]
    #[test_case([2,4,6,8,10,0], &[0,1,2,3,4,5], &[0b000101], &[2,4,6,9,10,1])]
    #[test_case([6,4,2,8,10,0], &[5,3,4,2,0,1], &[0b000000], &[0,8,10,2,6,4])]
    #[test_case([6,4,2,8,10,0], &[5,3,4,2,0,1], &[0b010001], &[0,9,10,2,6,5])]
    #[test_case([0,2,4,6,8,10], &[0,1,5,3,2,4], &[0b001001], &[0,2,11,6,4,9])]
    #[test_case([6,4,2,9,10,1], &[5,3,4,2,0,1], &[0b010001], &[1,8,10,2,6,5])]
    fn test_apply_full_corner(start_state: [u8; 6], permutation_effect: &[u8], orientation_effect: &[u8], expected: &[u8]) {
        let mut state : [u8;6] = start_state;
        apply_full_corner(&mut state, permutation_effect, orientation_effect);
        assert_eq!(state, expected);
    }
}
