use std::time::Instant;

mod drawstate;
mod movedefs;
mod coordinates;
mod state;
mod movetables;

use crate::coordinates::Coordinate;
use crate::movedefs::Face;
use crate::state::RawState;
use crate::movetables::MoveTable;


fn main() {

    let now = Instant::now();
    for coord in Coordinate::iter() {
        let now = Instant::now();

        println!("Generating move tables for {:?}...", coord);

        let mut move_table = MoveTable::new(coord);
        move_table.init();
        move_table.populate();

        println!("Done. Time taken: {} seconds", (now.elapsed().as_micros() as f64 / 1_000_000.0));
    }
    println!("Total time taken: {} seconds", (now.elapsed().as_micros() as f64 / 1_000_000.0));

    let mut state = RawState::solved();

    state.apply(Face::U.turn());
    state.apply(Face::L.turn());
    state.apply(Face::F.turn());
    state.apply(Face::R.turn());
    state.apply(Face::BL.turn());
    state.apply(Face::B.turn());
    state.apply(Face::BR.turn());
    state.apply(Face::D.turn());
    
    let svg_data = drawstate::get_svg_for_state(&state);
    drawstate::write_svg("test.svg", &svg_data);
}
