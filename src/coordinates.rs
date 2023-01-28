use core::panic;
use lazy_static::lazy_static;

use crate::state::apply_raw_permutation;
use crate::movedefs::TurnEffectType;


pub const NUM_CORNER_PERMS: usize = 360;
pub const NUM_CORNER_ORIENTATIONS: usize = 32;
pub const NUM_EDGE_PERMS: usize = 239_500_800;
pub const NUM_FACE_PIECE_PERMS: usize = 369_600;
pub const NUM_ACROSS_FACE_PERMS: usize = 34_650;

const NUM_CORNERS: usize = 6;
const NUM_EDGES: usize = 12;
const NUM_CENTRES: usize = 12;




lazy_static! {
    static ref BINOMIAL_TABLE: [[u64; 13]; 13] = precompute_binomial_table();
}

fn precompute_binomial_table() -> [[u64; 13]; 13] {
    let mut binomial_table = [[0; 13]; 13];
    for n in 0..=12 {
        binomial_table[n][0] = 1;
        for k in 1..=n {
            binomial_table[n][k] = binomial_table[n-1][k-1] + binomial_table[n-1][k];
        }
    }
    binomial_table
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coordinate {
    CornerPermutation,
    CornerOrientation,
    EdgeInFace,
    EdgeAcrossFaces,
    UpCentre,
    TripleCentre,
}

impl Coordinate {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::CornerPermutation,
            // Self::CornerOrientation,
            Self::EdgeInFace,
            Self::EdgeAcrossFaces,
            Self::UpCentre,
            Self::TripleCentre,
        ].iter().copied()
    }

    pub fn state_to_coord(&self, state: &[u8]) -> u32 {
        match self {
            Self::CornerPermutation => permutation_to_coord(state),
            Self::CornerOrientation => state[0] as u32,
            Self::EdgeInFace => face_position_to_coord(state),
            Self::EdgeAcrossFaces => perm_across_face_coord(state),
            Self::UpCentre => face_position_to_coord(state),
            Self::TripleCentre => face_position_to_coord(state),
        }
    }

    pub fn coord_to_state(&self, coord: u32) -> Vec<u8> {
        match self {
            Self::CornerPermutation => invert_coord_to_permutation::<6>(coord).to_vec(),
            Self::CornerOrientation => vec![coord as u8],
            Self::EdgeInFace => invert_coord_to_face_positions(coord).to_vec(),
            Self::EdgeAcrossFaces => invert_coord_to_perm_across_face(coord).to_vec(),
            Self::UpCentre => invert_coord_to_face_positions(coord).to_vec(),
            Self::TripleCentre => invert_coord_to_face_positions(coord).to_vec(),
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            Self::CornerPermutation => NUM_CORNER_PERMS,
            Self::CornerOrientation => NUM_CORNER_ORIENTATIONS,
            Self::EdgeInFace => NUM_FACE_PIECE_PERMS,
            Self::EdgeAcrossFaces => NUM_ACROSS_FACE_PERMS,
            Self::UpCentre => NUM_FACE_PIECE_PERMS,
            Self::TripleCentre => NUM_FACE_PIECE_PERMS,
        }
    }

    pub fn get_turn_effect_type(&self) -> TurnEffectType {
        match self {
            Self::CornerPermutation => TurnEffectType::CornerPermutation,
            Self::CornerOrientation => TurnEffectType::CornerOrientation,
            Self::EdgeInFace => TurnEffectType::EdgeInFace,
            Self::EdgeAcrossFaces => TurnEffectType::EdgeAcrossFaces,
            Self::UpCentre => TurnEffectType::UpCentre,
            Self::TripleCentre => TurnEffectType::TripleCentre,
        }
    }
}

