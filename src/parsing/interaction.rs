use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};

use log::info;

use crate::model::{Interaction, InteractionType, Line, LineContents, Message, Participant};
use crate::InteractionSet;

// == Interaction Parser ==================================
#[derive(Debug)]
pub struct InteractionParser;

impl InteractionParser {
    fn interaction_type(from: &Participant, to: &Participant) -> InteractionType {
        match from.index.cmp(&to.index) {
            std::cmp::Ordering::Less => InteractionType::L2R,
            std::cmp::Ordering::Equal => InteractionType::SelfRef,
            std::cmp::Ordering::Greater => InteractionType::R2L,
        }
    }

    /// Parse interaction lines from a parsed document.
    ///
    /// For each `Interaction` or `InteractionWithMessage` line, looks up the
    /// corresponding participants and builds an [`Interaction`] value.
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
                    let from_p = participants.iter().find(|p| p.name == f.0).unwrap();
                    let to_p = participants.iter().find(|p| p.name == t.0).unwrap();
                    Interaction {
                        index: interaction_index.fetch_add(1, Ordering::Relaxed),
                        from_participant: from_p.to_owned(),
                        to_participant: to_p.to_owned(),
                        interaction_type: Self::interaction_type(from_p, to_p),
                        message: None,
                    }
                }
                LineContents::InteractionWithMessage(f, t, m) => {
                    info!("IwM: {:?}, {:?}, {:?}", f, t, m);
                    let from_p = participants.iter().find(|p| p.name == f.0).unwrap();
                    let to_p = participants.iter().find(|p| p.name == t.0).unwrap();
                    Interaction {
                        index: interaction_index.fetch_add(1, Ordering::Relaxed),
                        from_participant: from_p.clone(),
                        to_participant: to_p.clone(),
                        interaction_type: Self::interaction_type(from_p, to_p),
                        message: Some(Message(m.0.clone())),
                    }
                }
                _ => unreachable!("filter above only allows Interaction variants"),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        FromParticipant, InteractionMessage, LineContents, ToParticipant,
    };
    use crate::rendering::Rect;

    fn make_rect() -> Rect {
        Rect { x: 0, y: 0, w: 0, h: 0 }
    }

    fn make_participant(name: &str, index: usize) -> Participant {
        Participant {
            name: name.to_string(),
            index,
            active_from: 0,
            active_to: 0,
            rect: make_rect(),
        }
    }

    #[test]
    fn test_interaction_parser_l2r() {
        let document = vec![Line {
            line_contents: LineContents::Interaction(
                FromParticipant("Client".to_owned()),
                ToParticipant("Server".to_owned()),
            ),
            line_data: "Client -> Server".to_owned(),
            line_number: 0,
        }];

        let mut participants = HashSet::new();
        participants.insert(make_participant("Client", 0));
        participants.insert(make_participant("Server", 1));

        let inters = InteractionParser::parse(&document, &participants);
        assert_eq!(1, inters.len());

        let interaction = inters.first().unwrap();
        assert_eq!(0, interaction.index);
        assert_eq!("Client", interaction.from_participant.name);
        assert_eq!("Server", interaction.to_participant.name);
        assert_eq!(None, interaction.message);
        assert_eq!(InteractionType::L2R, interaction.interaction_type);
    }

    #[test]
    fn test_interaction_parser_r2l() {
        let document = vec![
            Line {
                line_contents: LineContents::Interaction(
                    FromParticipant("Client".to_owned()),
                    ToParticipant("Server".to_owned()),
                ),
                line_data: "Client -> Server".to_owned(),
                line_number: 0,
            },
            Line {
                line_contents: LineContents::Interaction(
                    FromParticipant("Server".to_owned()),
                    ToParticipant("Client".to_owned()),
                ),
                line_data: "Server -> Client".to_owned(),
                line_number: 1,
            },
        ];

        let mut participants = HashSet::new();
        participants.insert(make_participant("Client", 0));
        participants.insert(make_participant("Server", 1));

        let inters = InteractionParser::parse(&document, &participants);
        assert_eq!(2, inters.len());

        let mut sorted = inters;
        sorted.sort_by_key(|i| i.index);

        assert_eq!(InteractionType::L2R, sorted[0].interaction_type);
        assert_eq!(InteractionType::R2L, sorted[1].interaction_type);
    }

    #[test]
    fn test_interaction_parser_self_ref() {
        let document = vec![Line {
            line_contents: LineContents::InteractionWithMessage(
                FromParticipant("Client".to_owned()),
                ToParticipant("Client".to_owned()),
                InteractionMessage("Processing".to_owned()),
            ),
            line_data: "Client -> Client: Processing".to_owned(),
            line_number: 0,
        }];

        let mut participants = HashSet::new();
        participants.insert(make_participant("Client", 0));

        let inters = InteractionParser::parse(&document, &participants);
        assert_eq!(1, inters.len());

        let interaction = inters.first().unwrap();
        assert_eq!("Client", interaction.from_participant.name);
        assert_eq!("Client", interaction.to_participant.name);
        assert_eq!(
            Some(Message("Processing".to_string())),
            interaction.message
        );
        assert_eq!(InteractionType::SelfRef, interaction.interaction_type);
    }

    #[test]
    fn test_interaction_parser_with_message() {
        let document = vec![Line {
            line_contents: LineContents::InteractionWithMessage(
                FromParticipant("A".to_owned()),
                ToParticipant("B".to_owned()),
                InteractionMessage("hello".to_owned()),
            ),
            line_data: "A -> B: hello".to_owned(),
            line_number: 0,
        }];

        let mut participants = HashSet::new();
        participants.insert(make_participant("A", 0));
        participants.insert(make_participant("B", 1));

        let inters = InteractionParser::parse(&document, &participants);
        assert_eq!(1, inters.len());
        assert_eq!(
            Some(Message("hello".to_string())),
            inters[0].message
        );
    }

    #[test]
    fn test_interaction_parser_skips_non_interaction_lines() {
        let document = vec![
            Line {
                line_contents: LineContents::Empty,
                line_data: "".to_owned(),
                line_number: 0,
            },
            Line {
                line_contents: LineContents::Comment,
                line_data: "# a comment".to_owned(),
                line_number: 1,
            },
            Line {
                line_contents: LineContents::Interaction(
                    FromParticipant("A".to_owned()),
                    ToParticipant("B".to_owned()),
                ),
                line_data: "A -> B".to_owned(),
                line_number: 2,
            },
        ];

        let mut participants = HashSet::new();
        participants.insert(make_participant("A", 0));
        participants.insert(make_participant("B", 1));

        let inters = InteractionParser::parse(&document, &participants);
        assert_eq!(1, inters.len());
    }
}
