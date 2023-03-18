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
    belonging on the D face) for conveinience, although mechanically these corners are interchangable and are not technically different
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

use lazy_static::lazy_static;
use rand::{thread_rng, Rng};

use crate::coordinates::{CoordinateType, NUM_CORNER_STATES, get_down_centre_coord_for_matched_triples, NUM_FACE_PIECE_PERMS, NUM_ACROSS_FACE_PERMS};
use crate::movedefs::{RawTurn, NUM_CORNERS, Turn};
use crate::movetables::{MoveTables, ApplyMove};

lazy_static! {
    static ref SOLVED_CENTRES: [u32; NUM_CORNER_STATES] = precompute_solved_triple_centre_coords();
}


#[derive(Clone, Debug, PartialEq)]
pub struct RawState {
    pub corners: Vec<u8>,
    pub corner_orientation: u8,
    pub edges: Vec<u8>,
    pub up_centres: Vec<u8>,
    pub down_centres: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordState {
    pub corners: u32,
    pub edges_within_faces: u32,
    pub edges_across_faces: u32,
    pub up_centres: u32,
    pub down_centres: u32,
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
        let corners = vec![0,1,2,3,4,5];
        let corner_orientation = 0b000000;
        let edges = vec![0,1,2,3,4,5,6,7,8,9,10,11];
        let up_centres = vec![0,0,0,3,3,3,6,6,6,9,9,9];
        let down_centres = vec![0,0,0,3,3,3,6,6,6,9,9,9];

        Self::new(&corners, corner_orientation, &edges, &up_centres, &down_centres)
    }

    pub fn apply_sequence(&mut self, sequence: &[&Turn]) {
        for turn in sequence {
            self.apply(turn);
        }
    }

    pub fn apply(&mut self, turn: &Turn) {
        let m: &RawTurn = turn.face.get_raw_turn();

        apply_raw_permutation(&mut self.corners, &m.corner_permutation);
        apply_orientation(&mut self.corner_orientation, &m.corner_permutation, &m.corner_orientation[0]);
        apply_raw_permutation(&mut self.edges, &m.edges);
        apply_raw_permutation(&mut self.up_centres, &m.up_centres);
        apply_raw_permutation(&mut self.down_centres, &m.down_centres);

        if turn.invert {
            self.apply(&Turn::new(turn.face, false));
        }
    }

    pub fn to_coords(&self) -> CoordState {
        CoordState {
            corners: self.get_corner_coord(),
            edges_within_faces: self.get_edge_within_face(),
            edges_across_faces: self.get_edge_across_face(),
            up_centres: self.get_up_centres(),
            down_centres: self.get_down_centres(),
        }
    }

    fn get_corner_coord(&self) -> u32 {
        let mut state = self.corners.clone();
        let mut orientation = self.corner_orientation;
        let mut first_flip: u8 = 0;
        for i in (1..NUM_CORNERS).rev() {
            let flip = orientation % 2;
            orientation /= 2;

            first_flip ^= flip;
            state[i] = state[i]*2 + flip;
        }
        state[0] = state[0]*2 + first_flip;
        CoordinateType::CornerState.state_to_coord(&state)
    }

    fn get_edge_within_face(&self) -> u32 {
        CoordinateType::EdgeInFace.state_to_coord(&self.edges)
    }

    fn get_edge_across_face(&self) -> u32 {
        CoordinateType::EdgeAcrossFaces.state_to_coord(&self.edges)
    }

    fn get_up_centres(&self) -> u32 {
        CoordinateType::UpCentre.state_to_coord(&self.up_centres)
    }

    fn get_down_centres(&self) -> u32 {
        CoordinateType::DownCentre.state_to_coord(&self.down_centres)
    }
}

impl CoordState {
    pub fn solved() -> Self {
        Self {
            corners: 0,
            edges_within_faces: 0,
            edges_across_faces: 0,
            up_centres: 0,
            down_centres: 0,
        }
    }

    pub fn get_random() -> Self {
        let mut rng = thread_rng();
        Self {
            corners: rng.gen_range(0..NUM_CORNER_STATES) as u32,
            edges_within_faces: rng.gen_range(0..NUM_FACE_PIECE_PERMS) as u32,
            edges_across_faces: rng.gen_range(0..NUM_ACROSS_FACE_PERMS) as u32,
            up_centres: rng.gen_range(0..NUM_FACE_PIECE_PERMS) as u32,
            down_centres: rng.gen_range(0..NUM_FACE_PIECE_PERMS) as u32,
        }
    }

    pub fn apply_sequence(&mut self, move_tables: &MoveTables, sequence: &[&Turn]) {
        for turn in sequence {
            self.apply(move_tables, turn);
        }
    }

