use std::{
    collections::HashMap,
    sync::atomic::{AtomicI32, AtomicU32, Ordering},
};
use std::{collections::HashSet, ops::Index};

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use Ordering::Relaxed;

use crate::v3::model::{Interaction, InteractionType, Line, LineContents, Message, Participant};
use crate::v3::InteractionSet;

use super::model::{FromParticipant, InteractionMessage, ToParticipant};

lazy_static! {
    static ref INTERACTION_REGEX: Regex = Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap();
}

// == Document Parser =====================================
pub struct DocumentParser;
impl DocumentParser {
    pub fn parse(line: &str) -> Vec<Line> {
        let line_number = AtomicU32::new(0);
        line.lines()
            .into_iter()
            .map(|line| line.trim())
            .map(|line| {
                if line.is_empty() {
                    Line {
                        line_number: line_number.fetch_add(1, Relaxed),
                        line_contents: LineContents::Empty,
                        line_data: line.to_owned(),
                    }
                } else if line.starts_with('#') {
                    Line {
                        line_number: line_number.fetch_add(1, Relaxed),
                        line_contents: LineContents::Comment,
                        line_data: line.to_owned(),
                    }
                } else if line.starts_with(':') {
                    Line {
                        line_number: line_number.fetch_add(1, Relaxed),
                        line_contents: LineContents::MetaData,
                        line_data: line.to_owned(),
                    }
                } else {
                    match INTERACTION_REGEX.captures(line) {
                        None => Line {
                            line_number: line_number.fetch_add(1, Relaxed),
                            line_contents: LineContents::Invalid,
                            line_data: line.to_owned(),
                        },
                        Some(captures) => {
                            let from_name = FromParticipant(captures.index(1).trim().to_owned());
                            let to_name = ToParticipant(captures.index(2).trim().to_owned());

                            if captures.len() >= 3 && !captures.index(3).is_empty() {
                                let msg = InteractionMessage(captures.index(3).trim().to_owned());
                                Line {
                                    line_number: line_number.fetch_add(1, Relaxed),
                                    line_contents: LineContents::InteractionWithMessage(
                                        from_name, to_name, msg,
                                    ),
                                    line_data: line.to_owned(),
                                }
                            } else {
                                Line {
                                    line_number: line_number.fetch_add(1, Relaxed),
                                    line_contents: LineContents::Interaction(from_name, to_name),
                                    line_data: line.to_owned(),
                                }
                            }
                        }
                    }
                }
            })
            .collect_vec()
    }
}

#[test]
fn test_document_parser_with_invalid() {
    let text = "    Client -> Server: Message
    Server
    -> Server: Response";
    let vec = DocumentParser::parse(text);
    assert_eq!(3, vec.len());

    // line 0
    let client_participant = FromParticipant("Client".to_string());
    let server_participant = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Message".to_string());
    assert_eq!(0, vec[0].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(client_participant, server_participant, expect_msg),
        vec[0].line_contents
    );

    // line 1
    assert_eq!(1, vec[1].line_number);
    assert_eq!(LineContents::Invalid, vec[1].line_contents);

    // line 2
    assert_eq!(2, vec[2].line_number);
    assert_eq!(LineContents::Invalid, vec[2].line_contents);
}

#[test]
fn test_document_parser() {
    let text = "
    :title Test
    Client -> Server: Message
    Server -> Database
    Database -> Server: Response";
    let vec = DocumentParser::parse(text);
    assert_eq!(5, vec.len());

    // line 0
    assert_eq!(0, vec[0].line_number);
    assert_eq!(LineContents::Empty, vec[0].line_contents);
    assert_eq!("", vec[0].line_data);

    // line 1
    assert_eq!(1, vec[1].line_number);
    assert_eq!(LineContents::MetaData, vec[1].line_contents);
    assert_eq!(":title Test", vec[1].line_data);

    // line 2
    let expect_from = FromParticipant("Client".to_string());
    let expect_to = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Message".to_string());
    assert_eq!(2, vec[2].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(expect_from, expect_to, expect_msg),
        vec[2].line_contents
    );
    assert_eq!("Client -> Server: Message", vec[2].line_data);

    // line 3
    let expect_from = FromParticipant("Server".to_string());
    let expect_to = ToParticipant("Database".to_string());
    assert_eq!(3, vec[3].line_number);
    assert_eq!(
        LineContents::Interaction(expect_from, expect_to),
        vec[3].line_contents
    );
    assert_eq!("Server -> Database", vec[3].line_data);

    // line 4
    let expect_from = FromParticipant("Database".to_string());
    let expect_to = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Response".to_string());
    assert_eq!(4, vec[4].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(expect_from, expect_to, expect_msg),
        vec[4].line_contents
    );
    assert_eq!("Database -> Server: Response", vec[4].line_data);
}

// == Participant Parser ==================================
#[derive(Debug)]
pub struct ParticipantParser;

