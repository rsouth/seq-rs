use fontdue::Font;
use itertools::Itertools;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::ops::{AddAssign, Index};
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

type InteractionSet = Vec<Interaction>;
type ParticipantSet = Vec<Participant>;

// == Message =============================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Message(pub String);

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
    fn parse(_document: &[Line]) -> Diagram {
        // let participant_parser = ParticipantParser::default();
        // let _participants = participant_parser.parse(lines);

        println!("Document: {:?}", _document);

        Diagram {
            theme: Default::default(),
            header: Header {},
            interactions: vec![],
            participants: vec![],
        }
    }
}

pub struct DocumentParser {
    interaction_regex: Regex,
}
impl Default for DocumentParser {
    fn default() -> Self {
        DocumentParser {
            interaction_regex: Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap(),
        }
    }
}
impl DocumentParser {
    pub fn parse(&self, line: &str) -> Vec<Line> {
        let line_number = AtomicU32::new(0);
        line.lines()
            .into_iter()
            .map(|line| line.trim())
            .map(|line| {
                if line.is_empty() {
                    Line {
                        line_number: line_number.fetch_add(1, Ordering::Relaxed),
                        line_contents: LineContents::Nothing,
                        line_data: line.to_string(),
                    }
                } else {
                    match self.interaction_regex.captures(line) {
                        None => Line {
                            line_number: line_number.fetch_add(1, Ordering::Relaxed),
                            line_contents: LineContents::Invalid,
                            line_data: line.to_string(),
                        },
                        Some(captures) => {
                            if captures.len() >= 3 && !captures.index(3).is_empty() {
                                Line {
                                    line_number: line_number.fetch_add(1, Ordering::Relaxed),
                                    line_contents: LineContents::InteractionWithMessage,
                                    line_data: line.to_string(),
                                }
                            } else {
                                Line {
                                    line_number: line_number.fetch_add(1, Ordering::Relaxed),
                                    line_contents: LineContents::Interaction,
                                    line_data: line.to_string(),
                                }
                            }
                        }
                    }
                }
            })
            .collect_vec()
    }
}