/// Converts a permutation of up to 12 pieces into a single 32-bit coordinate.
/// First we encode the number of elements after the nth element that belong before it.
/// We ignore the two left most pieces as we assume that the permutation will always be even.
/// Then we treat this as a variable base number system where the coefficient of the nth 
/// digit from the right is (n!/2). 
/// 
/// Example:
/// ```
///   let coord = permutation_to_coord(&[0,1,2,3,4,5]);
/// ```
fn permutation_to_coord(positions: &[u8]) -> u32 {
    let mut coord: u32 = 0;  // Note: 32 bits can only handle up to 12 pieces.
    for i in (2..positions.len()).rev() { 
        for j in 0..i {
            if positions[i] < positions[j] {
                coord += 1;
            }
        }
        if i > 2 {
            coord *= i as u32;
        }
    }
    coord
}

/// Converts a 32-bit coordinate into a permutation of up to 12 pieces.
/// 
/// Example
/// ```
///   let perm = invert_coord_to_permutation::<6>(0);
/// ```
fn invert_coord_to_permutation<const N: usize>(coord: u32) -> [u8; N] {
    let mut perm: [u8; N] = invert_coord_to_permutation_ignore_parity::<N>(coord);    
    if !is_even_parity(&perm) {
        perm.swap(0, 1);
    }
    perm
}

fn invert_coord_to_permutation_ignore_parity<const N: usize>(mut coord: u32) -> [u8; N] {
    let mut perm: [u8; N] = [0; N];
    let mut available: Vec<usize> = (0..N).rev().collect();
    let factors: [usize; N] = get_factors();
    for i in (0..N).rev() {
        perm[i] = available.remove(coord as usize / factors[i]) as u8;
        coord %= factors[i] as u32;
    }
    perm
}

/// Converts a permutation 4 sets of 3 centre pieces into a single 32-bit coordinate.
/// We do this by creating 3 sub-coordinates, with each sub-coordinate encoding only one 
/// set of identical centres, and ignoring pieces already encoded in another sub-coord.
/// Therefore, the three sub-coords encode 3 positions out of 12, 9 and 6 possible positions.
/// For each possible position that is not occupied by the centre of interest, we capture the 
/// number of possible combinations of pieces that come before that position. The sub-coordinate 
/// is then the sum of all of these coordinates. These coordinates have ranges of 0..19, 0..83 
/// and 0..219 (ranges of size binomial(6,3), binomial(9,3), and binomial(12,3) respectively),
/// so we combine them to a total coordinate with a range of 0..369599.
/// 
/// Note: This is also useful for edges if you treat edges from the same up face as 
/// interchangeable. When used in conjunction with the edge across face coordinate, the two 
/// combined can give the full edge permutation.
/// 
/// Example:
/// ```
///   let coord = face_position_to_coord(&[0,1,2,3,4,5,6,7,8,9,10,11]);
/// ```
fn face_position_to_coord(positions: &[u8]) -> u32 {
    sub_permutation_coord(positions, 4, 3)
}

/// Converts a 32-bit coordinate into a permutation of 4 sets of 3 pieces.
/// First we split the coordinate into the 3 sub-coordinates, then we extract the masked 
/// vector of occupied positions for 3 of the 4 centre sets. 
/// 
/// Example
/// ```
///   let perm = invert_coord_to_face_positions(0);
/// ```
fn invert_coord_to_face_positions(coord: u32) -> [u8; NUM_CENTRES] {    
    invert_coord_to_sub_permutation::<3>(coord)
}

/// Converts a permutation of edges into a coordinate that encodes the positions of edges within a face.
/// We define 3 groups of edges, with each group consisting of a single edge from each up face. This is 
/// done by grouping edges according to the defined order modulo 3. This is then encoded in exactly the same
/// way as the face position coordinate, although there are 3 groups of 4 instead of 4 groups of 3. There are 
/// therefore 2 sub coordinates, since the final group is determined by the other two. The sub coordinates have 
/// ranges 0..494 and 0..69, giving a combined coordinate range of 0..34649.
fn perm_across_face_coord(edges: &[u8]) -> u32 {
    let mut positions: [u8; NUM_EDGES] = [0,4,8,1,5,9,2,6,10,3,7,11];
    let ordering: [u8; NUM_EDGES] = [0,3,6,9,1,4,7,10,2,5,8,11];
    apply_raw_permutation(&mut positions, edges);
    apply_raw_permutation(&mut positions, &ordering);
    sub_permutation_coord(&positions, 3, 4)
}

