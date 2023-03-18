use crate::coordinates::CoordinateType;
use crate::movetables::MoveTables;
use crate::pruningtables::{SimplePruningTable, PruningTable};
use crate::state::{CoordState, do_triple_centres_match_corners};
use crate::movedefs::Turn;


pub fn search_phase_1(state: &CoordState, move_tables: &MoveTables, pruning_tables: &SimplePruningTable, limit: u8, prev_turn: Option<&Turn>) -> Vec<Turn> {
    if limit > 0 {
        for turn in Turn::get_all_turns().iter().rev() {
            if is_redundant_turn(prev_turn, turn) {
                continue;
            }

            // println!("  - solving {:?} limited to {:?}", turn, limit);
            let mut next_state = *state;
            next_state.apply(move_tables, turn);

            if is_phase_1_solved(&next_state) {
                return vec![*turn];
            }
            else if !should_prune_phase_1(&next_state, pruning_tables, limit) {
                let mut solution = search_phase_1(&next_state, move_tables, pruning_tables, limit - 1, Some(turn));
                if !solution.is_empty() {
                    solution.insert(0, *turn);
                    return solution
                }
            }
        }
    }
    vec![]
}

fn is_redundant_turn(prev_turn: Option<&Turn>, curr_turn: &Turn) -> bool {
    match prev_turn {
        Some(prev_turn) => {
            let prev_axis = prev_turn.face.get_primary_face();
            let curr_axis = curr_turn.face.get_primary_face();
            
            // Don't turn the same face twice
            prev_turn.face == curr_turn.face || 
            // Skip if it is the same axis, and the current face is primary (favour the secondary face in phase 1)
            (prev_axis == curr_axis && curr_turn.face == curr_axis)
        },
        None => false,
    }
}

fn should_prune_phase_1(state: &CoordState, pruning_tables: &SimplePruningTable, limit: u8) -> bool {
    let coords = [state.edges_within_faces, state.down_centres];
    let coord_types = [CoordinateType::EdgeInFace, CoordinateType::DownCentre];
    let distance = pruning_tables.get_distance_lower_bound(&coords, &coord_types);

    distance > limit
}

fn is_phase_1_solved(state: &CoordState) -> bool {
    state.edges_within_faces == 0 && 
    state.up_centres == 0 && 
    do_triple_centres_match_corners(state.corners, state.down_centres) 
}
