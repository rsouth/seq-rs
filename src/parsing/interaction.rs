use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::model::{Interaction, InteractionType, Line, LineContents, Message, Participant};

use crate::InteractionSet;

// == Interaction Parser ==================================
#[derive(Debug)]
pub struct InteractionParser;

impl InteractionParser {
    fn interaction_type(f: &Participant, t: &Participant) -> InteractionType {
        match f.index.cmp(&t.index) {
            std::cmp::Ordering::Less => InteractionType::L2R,
            std::cmp::Ordering::Equal => InteractionType::SelfRef,
            std::cmp::Ordering::Greater => InteractionType::R2L,
        }
    }

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

                    let from_p = participants.iter().find(|p| p.name == f.0).unwrap();
                    let to_p = participants.iter().find(|p| p.name == t.0).unwrap();

                    Interaction {
                        index: interaction_index.fetch_add(1, Ordering::Relaxed),
                        from_participant: from_p.to_owned(),
                        to_participant: to_p.to_owned(),
                        interaction_type: InteractionParser::interaction_type(from_p, to_p),
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
                        interaction_type: InteractionParser::interaction_type(from_p, to_p),
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

// #[test]
// fn test_interaction_parser() {
//     use crate::model::{FromParticipant, ToParticipant};
//     let document = vec![Line {
//         line_contents: LineContents::Interaction(
//             FromParticipant("Client".to_owned()),
//             ToParticipant("Server".to_owned()),
//         ),
//         line_data: "Client -> Server".to_owned(),
//         line_number: 0,
//     }];
//     let mut participants = HashSet::new();
//     participants.insert(Participant {
//         index: 0,
//         name: "Client".to_string(),
//         active_from: 0,
//         active_to: 0,
//     });
//     participants.insert(Participant {
//         index: 1,
//         name: "Server".to_string(),
//         active_from: 0,
//         active_to: 0,
//     });
//
//     let inters = InteractionParser::parse(&document, &participants);
//     assert_eq!(1, inters.len());
//     let interaction = inters.first().unwrap();
//     assert_eq!(0, interaction.index);
//     assert_eq!("Client", interaction.from_participant.name);
//     assert_eq!("Server", interaction.to_participant.name);
//     assert_eq!(None, interaction.message);
//     assert_eq!(InteractionType::L2R, interaction.interaction_type);
// }
//
// #[test]
// fn test_interaction_parser_with_r2l() {
//     use crate::model::{FromParticipant, ToParticipant};
//     use crate::parsing::participant::ParticipantParser;
//     let document = vec![
//         Line {
//             line_contents: LineContents::Interaction(
//                 FromParticipant("Client".to_owned()),
//                 ToParticipant("Server".to_owned()),
//             ),
//             line_data: "Client -> Server".to_owned(),
//             line_number: 0,
//         },
//         Line {
//             line_contents: LineContents::Interaction(
//                 FromParticipant("Server".to_owned()),
//                 ToParticipant("Client".to_owned()),
//             ),
//             line_data: "Server -> Client".to_owned(),
//             line_number: 1,
//         },
//     ];
//
//     let participants = ParticipantParser::parse(&document);
//
//     let inters = InteractionParser::parse(&document, &participants);
//     assert_eq!(2, inters.len());
//     let interaction = inters.first().unwrap();
//     assert_eq!(0, interaction.index);
//     assert_eq!("Client", interaction.from_participant.name);
//     assert_eq!("Server", interaction.to_participant.name);
//     assert_eq!(None, interaction.message);
//     assert_eq!(InteractionType::L2R, interaction.interaction_type);
//     //
//     let interaction = inters.last().unwrap();
//     assert_eq!(1, interaction.index);
//     assert_eq!("Server", interaction.from_participant.name);
//     assert_eq!("Client", interaction.to_participant.name);
//     assert_eq!(None, interaction.message);
//     assert_eq!(InteractionType::R2L, interaction.interaction_type);
// }
//
// #[test]
// fn test_interaction_parser_with_selfref() {
//     use crate::model::{FromParticipant, InteractionMessage, ToParticipant};
//     use crate::parsing::participant::ParticipantParser;
//     let document = vec![Line {
//         line_contents: LineContents::InteractionWithMessage(
//             FromParticipant("Client".to_owned()),
//             ToParticipant("Client".to_owned()),
//             InteractionMessage("Processing".to_owned()),
//         ),
//         line_data: "Client -> Client: Processing".to_owned(),
//         line_number: 0,
//     }];
//
//     let participants = ParticipantParser::parse(&document);
//
//     let inters = InteractionParser::parse(&document, &participants);
//     assert_eq!(1, inters.len());
//     let interaction = inters.first().unwrap();
//     assert_eq!(0, interaction.index);
//     assert_eq!("Client", interaction.from_participant.name);
//     assert_eq!("Client", interaction.to_participant.name);
//     assert_eq!(
//         Message("Processing".to_string()),
//         interaction.message.as_ref().unwrap().to_owned()
//     );
//     assert_eq!(InteractionType::SelfRef, interaction.interaction_type);
// }