fn invert_coord_to_perm_across_face(coord: u32) -> [u8; NUM_EDGES] {
    let mut positions: [u8; NUM_EDGES] = [0,1,2,0,1,2,0,1,2,0,1,2];
    let ordering: [u8; NUM_EDGES] = [0,4,8,1,5,9,2,6,10,3,7,11];
    let mut state = invert_coord_to_sub_permutation::<4>(coord);
    apply_raw_permutation(&mut state, &ordering);
    apply_raw_permutation(&mut positions, &state);
    positions
}

fn invert_coord_to_sub_permutation<const N: usize>(coord: u32) -> [u8; NUM_CENTRES] {
    let mut state: [u8; NUM_CENTRES] = [0; NUM_CENTRES];
    let sub_coords = get_sub_coords::<N>(coord);

    let num_levels: usize = (NUM_CENTRES / N) - 1;

    for i in 0..num_levels {
        let mut pieces = invert_single_face_centre_coord(
            sub_coords[num_levels-1-i], 
            N as u8,
            (NUM_CENTRES - (N*i)) as u8,
            (NUM_CENTRES - (N * (i+1))) as u8);       
        
        for j in 0..NUM_CENTRES {
            if state[j] == 0 {
                let piece = pieces.pop().unwrap();
                if piece != u8::MAX {
                    state[j] = piece;
                }
            }
        }
    }
    state
}

fn sub_permutation_coord(positions: &[u8], num_groups: u32, num_per_group: u32) -> u32 {
    let mut coord: u32 = 0;    
    for i in (0..num_groups).rev() {
        let face = num_per_group * (i+1);
        let mut n = -1;
        let mut k = -1;
        for position in positions {
            let piece = (position / num_per_group as u8) * num_per_group as u8;
            n += (piece <= (face as u8)) as i32;
            k += (piece == (face as u8)) as i32;
            if  (n >= 0) && (k >= 0) && (piece <= (face as u8)) && (piece != (face as u8)) {
                coord += BINOMIAL_TABLE[n as usize][k as usize] as u32;
            }
        }
        let mut multiplier: u32 = face;
        let mut divider: u32 = 1;
        for j in 1..num_per_group {
            multiplier *= face - j;
            divider *= (j+1);
        }        
        coord *= multiplier / divider;
    }
    coord
}

fn get_factors<const N: usize>() -> [usize; N] {
    let mut factors = [1 ; N];
    for i in 3..N {
        factors[i] = factors[i-1] * i;
    }
    factors
}

fn is_even_parity(perm: &[u8]) -> bool {
    // TODO. This is O(n^2). It would be nice to use a O(n log n) method.
    let n = perm.len();
    let mut result = true;
    for i in 0..(n-1) {
        for j in (i+1)..n {
            result ^= perm[i] > perm[j];
        }
    }
    result
}

fn get_sub_coords<const N: usize>(mut coord: u32) -> Vec<u32> {    
    let num_levels: usize = (NUM_CENTRES / N) - 1;
    let factors: Vec<u32> = match N {
        4 => vec![70, 495],
        3 => vec![20, 84, 220],
        _ => panic!(),
    };

    let mut sub_coords: Vec<u32> = vec![0; num_levels];

    for i in 0..num_levels {
        sub_coords[i] = coord % factors[i];
        coord /= factors[i];
    }
    sub_coords
}

