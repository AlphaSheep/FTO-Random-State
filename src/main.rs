use core::time;
use std::time::Instant;
use std::thread::sleep;

mod drawstate;
mod movedefs;
mod coordinates;
mod state;
mod movetables;
mod pruningtables;
mod search;

use crate::movedefs::{Face, Turn};
use crate::pruningtables::SimplePruningTable;
use crate::search::search_phase_1;
use crate::state::{CoordState, RawState};
use crate::movetables::MoveTables;


fn main() {

    let now = Instant::now();
    println!("Loading move tables");
    let move_tables = MoveTables::try_load_or_generate();
    println!("Total time taken: {} seconds", (now.elapsed().as_micros() as f64 / 1_000_000.0));

    let now = Instant::now();
    println!("Loading pruning tables");
    let mut pruning_tables = SimplePruningTable::init(&Face::get_all_faces());
    pruning_tables.populate(&move_tables);
    println!("Total time taken: {} seconds", (now.elapsed().as_micros() as f64 / 1_000_000.0));

    // let random = CoordState::get_random();
    let mut random = CoordState::solved();
    random.apply_sequence(&move_tables, &[
        &Turn::new(Face::R, false),
        &Turn::new(Face::U, false),
        &Turn::new(Face::R, false),
        &Turn::new(Face::U, false),
    ]);

    // let svg_data_random = drawstate::get_svg_for_state(&random.to_raw());
    // drawstate::write_svg("random_start.svg", &svg_data_random);

    let now = Instant::now();

    let limit = 9;
    println!("Searching to depth {}.", limit);

    let solution = search_phase_1(&random, &move_tables, &pruning_tables, limit, None);
    println!("{:?}", solution);

    println!("Total time taken: {} seconds", (now.elapsed().as_micros() as f64 / 1_000_000.0));

    sleep(time::Duration::from_secs(1));
    println!("Done")
}
