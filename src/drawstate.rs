
/*
Corners:            UBL    UBR    UF    DB    DR    DL
Edges:                UB    UR    UL    BLB    BLL    BLD    BRR    BRB    BRD    FL    FR    FD
Up Centres:     UBL    UBR    UF    BLU    BLF    BLBR    BRU    BRBL    BRF    FU    FBR    FBL
Down Centres: BR    BL    BD    RL    RB    RD    LB    LR    LD    DL    DR    DB
*/

use std::fs;
use crate::state::{RawState, apply_raw_permutation, flip_num_to_bool_array};


const SVG_TEMPLATE_FILE: &str = "./assets/fto.svg";

const COLOURS: &[&str] = &[
    "#fff",
    "#f00",
    "#f80",
    "#888",
    "#ff0",
    "#00f",
    "#808",
    "#080",
];

const U: u8 = 0;
const F: u8 = 1;
const BL: u8 = 2;
const BR: u8 = 3;
const D: u8 = 4;
const B: u8 = 5;
const L: u8 = 6;
const R: u8 = 7;

const CORNER_NAMES_UP_GOOD: &[&str] = &[
    "corn-UBL-U",
    "corn-UBR-U",
    "corn-UF-U",
    "corn-DB-BL",
    "corn-DR-BR",
    "corn-DL-F",
];
const CORNER_NAMES_UP_FLIPPED: &[&str] = &[
    "corn-UBL-BL",
    "corn-UBR-BR",
    "corn-UF-F",
    "corn-DB-BR",
    "corn-DR-F",
    "corn-DL-BL",
];
const CORNER_NAMES_DOWN_GOOD: &[&str] = &[
    "corn-UBL-L",
    "corn-UBR-B",
    "corn-UF-R",
    "corn-DB-D",
    "corn-DR-D",
    "corn-DL-D",
];
const CORNER_NAMES_DOWN_FLIPPED: &[&str] = &[
    "corn-UBL-B",
    "corn-UBR-R",
    "corn-UF-L",
    "corn-DB-B",
    "corn-DR-R",
    "corn-DL-L",
];
const EDGE_UP_NAMES: &[&str] = &[
    "edge-UB-U",
    "edge-UR-U",
    "edge-UL-U",
    "edge-BLB-BL",
    "edge-BLL-BL",
    "edge-BLD-BL",
    "edge-BRR-BR",
    "edge-BRB-BR",
    "edge-BRD-BR",
    "edge-FL-F",
    "edge-FR-F",
    "edge-FD-F",
];
const EDGE_DOWN_NAMES: &[&str] = &[
    "edge-UB-B",
    "edge-UR-R",
    "edge-UL-L",
    "edge-BLB-B",
    "edge-BLL-L",
    "edge-BLD-D",
    "edge-BRR-R",
    "edge-BRB-B",
    "edge-BRD-D",
    "edge-FL-L",
    "edge-FR-R",
    "edge-FD-D",
];
const UP_CENTRE_NAMES: &[&str] = &[
    "cent-UBL",
    "cent-UBR",
    "cent-UF",
    "cent-BLU",
    "cent-BLF",
    "cent-BLBR",
    "cent-BRU",
    "cent-BRBL",
    "cent-BRF",
    "cent-FU",
    "cent-FBR",
    "cent-FBL",
];
const DOWN_CENTRE_NAMES: &[&str] = &[
    "cent-BR",
    "cent-BL",
    "cent-BD",
    "cent-RL",
    "cent-RB",
    "cent-RD",
    "cent-LB",
    "cent-LR",
    "cent-LD",
    "cent-DL",
    "cent-DR",
    "cent-DB",
];

const STYLE_PLACEHOLDER: &str = "<!--*style placeholder-->";


struct StickerState {
    corner_up_good: [u8; 6],
    corner_up_flipped: [u8; 6],
    corner_down_good: [u8; 6],
    corner_down_flipped: [u8; 6],
    edge_up: [u8; 12],
    edge_down: [u8; 12],
    up_centres: [u8; 12],
    down_centres: [u8; 12],
}

impl StickerState {
    pub fn get_initial() -> Self {
        Self {
            corner_up_good: [U,U,U,BL,BR,F],
            corner_up_flipped: [BL,BR,F,BR,F,BL],
            corner_down_good: [L,B,R,D,D,D],
            corner_down_flipped: [B,R,L,B,R,L],
            edge_up: [U,U,U,BL,BL,BL,BR,BR,BR,F,F,F],
            edge_down: [B,R,L,B,L,D,R,B,D,L,R,D],
            up_centres: [U,U,U,BL,BL,BL,BR,BR,BR,F,F,F],
            down_centres: [B,B,B,R,R,R,L,L,L,D,D,D],
        }
    }

