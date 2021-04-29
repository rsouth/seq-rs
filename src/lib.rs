pub mod parser;
pub mod rendering;
pub mod text;

#[macro_use]
extern crate log;

use crate::parser::{InteractionParser, InteractionSet};
use std::str::Lines;

#[derive(Debug)]
pub struct Diagram {
    pub interaction_set: InteractionSet,
}
pub fn parse_diagram(lines: Lines) -> Diagram {
    let interaction_set = InteractionParser::default().parse_interactions(lines);

    Diagram { interaction_set }
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
