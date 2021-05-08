use crate::v2::{get_text, Message};
use fontdue::Font;
use itertools::Itertools;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::ops::Index;
use std::str::Lines;

type InteractionSet = Vec<Interaction>;
type ParticipantSet = Vec<Participant>;

// == Theme ===============================================

#[derive(Debug)]
struct Theme {
    title_font: Font,
    body_font: Font,
    title_font_px: f32,
    partic_font_px: f32,
    message_font_px: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title_font: Theme::load_font(include_bytes!("../../assets/Roboto-Thin.ttf") as &[u8]),
            body_font: Theme::load_font(include_bytes!("../../assets/Roboto-Thin.ttf") as &[u8]),
            title_font_px: 30.0,
            partic_font_px: 18.0,
            message_font_px: 16.0,
        }
    }
}

impl Theme {
    fn load_font(font_data: &[u8]) -> Font {
        // Parse it into the font type.
        let settings = fontdue::FontSettings {
            ..fontdue::FontSettings::default()
        };

        fontdue::Font::from_bytes(font_data, settings).unwrap()
    }
}

// == Diagram =============================================

#[derive(Debug)]
struct Diagram {
    theme: Theme,
    header: Header,
    interactions: InteractionSet,
    participants: ParticipantSet,
}

impl Diagram {
    fn parse(lines: String) -> Diagram {
        let participant_parser = ParticipantParser::default();
        let participants = participant_parser.parse(lines);

        Diagram {
            theme: Default::default(),
            header: Header {},
            interactions: vec![],
            participants: vec![],
        }
    }
}

// == Participant Parser ==================================

#[derive(Debug)]
pub struct ParticipantParser {
    interaction_regex: Regex,
    participants: HashMap<String, Participant>,
}
impl Default for ParticipantParser {
    fn default() -> Self {
        ParticipantParser {
            interaction_regex: Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap(),
            participants: Default::default(),
        }
    }
}
impl ParticipantParser {
    pub fn parse(&self, lines: String) -> HashMap<String, Participant> {
        // parse participant names + index
        self.parse_participant_names(lines.lines());

        // when is each participant active from / to?

        HashMap::new()
    }

    pub fn parse_participant_names(&self, lines: Lines) -> HashSet<String> {
        lines
            // .map(|line| line.trim())
            .filter_map(|line| {
                if line.trim().len() == 0 {
                    None
                } else if line.trim().starts_with(":") {
                    None
                } else if line.trim().starts_with("#") {
                    None
                } else {
                    Some(line.trim())
                }
            })
            // .filter(|line| !line.starts_with("#"))
            // .filter(|line| !line.starts_with(":"))
            .filter_map(|line| self.interaction_regex.captures(line))
            .filter_map(|captures| {
                if captures.len() >= 3 {
                    let from_name = captures.index(1);
                    let to_name = captures.index(2);
                    Some(vec![from_name.to_string(), to_name.to_string()])
                } else {
                    None
                }
            })
            .flatten()
            .map(|p| p.to_string())
            .collect::<HashSet<String>>()
    }
}

#[test]
fn testpp() {
    let pp = ParticipantParser::default();
    let input = "Client -> Server: Message";
    assert_eq!(2, pp.parse_participant_names(input.lines()).len());

    let input = "Client -> Server: Message
    Server -> Database: Query";
    assert_eq!(3, pp.parse_participant_names(input.lines()).len());
}

// == Interaction Parser ==================================

#[derive(Debug)]
struct InteractionParser {
    interaction_regex: Regex,
}
impl Default for InteractionParser {
    fn default() -> Self {
        InteractionParser {
            interaction_regex: Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap(),
        }
    }
}
impl InteractionParser {
    pub fn parse(&mut self, lines: Lines, participants: &mut ParticipantSet) -> InteractionSet {
        InteractionSet::new()
    }
}

// == Header ==============================================

#[derive(Debug)]
struct Header {}

// == Participant =========================================

#[derive(Debug, Clone)]
pub struct Participant {
    name: String,
    index: usize,
    active_from: i32,
    active_to: i32,
}

impl Participant {
    fn new(name: &str, index: usize) -> Self {
        Participant {
            name: name.to_string(),
            index,
            active_from: -1,
            active_to: -1,
        }
    }
}

// == Interaction Type ====================================

#[derive(Debug)]
enum InteractionType {
    L2R,
    // R2L,
    // SELF,
}

#[derive(Debug)]
struct Interaction {
    index: i32,
    from_participant: Participant,
    to_participant: Participant,
    interaction_type: InteractionType,
}

#[test]
fn testtsst() {
    let _ = Diagram::parse(get_text());
}