impl Default for ParticipantParser {
    fn default() -> Self {
        ParticipantParser {}
    }
}

impl ParticipantParser {
    ///
    /// iterate lines, looking only at Interaction types
    /// note down the first appearance of a Participant
    ///  -> this is it's index, and it's activation_start
    /// note down the last appearange of a Participant
    ///  -> this is it's activation_end
    pub fn parse(document: &[Line]) -> HashSet<Participant> {
        let p_idx = AtomicI32::new(0);
        let i_idx = AtomicI32::new(0);
        let mut idx_for_p: HashMap<String, i32> = HashMap::new();
        let mut start_idx_for_p: HashMap<String, i32> = HashMap::new();
        let mut end_idx_for_p: HashMap<String, i32> = HashMap::new();

        document
            .iter()
            .filter(|line| {
                matches!(
                    line.line_contents,
                    LineContents::Interaction(_, _) | LineContents::InteractionWithMessage(_, _, _)
                )
            })
            .for_each(|line| {
                info!("Pass 1: {:#?}", line);
                match &line.line_contents {
                    LineContents::Interaction(f, t)
                    | LineContents::InteractionWithMessage(f, t, _) => {
                        // participant index
                        if !idx_for_p.contains_key(&f.0) {
                            idx_for_p.insert(f.0.to_string(), p_idx.fetch_add(1, Relaxed));
                        }

                        if !idx_for_p.contains_key(&t.0) {
                            idx_for_p.insert(t.0.to_string(), p_idx.fetch_add(1, Relaxed));
                        }

                        // active start index
                        if !start_idx_for_p.contains_key(&f.0) {
                            start_idx_for_p.insert(f.0.to_string(), i_idx.load(Relaxed));
                        }

                        if !start_idx_for_p.contains_key(&t.0) {
                            start_idx_for_p.insert(t.0.to_string(), i_idx.load(Relaxed));
                        }

                        // active end index
                        end_idx_for_p.insert(f.0.to_string(), i_idx.load(Relaxed));
                        end_idx_for_p.insert(t.0.to_string(), i_idx.load(Relaxed));
                    }
                    _ => {
                        panic!("...");
                    }
                }

                i_idx.fetch_add(1, Relaxed);
            });

        info!("After first pass:");
        info!("Participant idx: {:#?}", idx_for_p);
        info!("Participant active from: {:#?}", start_idx_for_p);
        info!("Participant active to: {:#?}", end_idx_for_p);

        idx_for_p
            .iter()
            .map(|p_name| {
                let name = p_name.0.to_owned();
                Participant {
                    name: name.clone(),
                    index: *p_name.1 as usize,
                    active_from: *start_idx_for_p.get(&name).unwrap(),
                    active_to: *end_idx_for_p.get(&name).unwrap(),
                }
            })
            .collect::<HashSet<Participant>>()
    }
}

#[test]
fn test_parse_participant_names() {
    let document = vec![Line {
        line_contents: LineContents::InteractionWithMessage(
            FromParticipant("Client".to_string()),
            ToParticipant("Server".to_string()),
            InteractionMessage("Message".to_string()),
        ),
        line_data: "Client -> Server: Message".to_string(),
        line_number: 0,
    }];
    let data = ParticipantParser::parse(&document);
    assert_eq!(2, data.len());
    assert_eq!(
        0,
        data.iter()
            .filter(|p| p.name == "Client")
            .exactly_one()
            .unwrap()
            .index
    );
    assert_eq!(
        1,
        data.iter()
            .filter(|p| p.name == "Server")
            .exactly_one()
            .unwrap()
            .index
    );
}

