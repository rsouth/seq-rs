pub mod parser;
pub mod rendering;

#[macro_use]
extern crate log;

use crate::parser::{InteractionParser, InteractionSet};
use std::str::Lines;

#[derive(Debug)]
pub struct Diagram(InteractionSet);

pub fn parse_diagram(lines: Lines) -> Diagram {
    let interactions = InteractionParser::default().parse_interactions(lines);

    Diagram(interactions)
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
