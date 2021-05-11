use std::{
    collections::HashMap,
    sync::atomic::{AtomicI32, Ordering},
};

use Ordering::Relaxed;

use crate::v3::{
    model::{Line, LineContents, Participant},
    ParticipantSet,
};

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
    pub fn parse(document: &[Line]) -> ParticipantSet {
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
            .collect::<ParticipantSet>()
    }
}

#[test]
fn test_parse_participant_names() {
    use crate::v3::model::{FromParticipant, InteractionMessage, ToParticipant};
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
    assert_eq!(0, data.iter().find(|p| p.name == "Client").unwrap().index);
    assert_eq!(1, data.iter().find(|p| p.name == "Server").unwrap().index);
}

#[test]
fn test_parse_participants() {
    use crate::v3::model::{FromParticipant, InteractionMessage, ToParticipant};
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
    assert_eq!(0, data.iter().find(|p| p.name == "Client").unwrap().index);
    assert_eq!(1, data.iter().find(|p| p.name == "Server").unwrap().index);
    assert_eq!(2, data.iter().find(|p| p.name == "Database").unwrap().index);

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
    use crate::v3::model::{FromParticipant, InteractionMessage, ToParticipant};
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