#[test]
fn test_parse_participants() {
    let document = vec![
        Line {
            line_contents: LineContents::Empty,
            line_data: "".to_string(),
            line_number: 0,
        },
        Line {
            line_contents: LineContents::InteractionWithMessage(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string()),
                InteractionMessage("Message".to_string()),
            ),
            line_data: "Client -> Server: Message".to_string(),
            line_number: 1,
        },
        Line {
            line_contents: LineContents::InteractionWithMessage(
                FromParticipant("Server".to_string()),
                ToParticipant("Database".to_string()),
                InteractionMessage("Query".to_string()),
            ),
            line_data: "Server -> Database: Query".to_string(),
            line_number: 2,
        },
        Line {
            line_contents: LineContents::Interaction(
                FromParticipant("Server".to_string()),
                ToParticipant("Client".to_string()),
            ),
            line_data: "Client -> Server".to_string(),
            line_number: 1,
        },
    ];
    let data = ParticipantParser::parse(&document);
    assert_eq!(3, data.len());
    assert_eq!(
        0,
        data.iter()
            .filter(|p| p.name == "Client")
            .exactly_one()
            .unwrap()
            .index
    );
    assert_eq!(
        1,
        data.iter()
            .filter(|p| p.name == "Server")
            .exactly_one()
            .unwrap()
            .index
    );
    assert_eq!(
        2,
        data.iter()
            .filter(|p| p.name == "Database")
            .exactly_one()
            .unwrap()
            .index
    );

    data.iter().for_each(|partic| {
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
    let document = vec![
        Line {
            line_contents: LineContents::Empty,
            line_data: "".to_string(),
            line_number: 0,
        },
        Line {
            line_contents: LineContents::InteractionWithMessage(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string()),
                InteractionMessage("Message".to_string()),
            ),
            line_data: "Client -> Server: Message".to_string(),
            line_number: 1,
        },
        Line {
            line_contents: LineContents::InteractionWithMessage(
                FromParticipant("Server".to_string()),
                ToParticipant("Database".to_string()),
                InteractionMessage("Query".to_string()),
            ),
            line_data: "Server -> Database: Query".to_string(),
            line_number: 2,
        },
        Line {
            line_contents: LineContents::Interaction(
                FromParticipant("Server".to_string()),
                ToParticipant("Client".to_string()),
            ),
            line_data: "Client -> Server".to_string(),
            line_number: 1,
        },
    ];
    let data = ParticipantParser::parse(&document);
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
pub struct InteractionParser;

#[allow(dead_code)]
impl InteractionParser {
    fn interaction_type(f: &Participant, t: &Participant) -> InteractionType {
        match f.index.cmp(&t.index) {
            std::cmp::Ordering::Less => InteractionType::L2R,
            std::cmp::Ordering::Equal => InteractionType::SelfRef,
            std::cmp::Ordering::Greater => InteractionType::R2L,
        }
    }

    // todo Pass #2 - interactions
    // iterate lines, looking only at Interaction types
    // note if an interaction is L2R, R2L, SelfRef etc.
    // note if an interaction is Message vs Reply
    // note if an interaction is Sync vs Async
    pub fn parse(document: &[Line], participants: &HashSet<Participant>) -> InteractionSet {
        info!("InteractionParser.parse({:#?})", document);

        let interaction_index = AtomicU32::new(0);
        document
            .iter()
            .filter(|line| {
                matches!(
                    line.line_contents,
                    LineContents::Interaction(_, _) | LineContents::InteractionWithMessage(_, _, _)
                )
            })
            .map(|line| match &line.line_contents {
                LineContents::Interaction(f, t) => {
                    info!("I: {:?}, {:?}", f, t);

                    let from_p = participants
                        .iter()
                        .filter(|p| p.name == f.0)
                        .exactly_one()
                        .unwrap()
                        .clone();
                    let to_p = participants
                        .iter()
                        .filter(|p| p.name == t.0)
                        .exactly_one()
                        .unwrap()
                        .clone();

                    Interaction {
                        index: interaction_index.fetch_add(1, Ordering::Relaxed),
                        from_participant: from_p.clone(),
                        to_participant: to_p.clone(),
                        interaction_type: InteractionParser::interaction_type(&from_p, &to_p),
                        message: None,
                    }
                }
                LineContents::InteractionWithMessage(f, t, m) => {
                    info!("IwM: {:?}, {:?}, {:?}", f, t, m);

                    let from_p = participants
                        .iter()
                        .filter(|p| p.name == f.0)
                        .exactly_one()
                        .unwrap()
                        .clone();
                    let to_p = participants
                        .iter()
                        .filter(|p| p.name == t.0)
                        .exactly_one()
                        .unwrap()
                        .clone();

                    Interaction {
                        index: interaction_index.fetch_add(1, Ordering::Relaxed),
                        from_participant: from_p.clone(),
                        to_participant: to_p.clone(),
                        interaction_type: InteractionParser::interaction_type(&from_p, &to_p),
                        message: Some(Message(m.0.to_string())),
                    }
                }
                _ => {
                    panic!("...");
                }
            })
            .collect::<InteractionSet>()
    }
}

#[test]
fn test_interaction_parser() {
    let document = vec![Line {
        line_contents: LineContents::Interaction(
            FromParticipant("Client".to_owned()),
            ToParticipant("Server".to_owned()),
        ),
        line_data: "Client -> Server".to_owned(),
        line_number: 0,
    }];
    let mut participants = HashSet::new();
    participants.insert(Participant {
        index: 0,
        name: "Client".to_string(),
        active_from: 0,
        active_to: 0,
    });
    participants.insert(Participant {
        index: 1,
        name: "Server".to_string(),
        active_from: 0,
        active_to: 0,
    });

    let inters = InteractionParser::parse(&document, &participants);
    assert_eq!(1, inters.len());
    let interaction = inters.first().unwrap();
    assert_eq!(0, interaction.index);
    assert_eq!("Client", interaction.from_participant.name);
    assert_eq!("Server", interaction.to_participant.name);
    assert_eq!(None, interaction.message);
}