    pub fn create_from_raw_state(state: &RawState) -> Self {
        let mut stickers = StickerState::get_initial();

        apply_raw_permutation(&mut stickers.corner_up_good, &state.corners);
        apply_raw_permutation(&mut stickers.corner_up_flipped, &state.corners);
        apply_raw_permutation(&mut stickers.corner_down_good, &state.corners);
        apply_raw_permutation(&mut stickers.corner_down_flipped, &state.corners);
        apply_sticker_orientation(&mut stickers.corner_up_good, &mut stickers.corner_up_flipped, &state.corner_orientation);
        apply_sticker_orientation(&mut stickers.corner_down_good, &mut stickers.corner_down_flipped, &state.corner_orientation);

        apply_raw_permutation(&mut stickers.edge_up, &state.edges);
        apply_raw_permutation(&mut stickers.edge_down, &state.edges);

        apply_raw_permutation(&mut stickers.up_centres, &state.up_centres);
        apply_raw_permutation(&mut stickers.down_centres, &state.down_centres);

        stickers
    }
}


pub fn get_svg_for_state(state: &RawState) -> String {
    let template = get_svg_template();
    let styles = get_style_section(&state);
    template.replace(STYLE_PLACEHOLDER, &styles)
}

fn apply_sticker_orientation(good_stickers: &mut [u8], flipped_stickers: &mut [u8], effect: &u8) {
    let flip = flip_num_to_bool_array(effect);

    for i in 0..flip.len() {
        if flip[i] {
            std::mem::swap(&mut good_stickers[i], &mut flipped_stickers[i]);
        }
    }
}

fn get_svg_template() -> String {
    fs::read_to_string(SVG_TEMPLATE_FILE)
    .expect("should read the template file")
}

fn get_style_section(state: &RawState) -> String {
    let header: String = String::from("<style>");
    let footer: String = String::from("</style>");

    let stickers = StickerState::create_from_raw_state(&state);
    let styles = get_sticker_styles(&stickers);

    header + &styles + &footer
}

fn get_sticker_styles<'a>(stickers: &'a StickerState) -> String {
    let mut styles: String = String::from("");
    let sticker_arrays: Vec<&[u8]> = vec![
        &stickers.corner_up_good, &stickers.corner_up_flipped, &stickers.corner_down_good, &stickers.corner_down_flipped,
        &stickers.edge_up, &stickers.edge_down,
        &stickers.up_centres, &stickers.down_centres];
    let names = vec![
        CORNER_NAMES_UP_GOOD, CORNER_NAMES_UP_FLIPPED, CORNER_NAMES_DOWN_GOOD, CORNER_NAMES_DOWN_FLIPPED,
        EDGE_UP_NAMES, EDGE_DOWN_NAMES,
        UP_CENTRE_NAMES, DOWN_CENTRE_NAMES];

    for i in 0..8 {
        let new_style = get_style_for_sticker_set(sticker_arrays[i], names[i]);
        styles += &new_style;
    }

    styles
}

fn get_style_for_sticker_set<'a>(set: &'a [u8], names: &[&str]) -> String {
    let mut styles: String = String::from("");
    for i in 0..set.len() {
        let colour = COLOURS[set[i] as usize];
        let next_style: String = get_style_for_sticker(names[i], colour);
        styles += &next_style;
    }
    styles
}

fn get_style_for_sticker(name: &str, fill: &str) -> String {
    format!(".{}{{fill:{}}} ", name, fill)
}

pub fn write_svg(filename: &str, svg_data: &str) {
    fs::write(filename, svg_data)
    .expect("should write the SVG file.");
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&"class", &"#fff", &".class{fill:#fff} ")]
    fn test_get_style_for_sticker(name: &str, fill: &str, expected: &str) {
        assert_eq!(get_style_for_sticker(name, fill), expected);
    }

    #[test_case(&[0,2,5], &[&"piece-a", &"piece-b", &"piece-c"], &".piece-a{fill:#fff} .piece-b{fill:#f80} .piece-c{fill:#00f} ")]
    fn test_get_style_for_sticker_set(set: &[u8], names: &[&str], expected: &str) {
        assert_eq!(get_style_for_sticker_set(set, names), expected);
    }

    #[test]
    fn test_sticker_iniital_state() {
        let stickers = StickerState::get_initial();
        assert_eq!(stickers.corner_up_good[0], U);
    }

    #[test]
    fn test_get_style_section() {
        let style: String = get_style_section(&RawState::solved());
        let start = &style[0..7];
        assert_eq!(start, "<style>");
        let end = &style[style.len() - 8 ..];
        assert_eq!(end, "</style>");
    }

    #[test]
    fn test_get_sticker_styles_does_not_error() {
        let stickers = StickerState::get_initial();
        let styles = get_sticker_styles(&stickers);
        assert!(styles.contains(&".corn-UBL-L{fill:#808}"));
        assert!(styles.contains(&".edge-FR-R{fill:#080}"));
        assert!(styles.contains(&".cent-UF{fill:#fff}"));
    }

    #[test]
    fn test_get_template_does_not_error() {
        let _ = get_svg_template();
    }

    #[test]
    fn test_get_svg_for_state_does_not_error() {
        let svg = get_svg_for_state(&RawState::solved());
        assert!(svg.contains(&".corn-UBL-L{fill:#808}"));
        assert!(svg.contains(&".edge-FR-R{fill:#080}"));
        assert!(svg.contains(&".cent-UF{fill:#fff}"));
    }
}
