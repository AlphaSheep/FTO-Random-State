use std::collections::HashMap;

use crate::coordinates::{self, Coordinate};
use crate::movedefs::{Turn, Face};
use crate::state::{self, apply_raw_permutation};

pub struct MoveTable {
    initialised: bool,
    popluated: bool,

    coord_type: Coordinate,

    pub table: HashMap<Face, Vec<u32>>,
    pub inverse_table: HashMap<Face, Vec<u32>>,
}

impl MoveTable {
    pub fn new(coord_type: Coordinate) -> Self {
        Self {
            initialised: false,
            popluated: false,

            coord_type,

            table: HashMap::new(),        
            inverse_table: HashMap::new(), 
        }
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

                let turn = Turn::get(face);
                let effect = turn.get_effect(coord_type.get_turn_effect_type());

                let mut cycle = [start_coord, 0, 0];

                apply_raw_permutation(&mut state, effect);
                cycle[1] = coord_type.state_to_coord(&state);
                apply_raw_permutation(&mut state, effect);
                cycle[2] = coord_type.state_to_coord(&state);
                apply_raw_permutation(&mut state, effect);

                let table = self.table.get_mut(&face).unwrap();
                let inv_table = self.inverse_table.get_mut(&face).unwrap();

                add_cycle_to_table(table, inv_table, &cycle);
            }
        }
        self.popluated = true;
    }
}

fn add_cycle_to_table(mut table: &mut Vec<u32>, mut inv_table: &mut Vec<u32>, cycle: &[u32]) {
    
    table[cycle[0] as usize] = cycle[1];
    table[cycle[1] as usize] = cycle[2];
    table[cycle[2] as usize] = cycle[0];
    inv_table[cycle[0] as usize] = cycle[2];
    inv_table[cycle[2] as usize] = cycle[1];
    inv_table[cycle[1] as usize] = cycle[0];
}
