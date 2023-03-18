use std::collections::HashMap;

use crate::coordinates::Coordinate;
use crate::movedefs::{RawTurn, Face, TurnEffectType, Turn};
use crate::state::{apply_raw_permutation, apply_full_corner};


pub struct MoveTables {
    pub tables: HashMap<Coordinate, MoveTable>,
}

impl MoveTables {
    pub fn generate() -> Self {
        let mut tables: HashMap<Coordinate, MoveTable> = HashMap::new();

        for coord in Coordinate::iter() {
            let move_table = MoveTable::new(coord);
            tables.insert(coord, move_table);
        }

        Self {
            tables
        }
    }

    pub fn apply_move_to_coord(&self, coord: u32, coord_type: Coordinate, turn: &Turn) -> u32 {
        let table = self.tables.get(&coord_type).unwrap();
        table.apply_move_to_coord(coord, turn)
    }
}

pub struct MoveTable {
    initialised: bool,
    populated: bool,

    coord_type: Coordinate,

    pub table: HashMap<Face, Vec<u32>>,
    pub inverse_table: HashMap<Face, Vec<u32>>,
}

impl MoveTable {
    fn empty(coord_type: Coordinate) -> Self {
        Self {
            initialised: false,
            populated: false,

            coord_type,

             table: HashMap::new(),        
            inverse_table: HashMap::new(), 
        }
    }

    pub fn new(coord_type: Coordinate) -> Self {
        let mut move_table = Self::empty(coord_type);
        move_table.init();
        move_table.populate();
        move_table
    }


    pub fn init(&mut self) {
        for face in Face::get_all_faces() {
            self.table.insert(face, vec![u32::MAX; self.coord_type.get_size()]);
            self.inverse_table.insert(face, vec![u32::MAX; self.coord_type.get_size()]);
        }
        self.initialised = true;
    }

    pub fn populate(&mut self) {

        let coord_type = self.coord_type;

        for start_coord in 0..(coord_type.get_size() as u32) {
            let mut state = coord_type.coord_to_state(start_coord);
            for face in Face::get_all_faces() {
                if self.table[&face][start_coord as usize] < u32::MAX {
                    continue;
                }

                let turn = RawTurn::get(face);
                
                let mut cycle = [start_coord, 0, 0];

                apply_turn_to_state(&mut state, turn, coord_type.get_turn_effect_type());
                cycle[1] = coord_type.state_to_coord(&state);
                apply_turn_to_state(&mut state, turn, coord_type.get_turn_effect_type());
                cycle[2] = coord_type.state_to_coord(&state);
                apply_turn_to_state(&mut state, turn, coord_type.get_turn_effect_type());

                let table = self.table.get_mut(&face).unwrap();
                let inv_table = self.inverse_table.get_mut(&face).unwrap();

                add_cycle_to_table(table, inv_table, &cycle);
            }
        }
        self.populated = true;
    }

    pub fn apply_move_to_coord(&self, coord: u32, turn: &Turn) -> u32 {
        let move_table = if turn.invert {
            self.inverse_table.get(&turn.face).unwrap()
        } else {
            self.table.get(&turn.face).unwrap()
        };
        move_table[coord as usize]
    }
}

fn add_cycle_to_table(table: &mut [u32], inv_table: &mut [u32], cycle: &[u32]) {
    table[cycle[0] as usize] = cycle[1];
    table[cycle[1] as usize] = cycle[2];
    table[cycle[2] as usize] = cycle[0];
    inv_table[cycle[0] as usize] = cycle[2];
    inv_table[cycle[2] as usize] = cycle[1];
    inv_table[cycle[1] as usize] = cycle[0];
}

fn apply_turn_to_state(state: &mut [u8], turn: &RawTurn, effect_type: TurnEffectType) {
    match effect_type {
        TurnEffectType::Corner => {
            let perm_effect = turn.get_effect(TurnEffectType::CornerPermutation);
            let orient_effect = turn.get_effect(TurnEffectType::CornerOrientation);
            apply_full_corner(state, perm_effect.as_ref(), orient_effect.as_ref());
        }
        _ => {
            let effect = turn.get_effect(effect_type);
            apply_raw_permutation(state, effect.as_ref());
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_move_to_edge_state() {
        // A turn should apply a raw permutation to edges
        let mut state = [0,1,2,3,4,5,6,7,8,9,10,11];
        apply_turn_to_state(&mut state, Face::F.turn(), TurnEffectType::EdgeInFace);
        assert_eq!(&state, &[0,1,2,3,4,5,6,7,8,11,9,10]);
    }

    #[test]
    fn test_apply_move_to_corner_state() {
        // A turn should apply permutation and  orientation to corners
        let mut state = [0,2,4,6,8,10];
        apply_turn_to_state(&mut state, Face::F.turn(), TurnEffectType::Corner);
        assert_eq!(&state, &[0,2,11,6,4,9]);
    }

    #[test]
    fn test_move_tables() {
        // Because generating the table is slow, we do it once then do all the checks we need to
        // We choose corner state because it needs the shortest time to generate
        let coord = Coordinate::CornerState; 
        let move_table = MoveTable::new(coord);

        let start_coord: u32 = 0;
        let end_coord: u32 = 3327;
        let coord = move_table.apply_move_to_coord(start_coord, &Turn::new(Face::F, false));
        assert_eq!(coord, end_coord);

        let coord = move_table.apply_move_to_coord(end_coord, &Turn::new(Face::F, true));
        assert_eq!(coord, start_coord);
    }
}