#[derive(Debug)]
pub struct Line {
    line_number: u32,
    line_contents: LineContents,
    line_data: String,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum LineContents {
    Invalid,
    Nothing,
    Comment,
    MetaData,
    Interaction,
    InteractionWithMessage,
}

#[test]
fn test_document_parser() {
    let text = "
    Client -> Server: Message
    Server -> Database
    Database -> Server: Response";
    let parser = DocumentParser::default();
    let vec = parser.parse(text);
    assert_eq!(4, vec.len());

    // line 0
    assert_eq!(0, vec[0].line_number);
    assert_eq!(LineContents::Nothing, vec[0].line_contents);
    assert_eq!("", vec[0].line_data);

    // line 1
    assert_eq!(1, vec[1].line_number);
    assert_eq!(LineContents::InteractionWithMessage, vec[1].line_contents);
    assert_eq!("Client -> Server: Message", vec[1].line_data);

    // line 2
    assert_eq!(2, vec[2].line_number);
    assert_eq!(LineContents::Interaction, vec[2].line_contents);
    assert_eq!("Server -> Database", vec[2].line_data);

    // line 3
    assert_eq!(3, vec[3].line_number);
    assert_eq!(LineContents::InteractionWithMessage, vec[3].line_contents);
    assert_eq!("Database -> Server: Response", vec[3].line_data);
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
    pub fn parse(&self, lines: &str) -> HashSet<Participant> {
        let lvec = lines.lines().collect_vec();
        // parse participant names + index
        let participant_names = self.parse_participant_names(&lvec);

        // when is each participant active from / to?
        self.parse_participants(&lvec, &participant_names)
    }

    pub fn parse_participants(
        &self,
        lines: &[&str],
        participants: &HashMap<String, i32>,
    ) -> HashSet<Participant> {
        let mut first: HashMap<String, i32> = HashMap::new();
        let mut last: HashMap<String, i32> = HashMap::new();
        let mut activation_count = 0;
        lines
            .iter()
            .map(|line| line.trim())
            .filter_map(|line| {
                if self.line_filter(line) {
                    None
                } else {
                    self.interaction_regex.captures(line)
                }
            })
            .map(|captures| {
                let from_name = captures.index(1);
                let to_name = captures.index(2);
                vec![from_name.to_string(), to_name.to_string()]
            })
            .for_each(|name| {
                name.iter().for_each(|n| {
                    first.entry(n.to_string()).or_insert(activation_count);
                    last.entry(n.to_string())
                        .and_modify(|i| *i = activation_count)
                        .or_insert(activation_count);
                });
                activation_count.add_assign(1);
            });

        participants
            .iter()
            .map(|p| Participant {
                name: p.0.to_string(),
                index: *p.1 as usize,
                active_from: *first.get(p.0.as_str()).unwrap(),
                active_to: *last.get(p.0.as_str()).unwrap(),
            })
            .collect::<HashSet<Participant>>()
    }

    pub fn parse_participant_names(&self, lines: &[&str]) -> HashMap<String, i32> {
        let idx = AtomicI32::new(0);
        let mut data: HashMap<String, i32> = HashMap::new();
        lines
            .iter()
            .map(|line| line.trim())
            .filter_map(|line| {
                if self.line_filter(line) {
                    None
                } else {
                    self.interaction_regex.captures(line)
                }
            })
            .for_each(|captures| {
                let from_name = captures.index(1);
                let to_name = captures.index(2);

                if !data.contains_key(from_name) {
                    data.entry(from_name.to_string())
                        .or_insert_with(|| idx.fetch_add(1, Ordering::Relaxed));
                }

                if !data.contains_key(to_name) {
                    data.entry(to_name.to_string())
                        .or_insert_with(|| idx.fetch_add(1, Ordering::Relaxed));
                }
            });
        data
    }

    #[inline]
    fn line_filter(&self, line: &str) -> bool {
        line.is_empty() || line.starts_with(':') || line.starts_with('#')
    }
}

#[test]
fn test_parse_participant_names() {
    let pp = ParticipantParser::default();
    let input = "Client -> Server: Message";
    let data = pp.parse_participant_names(&input.lines().collect_vec());
    assert_eq!(2, data.len());
    assert_eq!(0, *data.get("Client").unwrap());
    assert_eq!(1, *data.get("Server").unwrap());

    let input = "Client -> Server: Message
    Server -> Database: Query";
    let data = pp.parse_participant_names(&input.lines().collect_vec());
    assert_eq!(3, data.len());
    assert_eq!(0, *data.get("Client").unwrap());
    assert_eq!(1, *data.get("Server").unwrap());
    assert_eq!(2, *data.get("Database").unwrap());
}

#[test]
fn test_parse_participants() {
    let pp = ParticipantParser::default();
    let input = "
    Client -> Server: Message
    Server -> Database: Query
    Server -> Client";
    let data = pp.parse_participant_names(&input.lines().collect_vec());
    assert_eq!(3, data.len());
    assert_eq!(0, *data.get("Client").unwrap());
    assert_eq!(1, *data.get("Server").unwrap());
    assert_eq!(2, *data.get("Database").unwrap());

    let p = pp.parse_participants(&input.lines().collect_vec(), &data);
    assert_eq!(data.len(), p.len());
    assert_eq!(0, *data.get("Client").unwrap());
    assert_eq!(1, *data.get("Server").unwrap());
    assert_eq!(2, *data.get("Database").unwrap());

    p.iter().for_each(|partic| {
        if partic.name == "Client" || partic.name == "Server" {
            assert_eq!(0, partic.active_from);
            assert_eq!(2, partic.active_to);
        } else if partic.name == "Database" {
            assert_eq!(1, partic.active_from);
            assert_eq!(1, partic.active_to);
        } else {
            assert!(false)
        }
    });
}

#[test]
fn test_participant_parser() {
    let pp = ParticipantParser::default();
    let input = "
    Client -> Server: Message
    Server -> Database: Query
    Server -> Client";
    let data = pp.parse(input);
    assert_eq!(3, data.len());

    data.iter().for_each(|p| {
        if p.name == "Client" {
            assert_eq!(0, p.index);
            assert_eq!(0, p.active_from);
            assert_eq!(2, p.active_to);
        } else if p.name == "Server" {
            assert_eq!(1, p.index);
            assert_eq!(0, p.active_from);
            assert_eq!(2, p.active_to);
        } else if p.name == "Database" {
            assert_eq!(2, p.index);
            assert_eq!(1, p.active_from);
            assert_eq!(1, p.active_to);
        } else {
            assert!(false);
        }
    });
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
    #[inline]
    fn line_filter(&self, line: &str) -> bool {
        line.is_empty() || line.starts_with(':') || line.starts_with('#')
    }

    pub fn parse(&self, _lines: &str, _participants: &HashSet<Participant>) -> InteractionSet {
        info!("InteractionParser.parse({:?}, {:?})", _lines, _participants);

        let interaction_index = AtomicI32::new(0);

        let lvec = _lines.lines().collect_vec();
        // parse participant names + index
        lvec.iter()
            .map(|line| line.trim())
            .filter_map(|line| {
                if self.line_filter(line) {
                    None
                } else {
                    self.interaction_regex.captures(line)
                }
            })
            .map(|captures| {
                let from_name = captures.index(1);
                let to_name = captures.index(2);
                let message = if captures.len() == 4 && !captures.index(3).trim().is_empty() {
                    Some(Message(captures.index(3).trim().to_string()))
                } else {
                    None
                };
                Interaction {
                    index: interaction_index.fetch_add(1, Ordering::Relaxed) as u32,
                    from_participant: _participants
                        .iter()
                        .filter(|p| p.name == from_name)
                        .exactly_one()
                        .unwrap()
                        .clone(),
                    to_participant: _participants
                        .iter()
                        .filter(|p| p.name == to_name)
                        .exactly_one()
                        .unwrap()
                        .clone(),
                    interaction_type: InteractionType::L2R,
                    message,
                }
            })
            .collect_vec()
    }
}

#[test]
fn test_interaction_parser() {
    let input = "Client -> Server: Message";
    let pp = ParticipantParser::default();
    let partics = pp.parse(input);

    let ip = InteractionParser::default();
    let inters = ip.parse(input, &partics);
    assert_eq!(1, inters.len());
    let interaction = inters.first().unwrap();
    assert_eq!(0, interaction.index);
    assert_eq!("Client", interaction.from_participant.name);
    assert_eq!("Server", interaction.to_participant.name);
    assert_eq!("Message", interaction.message.as_ref().unwrap().0);
}

// == Header ==============================================

#[derive(Debug)]
struct Header {}

// == Participant =========================================

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Participant {
    name: String,
    index: usize,
    active_from: i32,
    active_to: i32,
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
    index: u32,
    from_participant: Participant,
    to_participant: Participant,
    interaction_type: InteractionType,
    message: Option<Message>,
}

#[test]
fn test_parse_diagram() {
    let mut diagram: Vec<Line> = Vec::new();
    diagram.push(Line {
        line_number: 0,
        line_contents: LineContents::Invalid,
        line_data: "Test".to_string(),
    });
    let _ = Diagram::parse(&diagram);
}
