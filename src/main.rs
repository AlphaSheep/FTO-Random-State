use std::time::Instant;

mod drawstate;
mod movedefs;
mod coordinates;
mod state;
mod movetables;
mod pruningtables;

use crate::movedefs::{Face, Turn};
use crate::state::{CoordState, RawState};
use crate::movetables::MoveTables;


fn main() {

    let now = Instant::now();
    println!("Loading move tables");

    let move_tables = MoveTables::try_load_or_generate();

    println!("Total time taken: {} seconds", (now.elapsed().as_micros() as f64 / 1_000_000.0));

    let mut raw = RawState::solved();
    let mut coords = CoordState::solved();

    let sequence = [
        &Turn::new(Face::R, false),
        &Turn::new(Face::L, true),
        &Turn::new(Face::U, false),
        &Turn::new(Face::BR, true),
        &Turn::new(Face::B, false),
        &Turn::new(Face::U, false),
        &Turn::new(Face::D, true),
        &Turn::new(Face::R, false),
        &Turn::new(Face::BL, true),
    ];

    raw.apply_sequence(&sequence);
    coords.apply_sequence(&move_tables, &sequence);

    let svg_data_coords = drawstate::get_svg_for_state(&coords.to_raw());
    drawstate::write_svg("coords.svg", &svg_data_coords);
    let svg_data_raw = drawstate::get_svg_for_state(&raw);
    drawstate::write_svg("raw.svg", &svg_data_raw);

    println!("Coords: {:?}", coords.to_raw());
    println!("Raw     {:?}", raw);
}
