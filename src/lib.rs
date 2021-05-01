pub mod parser;
pub mod render_context;
pub mod rendering;
pub mod text;

#[macro_use]
extern crate log;

use crate::parser::{InteractionParser, InteractionSet};
use crate::rendering::Theme;
use itertools::Itertools;
use std::str::Lines;

#[derive(Debug)]
pub struct Diagram {
    pub theme: Theme,
    pub unique_participants: i32,
    pub interaction_set: InteractionSet,
}

pub fn parse_diagram(lines: Lines) -> Diagram {
    let interaction_set = InteractionParser::default().parse_interactions(lines);

    let theme = Theme {
        title_font_family: "Arial".to_string(),
        _title_font_pt: 40.,
        participant_font_family: "Arial".to_string(),
        participant_font_pt: 20.,
    };

    // pre-calculate all sizes...
    let vec = interaction_set
        .iter()
        .map(|p| smallvec::SmallVec::from_buf([&p.from_participant, &p.to_participant]))
        .flatten()
        .unique()
        .collect_vec();
    println!("{:?}", vec);

    Diagram::new(theme, interaction_set)
}

pub fn get_text() -> String {
    String::from(
        ":theme Default
 :title Example Sequence Diagram
 :author Mr. Sequence Diagram
 :date

 # diagram
 Client -> Server: Request
 Server -> Server: Parses request
 Server ->> Service: Query
 Service -->> Server: Data
 Server --> Client: Response
 Left -> Right
 {AMPS} -> Client: ",
    )
}