fn invert_single_face_centre_coord(mut coord: u32, num_pieces_to_place: u8, num_positions_to_fill: u8, fill_piece: u8) -> Vec<u8> {
    // Returns the permutation in reverse order, this allows using pop to get each piece in order
    // We use u8max as a mask to unoccupied postions.
    let mut pieces: Vec<u8> = vec![u8::MAX; num_positions_to_fill as usize];
    let mut num_left = num_pieces_to_place as usize;

    for j in 0..num_positions_to_fill {
        let n = num_positions_to_fill - j - 1;
        let n_choose_k = BINOMIAL_TABLE[n as usize][num_left-1] as u32;
        if coord >= n_choose_k {
            coord -= n_choose_k;
        }
        else {
            pieces[j as usize] = fill_piece;
            num_left -= 1;
        }
        if num_left < 1 {
            break;
        }
    }
    pieces
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[0,1,2,3,4,5], 0)]
    #[test_case(&[2,0,1,3,4,5], 1)]
    #[test_case(&[4,5,3,2,1,0], 359)]
    fn test_corner_perm_to_coord(state: &[u8], value: u32) {
        let coord = Coordinate::CornerPermutation;
        assert_eq!(coord.state_to_coord(state), value);
    }

    #[test_case(&[0b00000000], 0)]
    #[test_case(&[0b00000001], 1)]
    #[test_case(&[0b00011111], 31)]
    fn test_corner_orientation_to_coord(state: &[u8], value: u32) {
        let coord = Coordinate::CornerOrientation;
        assert_eq!(coord.state_to_coord(state), value);
    }

    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[1,0,3,2,4,5,6,7,8,9,10,11], 1)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 369_599)]
    fn test_edge_to_face_coord(state: &[u8], value: u32) {
        let coord = Coordinate::EdgeInFace;
        assert_eq!(coord.state_to_coord(state), value);
    }

    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[9,0,2,3,4,5,6,7,8,1,10,11], 1)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 34_649)]
    fn test_edge_to_across_face_coord(state: &[u8], value: u32) {
        let coord = Coordinate::EdgeAcrossFaces;
        assert_eq!(coord.state_to_coord(state), value);
    }

    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[1,0,3,2,4,5,6,7,8,9,10,11], 1)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 369_599)]
    fn test_up_centers_to_coord(state: &[u8], value: u32) {
        let coord = Coordinate::UpCentre;
        assert_eq!(coord.state_to_coord(state), value);
    }

    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[1,0,3,2,4,5,6,7,8,9,10,11], 1)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 369_599)]
    fn test_triple_centers_to_coord(state: &[u8], value: u32) {
        let coord = Coordinate::TripleCentre;
        assert_eq!(coord.state_to_coord(state), value);
    }

    #[test_case(&[0,1,2,3,4,5], 0)]
    #[test_case(&[2,0,1,3,4,5], 1)]
    #[test_case(&[4,5,3,2,1,0], 359)]
    fn test_invert_corner_perm_coord(state: &[u8], value: u32) {
        let coord = Coordinate::CornerPermutation;
        assert_eq!(coord.coord_to_state(value), state);
    }

    #[test_case(&[0b00000000], 0)]
    #[test_case(&[0b00000001], 1)]
    #[test_case(&[0b00011111], 31)]
    fn test_invert_corner_orientation_coord(state: &[u8], value: u32) {
        let coord = Coordinate::CornerOrientation;
        assert_eq!(coord.coord_to_state(value), state);
    }

    #[test_case(&[0,0,0,3,3,3,6,6,6,9,9,9], 0)]
    #[test_case(&[0,0,3,0,3,3,6,6,6,9,9,9], 1)]
    #[test_case(&[9,9,9,6,6,6,3,3,3,0,0,0], 369_599)]
    fn test_invert_edge_face_coord(state: &[u8], value: u32) {
        let coord = Coordinate::EdgeInFace;
        assert_eq!(coord.coord_to_state(value), state);
    }

    #[test_case(&[0,1,2,0,1,2,0,1,2,0,1,2], 0)]
    #[test_case(&[0,0,2,0,1,2,0,1,2,1,1,2], 1)]
    #[test_case(&[2,1,0,2,1,0,2,1,0,2,1,0], 34_649)]
    fn test_invert_edge_across_face_coord(state: &[u8], value: u32) {
        let coord = Coordinate::EdgeAcrossFaces;
        assert_eq!(coord.coord_to_state(value), state);
    }
    
    #[test_case(&[0,0,0,3,3,3,6,6,6,9,9,9], 0)]
    #[test_case(&[0,0,3,0,3,3,6,6,6,9,9,9], 1)]
    #[test_case(&[9,9,9,6,6,6,3,3,3,0,0,0], 369_599)]
    fn test_invert_up_centers_coord(state: &[u8], value: u32) {
        let coord = Coordinate::UpCentre;
        assert_eq!(coord.coord_to_state(value), state);
    }

    #[test_case(&[0,0,0,3,3,3,6,6,6,9,9,9], 0)]
    #[test_case(&[0,0,3,0,3,3,6,6,6,9,9,9], 1)]
    #[test_case(&[9,9,9,6,6,6,3,3,3,0,0,0], 369_599)]
    fn test_invert_triple_centers_coord(state: &[u8], value: u32) {
        let coord = Coordinate::TripleCentre;
        assert_eq!(coord.coord_to_state(value), state);
    }
    
    #[test_case(&[0,1,2,3,4,5], 0)]
    #[test_case(&[2,0,1,3,4,5], 1)]
    #[test_case(&[4,5,3,2,1,0], 359)]

    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[2,0,1,3,4,5,6,7,8,9,10,11], 1)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 239_500_799)]
    fn test_permutation_to_coord(positions: &[u8], expected: u32) {
        assert_eq!(permutation_to_coord(positions), expected);
    }
    
    #[test_case(0, &[0,1,2,3,4,5])]
    #[test_case(1, &[2,0,1,3,4,5])]
    #[test_case(359, &[4,5,3,2,1,0])]
    fn test_invert_corner_coord_to_permutation(coord: u32, expected: &[u8]) {
        assert_eq!(invert_coord_to_permutation::<NUM_CORNERS>(coord), expected);
    }

    #[test_case(0, &[0,1,2,3,4,5,6,7,8,9,10,11])]
    #[test_case(1, &[2,0,1,3,4,5,6,7,8,9,10,11])]
    #[test_case(239_500_799, &[11,10,9,8,7,6,5,4,3,2,1,0])]
    fn test_invert_edge_coord_to_permutation_even_parity(coord: u32, expected: &[u8]) {
        assert_eq!(invert_coord_to_permutation::<NUM_EDGES>(coord), expected);
    }

    #[test_case(&[0,1,2], true)]
    #[test_case(&[0,2,1], false)]
    #[test_case(&[0,1,2,3,4,5], true)]
    #[test_case(&[5,4,3,2,1,0], false)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], true)]    
    fn test_is_even_parity(arr: &[u8], expected: bool) {
        assert_eq!(is_even_parity(arr), expected);
    }

    #[test_case(&[0,0,0,3,3,3,6,6,6,9,9,9], 0)]
    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[0,0,3,0,3,3,6,6,6,9,9,9], 1)]
    #[test_case(&[0,3,6,9,0,3,6,9,0,3,6,9], 50_705)]
    #[test_case(&[9,9,9,6,6,6,3,3,3,0,0,0], 369_599)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 369_599)]
    fn test_face_position_to_coord(positions: &[u8], expected: u32) {
        assert_eq!(face_position_to_coord(positions), expected);
    }

    #[test_case(0, &[0,0,0,3,3,3,6,6,6,9,9,9])]
    #[test_case(1, &[0,0,3,0,3,3,6,6,6,9,9,9])]
    #[test_case(50_705, &[0,3,6,9,0,3,6,9,0,3,6,9])]
    #[test_case(369_599, &[9,9,9,6,6,6,3,3,3,0,0,0])]
    fn test_invert_coord_to_face_positions(coord: u32, expected: &[u8]) {
        assert_eq!(invert_coord_to_face_positions(coord), expected);
    }

    #[test_case(&[0,1,2,3,4,5,6,7,8,9,10,11], 0)]
    #[test_case(&[3,1,2,6,4,5,0,7,8,9,10,11], 0)]
    #[test_case(&[9,0,2,3,4,5,6,7,8,1,10,11], 1)]
    #[test_case(&[6,0,2,3,4,5,1,7,8,9,10,11], 2)]
    #[test_case(&[11,10,9,8,7,6,5,4,3,2,1,0], 34_649)]
    fn test_perm_across_face_coord(state: &[u8], expected: u32) {
        assert_eq!(perm_across_face_coord(state), expected);
    }
    
    #[test_case(0, &[0,1,2,0,1,2,0,1,2,0,1,2])]
    #[test_case(1, &[0,0,2,0,1,2,0,1,2,1,1,2])]
    #[test_case(2, &[0,0,2,0,1,2,1,1,2,0,1,2])]
    #[test_case(34_649, &[2,1,0,2,1,0,2,1,0,2,1,0])]
    fn test_invert_coord_to_perm_across_face(coord: u32, expected: &[u8]) {
        assert_eq!(invert_coord_to_perm_across_face(coord), expected);
    }

    #[test_case(0, 3, 6, 3, &[3,3,3,255,255,255])]
    #[test_case(1, 3, 6, 3, &[3,3,255,3,255,255])]
    #[test_case(0, 4, 8, 0, &[0,0,0,0,255,255,255,255])]
    #[test_case(1, 4, 8, 0, &[0,0,0,255,0,255,255,255])]
    #[test_case(5, 3, 6, 3, &[3,255,3,255,3,255])]
    #[test_case(19, 3, 6, 3, &[255,255,255,3,3,3])]
    #[test_case(0, 3, 9, 6, &[6,6,6,255,255,255,255,255,255])]
    #[test_case(15, 3, 9, 6, &[6,255,255,6,255,255,6,255,255])]
    #[test_case(83, 3, 9, 6, &[255,255,255,255,255,255,6,6,6])]
    #[test_case(0, 3, 12, 9, &[9,9,9,255,255,255,255,255,255,255,255,255,])]
    #[test_case(1, 3, 12, 9, &[9,9,255,9,255,255,255,255,255,255,255,255])]
    #[test_case(30, 3, 12, 9, &[9,255,255,255,9,255,255,255,9,255,255,255])]
    #[test_case(219, 3, 12, 9, &[255,255,255,255,255,255,255,255,255,9,9,9])]
    fn test_invert_single_face_centre_coord(coord: u32, num: u8, size: u8, fill: u8, expected: &[u8]) {
        assert_eq!(invert_single_face_centre_coord(coord, num, size, fill), expected);
    }

    #[test]
    fn test_precompute_binomial_table() {
        let binomial_table = precompute_binomial_table();
        assert_eq!(binomial_table[0][0], 1);
        assert_eq!(binomial_table[1][0], 1);
        assert_eq!(binomial_table[1][1], 1);
        assert_eq!(binomial_table[2][0], 1);
        assert_eq!(binomial_table[2][1], 2);
        assert_eq!(binomial_table[2][2], 1);
        assert_eq!(binomial_table[3][0], 1);
        assert_eq!(binomial_table[3][1], 3);
        assert_eq!(binomial_table[3][2], 3);
        assert_eq!(binomial_table[3][3], 1);
        assert_eq!(binomial_table[4][0], 1);
        assert_eq!(binomial_table[4][1], 4);
        assert_eq!(binomial_table[4][2], 6);
        assert_eq!(binomial_table[4][3], 4);
        assert_eq!(binomial_table[4][4], 1);
        assert_eq!(binomial_table[11][7], 330);
    }
}