    pub fn apply(&mut self, move_tables: &MoveTables, turn: &Turn) {
        self.corners = move_tables.apply_move_to_coord(self.corners, CoordinateType::CornerState, turn);
        self.edges_within_faces = move_tables.apply_move_to_coord(self.edges_within_faces, CoordinateType::EdgeInFace, turn);
        self.edges_across_faces = move_tables.apply_move_to_coord(self.edges_across_faces, CoordinateType::EdgeAcrossFaces, turn);
        self.up_centres = move_tables.apply_move_to_coord(self.up_centres, CoordinateType::UpCentre, turn);
        self.down_centres = move_tables.apply_move_to_coord(self.down_centres, CoordinateType::DownCentre, turn);
    }

    pub fn to_raw(&self) -> RawState {
        RawState {
            corners: self.get_corner_permutation(),
            corner_orientation: self.get_corner_orientation(),
            edges: self.get_edges(),
            up_centres: self.get_up_centres(),
            down_centres: self.get_down_centres(),
        }
    }

    fn get_corner_permutation(&self) -> Vec<u8> {
        let mut corner_permutation = CoordinateType::CornerState.coord_to_state(self.corners);
        for i in 0..corner_permutation.len() {
            corner_permutation[i] /= 2;
        }
        corner_permutation
    }

    fn get_corner_orientation(&self) -> u8 {
        let mut orientation = (self.corners / 360) as u8;
        // This is the orientation of the last 5 corners. We need to calculate whether the remaining corner
        // needs to be flipped.
        let mut temp = orientation;
        let mut first_flip: u8 = 0;
        for _ in 0..NUM_CORNERS {
            first_flip ^= temp % 2;
            temp /= 2;
        }
        orientation += first_flip << 5;
        orientation
    }

    fn get_edges(&self) -> Vec<u8> {
        let mut state = CoordinateType::EdgeInFace.coord_to_state(self.edges_within_faces);
        let across_face = CoordinateType::EdgeAcrossFaces.coord_to_state(self.edges_across_faces);
        for i in 0..state.len() {
            state[i] += across_face[i];
        }
        state
    }

    fn get_up_centres(&self) -> Vec<u8> {
        CoordinateType::UpCentre.coord_to_state(self.up_centres)
    }

    fn get_down_centres(&self) -> Vec<u8> {
        CoordinateType::DownCentre.coord_to_state(self.down_centres)
    }
}


pub fn do_triple_centres_match_corners(corners: u32, down_centres: u32) -> bool {
    SOLVED_CENTRES[corners as usize] == down_centres
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

fn precompute_solved_triple_centre_coords() -> [u32; NUM_CORNER_STATES] {
    let mut coord_map = [0; NUM_CORNER_STATES];
    // for coord in 0..NUM_CORNER_STATES {
    //     coord_map[coord] = get_down_centre_coord_for_matched_triples(coord as u32);
    // }
    for (coord, item) in coord_map.iter_mut().enumerate().take(NUM_CORNER_STATES) {
        *item = get_down_centre_coord_for_matched_triples(coord as u32);
    }
    coord_map
}

#[cfg(test)]
mod tests {
    use crate::movedefs::Face;

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

    #[test]
    fn test_to_raw() {
        let mut coord_state = CoordState::solved();
        let mut raw_state = coord_state.to_raw();
        let mut expected = RawState::solved();
        assert_eq!(raw_state, expected);

        coord_state.corners = 360;
        raw_state = coord_state.to_raw();
        expected.corner_orientation = 33;

        assert_eq!(raw_state, expected);
    }

    #[test]
    fn test_to_coord() {
        let mut raw_state = RawState::solved();
        let mut coord_state = raw_state.to_coords();
        let mut expected = CoordState::solved();
        assert_eq!(coord_state, expected);

        raw_state.corner_orientation = 33;
        coord_state = raw_state.to_coords();
        expected.corners = 360;

        assert_eq!(coord_state, expected);
    }

    #[test]
    fn test_apply_move() {
        let mut state = RawState::solved();

        state.apply(&Turn::new(Face::U, false));

        let expected = RawState::new(
            &[2,0,1,3,4,5],
            0,
            &[2,0,1,3,4,5,6,7,8,9,10,11],
            &[0,0,0,3,3,3,6,6,6,9,9,9],
            &[6,6,0,0,0,3,3,3,6,9,9,9]
        );

        assert_eq!(state, expected);
    }

    #[test]
    fn test_apply_move_sequence() {
        let mut state = RawState::solved();

        state.apply_sequence(&[
            &Turn::new(Face::U, true),
            &Turn::new(Face::U, true)
        ]);

        let expected = RawState::new(
            &[2,0,1,3,4,5],
            0,
            &[2,0,1,3,4,5,6,7,8,9,10,11],
            &[0,0,0,3,3,3,6,6,6,9,9,9],
            &[6,6,0,0,0,3,3,3,6,9,9,9]
        );

        assert_eq!(state, expected);
    }
}
