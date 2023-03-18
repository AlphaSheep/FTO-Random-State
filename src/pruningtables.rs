use std::collections::HashMap;

use crate::coordinates::CoordinateType;
use crate::movedefs::{Face, Turn};
use crate::movetables::{MoveTables, MoveTable, ApplyMove};


pub trait PruningTable {
    fn get_distance_lower_bound(&self, coords: &[u32], coord_types: &[CoordinateType]) -> u8;
}

pub struct SimplePruningTable {
    tables: HashMap<CoordinateType, Vec<u8>>,
    faces: Vec<Face>,
}

pub struct CompoundPruningTable {
    tables: HashMap<CoordinateType, Vec<u8>>,
    faces: Vec<Face>,
}

impl SimplePruningTable {
    pub fn init(faces: &[Face]) -> Self {
        Self {
            tables: HashMap::new(),
            faces: faces.to_vec(),
        }
    }

    fn get_allowed_turns(&self) -> Vec<Turn> {
        let mut turns = Vec::with_capacity(self.faces.len() * 2);
        for face in &self.faces {
            turns.push(Turn::new(*face, true));
            turns.push(Turn::new(*face, false));
        }
        turns
    }

    pub fn populate(&mut self, move_tables: &MoveTables) {
        
        for coord_type in CoordinateType::iter() {
            let move_table = move_tables.tables.get(&coord_type).unwrap();
            self.populate_coordinate(move_table, coord_type);
        }
    }

    fn populate_coordinate(&mut self, move_table: &MoveTable, coord_type: CoordinateType) {
        let num_coords = coord_type.get_size();

        let mut table: Vec<u8> = Vec::with_capacity(num_coords);
        for i in 0..coord_type.get_size() {
            table.push(u8::MAX);
        }

        let mut distance: u8 = 1;
        table[0] = 0;

        let mut remaining = num_coords - 1;
        let forward_stop_point = num_coords / 3;

        self.forward_fill_table(&mut table, move_table, &mut distance, &mut remaining, forward_stop_point);
        self.backward_fill_table(&mut table, move_table, &mut distance, &mut remaining);
        
        // println!("{:?} pruning table max depth = {:?}", coord_type, distance-1);

        self.tables.insert(coord_type, table);
    }

    fn forward_fill_table(&self, table: &mut Vec<u8>, move_table: &MoveTable, distance: &mut u8, remaining: &mut usize, forward_stop_point: usize) {
        let mut previous_fill_list: Vec<usize> = vec![0];
        let allowed_turns = self.get_allowed_turns();
        while *remaining > 0 && previous_fill_list.len() < forward_stop_point {
            // println!("  - Forward filling {:?} pruning table for distance {:?}. Checking {:?} coords. ({:?} remaining)", 
            //     move_table.coord_type, distance, previous_fill_list.len(), remaining);
            let mut next_fill_list: Vec<usize> = vec![];
            for coord in previous_fill_list {
                for turn in &allowed_turns {
                    let next_coord = move_table.apply_move_to_coord(coord as u32, move_table.coord_type, &turn) as usize;
                    if table[next_coord] == u8::MAX {
                        table[next_coord] = *distance;
                        next_fill_list.push(next_coord);
                        *remaining -= 1;
                    }
                }
            }
            previous_fill_list = next_fill_list;
            *distance += 1;                
        }
    }

    fn backward_fill_table(&self, table: &mut Vec<u8>, move_table: &MoveTable, distance: &mut u8, remaining: &mut usize) {
        let allowed_turns = self.get_allowed_turns();
        while *remaining > 0 {
            // println!("  - Backward filling {:?} pruning table for distance {:?}. Checking {:?} coords. ({:?} remaining)", 
            //     move_table.coord_type, distance, remaining, remaining);

            let num_coords = move_table.coord_type.get_size();
            
            for coord in 0..num_coords {
                if table[coord] == u8::MAX {
                    for turn in &allowed_turns {
                        let next_coord = move_table.apply_move_to_coord(coord as u32, move_table.coord_type, &turn) as usize;
                        if table[next_coord] == *distance - 1 {
                            table[coord] = *distance;
                            *remaining -= 1;
                            break;
                        }
                    }
                    if *remaining == 0 {
                        break
                    }
                }
            }
            *distance += 1;
        }
    }

}

impl PruningTable for SimplePruningTable {
    fn get_distance_lower_bound(&self, coords: &[u32], coord_types: &[CoordinateType]) -> u8 {
        let mut distance = 0;
        for i in 0..coords.len() {
            let coord = coords[i];
            let coord_type = coord_types[i];
            let lookup = self.tables.get(&coord_type).unwrap();
            let new_distance = lookup[coord as usize];
            if new_distance > distance {
                distance = new_distance;
            }
        }
        distance
    }
}

impl CompoundPruningTable {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pruning_tables() {
        let coord_type = CoordinateType::CornerState; 
        let move_table = MoveTable::new(coord_type);

        let mut pruning_table = SimplePruningTable::init(&Face::get_up_faces());
        pruning_table.populate_coordinate(&move_table, coord_type);


    }
}
