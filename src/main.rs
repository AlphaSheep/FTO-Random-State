mod drawstate;
mod movedefs;
mod state;

use crate::movedefs::Turn;
use crate::state::RawState;

fn main() {

    let mut state = RawState::solved();

    state.apply(Turn::get("R"));
    // state.apply(Turn::get("L"));
    // state.apply(Turn::get("F"));
    // state.apply(Turn::get("R"));
    // state.apply(Turn::get("BL"));
    // state.apply(Turn::get("B"));
    // state.apply(Turn::get("BR"));
    // state.apply(Turn::get("D"));
    
    let svg_data = drawstate::get_svg_for_state(&state);
    drawstate::write_svg(&"test.svg", &svg_data);
}
