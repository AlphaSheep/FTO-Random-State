use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write, BufReader, Read};

use crate::coordinates::Coordinate;
use crate::movedefs::{RawTurn, Face, TurnEffectType, Turn};
use crate::state::{apply_raw_permutation, apply_full_corner};


const MOVE_TABLE_FILE: &str = "./movetables.dat";


pub trait ApplyMove {
    fn apply_move_to_coord(&self, coord: u32, coord_type: Coordinate, turn: &Turn) -> u32; 
}

pub trait SubTables {
    fn get_sub_table<'a, T: ApplyMove>(&'a self, coord_type: &Coordinate) -> &'a T;
}

pub struct MoveTables {
    pub tables: HashMap<Coordinate, MoveTable>,
}

pub struct MoveTable {
    initialised: bool,
    populated: bool,

    pub coord_type: Coordinate,

    pub table: HashMap<Face, Vec<u32>>,
    pub inverse_table: HashMap<Face, Vec<u32>>,
}

impl MoveTables {
    pub fn try_load_or_generate() -> Self {
        match File::open(MOVE_TABLE_FILE) {
            Ok(file) => Self::load(file),
            _ => {
                let move_tables = Self::generate();
                move_tables.save();
                move_tables
            }
        }
    }

    fn generate() -> Self {
        let mut tables: HashMap<Coordinate, MoveTable> = HashMap::new();

        for coord in Coordinate::iter() {
            let move_table = MoveTable::new(coord);
            tables.insert(coord, move_table);
        }

        Self {
            tables
        }
    }

    fn save(&self) {
        let file = File::create(MOVE_TABLE_FILE).expect("Should have created the file");
        let mut writer = BufWriter::new(file);

        for (coord, table) in self.tables.iter() {
            writer.write_all(&[0,0,0,coord.to_byte()])
                .expect("Coordinate type should be written");

            table.save(&mut writer);

            writer.write_all(&[0,0,0,0])
                .expect("End of table should be written");
        }
        
        writer.write_all(&[0,0,0,0])
        .expect("End of move_table file should be written");
    }
    
    fn load(file: File) -> Self {        
        let mut reader = BufReader::new(file);

        let mut result = Self { tables: HashMap::new() };

        loop {
            let coord_byte = read_next_num(&mut reader) as u8;
            
            if coord_byte == 0 {
                break
            }
            let coord = Coordinate::from_byte(coord_byte);

            let table = MoveTable::read_from_buffer(&mut reader, coord);
            result.tables.insert(coord, table);
        }

        result
    }
}

impl ApplyMove for MoveTables {
    fn apply_move_to_coord(&self, coord: u32, coord_type: Coordinate, turn: &Turn) -> u32 {
        let table = self.tables.get(&coord_type).unwrap();
        table.apply_move_to_coord(coord, coord_type, turn)
    }
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

    pub fn save(&self, writer: &mut BufWriter<File>) {
        for (face, values) in self.table.iter() {
            writer.write_all(&[0,0,0,face.to_byte()])
                .expect("Face should be written");

            for value in values.iter() {
                writer.write_all(&value.to_be_bytes())
                    .expect("Value should be written");
            }
        }
    }

    pub fn read_from_buffer(reader: &mut BufReader<File>, coord_type: Coordinate) -> Self {
        let mut result = Self::empty(coord_type);
        result.init();

        let num_values = coord_type.get_size();

        loop {
            let face_byte = read_next_num(reader) as u8;
            if face_byte == 0 {
                break
            }
            let face = Face::from_byte(face_byte);            

            let table = result.table.get_mut(&face).unwrap();
            let inv_table = result.inverse_table.get_mut(&face).unwrap();

            for coord in 0..num_values {
                let value = read_next_num(reader);
                table[coord] = value;
                inv_table[value as usize] = coord as u32;
            }
        }
        result.populated = true;
        result
    }
}

impl ApplyMove for MoveTable {
    fn apply_move_to_coord(&self, coord: u32, _coord_type: Coordinate, turn: &Turn) -> u32 {
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

fn read_next_num(buf: &mut BufReader<File>) -> u32 {
    let mut data = [0; 4];
    buf.read_exact(&mut data)
        .expect("Should have read the data from the buffer");
    u32::from_be_bytes(data)
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
        let coord_type = Coordinate::CornerState; 
        let move_table = MoveTable::new(coord_type);

        let start_coord: u32 = 0;
        let end_coord: u32 = 3327;
        let coord = move_table.apply_move_to_coord(start_coord, coord_type, &Turn::new(Face::F, false));
        assert_eq!(coord, end_coord);

        let coord = move_table.apply_move_to_coord(end_coord, coord_type, &Turn::new(Face::F, true));
        assert_eq!(coord, start_coord);
    }
}
