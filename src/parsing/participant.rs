use std::{
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::model::{Line, LineContents, Participant};
use crate::rendering::text::measure_string;
use crate::rendering::Rect;
use crate::theme::Theme;
use crate::ParticipantSet;
use ordered_float::OrderedFloat;
use Ordering::Relaxed;

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
    pub fn parse(document: &[Line], theme: &Theme) -> ParticipantSet {
        let current_participant_index = AtomicU32::new(0);
        let current_interaction_index = AtomicU32::new(0);
        let mut participant_indices: HashMap<String, u32> = HashMap::new();
        let mut first_index_for_participant: HashMap<String, u32> = HashMap::new();
        let mut last_index_for_participant: HashMap<String, u32> = HashMap::new();
        let mut x_position_for_participant: HashMap<String, OrderedFloat<f32>> = HashMap::new();
        let mut rect_for_participant: HashMap<String, Rect> = HashMap::new();

        let mut current_x = theme.document_border_width;
        let partic_h_gap = 20.0; // todo get from theme...

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

                let (f, t) = match &line.line_contents {
                    LineContents::Interaction(f, t) => (f, t),
                    LineContents::InteractionWithMessage(f, t, _) => (f, t),
                    _ => {
                        // when we have entry/exit messages this method will need to support only having
                        // a from or to participant alone.
                        panic!("...");
                    }
                };

                // participant index - from participant
                if !participant_indices.contains_key(&f.0) {
                    participant_indices.insert(
                        f.0.to_string(),
                        current_participant_index.fetch_add(1, Relaxed),
                    );

                    // by virtue of not being in participant_indices, it can't have an x_pos or width either...
                    x_position_for_participant.insert(f.0.to_string(), current_x.into());
                    let w = measure_string(theme, f.0.to_string().as_str(), theme.partic_font_px);
                    current_x = current_x + partic_h_gap + *&w.w.into_inner();
                    rect_for_participant.insert(f.0.to_string(), w);
                }

                // participant index - to participant
                if !participant_indices.contains_key(&t.0) {
                    participant_indices.insert(
                        t.0.to_string(),
                        current_participant_index.fetch_add(1, Relaxed),
                    );

                    // by virtue of not being in participant_indices, it can't have an x_pos or width either...
                    x_position_for_participant.insert(t.0.to_string(), current_x.into());
                    let w = measure_string(theme, t.0.to_string().as_str(), theme.partic_font_px);
                    current_x = current_x + partic_h_gap + *&w.w.into_inner();
                    rect_for_participant.insert(t.0.to_string(), w);
                }

                // active start index for this interaction on the 'from' participant
                if !first_index_for_participant.contains_key(&f.0) {
                    first_index_for_participant
                        .insert(f.0.to_string(), current_interaction_index.load(Relaxed));
                }

                // active start index for this interaction on the 'to' participant
                if !first_index_for_participant.contains_key(&t.0) {
                    first_index_for_participant
                        .insert(t.0.to_string(), current_interaction_index.load(Relaxed));
                }

                // active end index for this interaction on the 'from' participant
                last_index_for_participant
                    .insert(f.0.to_string(), current_interaction_index.load(Relaxed));
                // active end index for this interaction on the 'to' participant
                last_index_for_participant
                    .insert(t.0.to_string(), current_interaction_index.load(Relaxed));

                current_interaction_index.fetch_add(1, Relaxed);
            });

        info!("After first pass:");
        info!("Participant idx: {:#?}", participant_indices);
        info!(
            "Participant active from: {:#?}",
            first_index_for_participant
        );
        info!("Participant active to: {:#?}", last_index_for_participant);

        let max_height = rect_for_participant.values().map(|p| p.h).max().unwrap();

        participant_indices
            .iter()
            .map(|p_name| Participant {
                active_from: *first_index_for_participant.get(p_name.0).unwrap(),
                active_to: *last_index_for_participant.get(p_name.0).unwrap(),
                x: *x_position_for_participant.get(p_name.0).unwrap(),
                y: theme.document_border_width.into(), //(*rect_for_participant.get(p_name.0).unwrap()).clone(),
                w: rect_for_participant.get(p_name.0).unwrap().w,
                h: max_height,
                name: p_name.0.to_owned(),
                index: *p_name.1,
            })
            .collect::<ParticipantSet>()
    }
}

// #[test]
// fn test_parse_participant_names() {
//     use crate::model::{FromParticipant, InteractionMessage, ToParticipant};
//     let document = vec![Line {
//         line_contents: LineContents::InteractionWithMessage(
//             FromParticipant("Client".to_string()),
//             ToParticipant("Server".to_string()),
//             InteractionMessage("Message".to_string()),
//         ),
//         line_data: "Client -> Server: Message".to_string(),
//         line_number: 0,
//     }];
//     let data = ParticipantParser::parse(&document);
//     assert_eq!(2, data.len());
//     assert_eq!(0, data.iter().find(|p| p.name == "Client").unwrap().index);
//     assert_eq!(1, data.iter().find(|p| p.name == "Server").unwrap().index);
// }

// #[test]
// fn test_parse_participants() {
//     use crate::model::{FromParticipant, InteractionMessage, ToParticipant};
//     let document = vec![
//         Line {
//             line_contents: LineContents::Empty,
//             line_data: "".to_string(),
//             line_number: 0,
//         },
//         Line {
//             line_contents: LineContents::InteractionWithMessage(
//                 FromParticipant("Client".to_string()),
//                 ToParticipant("Server".to_string()),
//                 InteractionMessage("Message".to_string()),
//             ),
//             line_data: "Client -> Server: Message".to_string(),
//             line_number: 1,
//         },
//         Line {
//             line_contents: LineContents::InteractionWithMessage(
//                 FromParticipant("Server".to_string()),
//                 ToParticipant("Database".to_string()),
//                 InteractionMessage("Query".to_string()),
//             ),
//             line_data: "Server -> Database: Query".to_string(),
//             line_number: 2,
//         },
//         Line {
//             line_contents: LineContents::Interaction(
//                 FromParticipant("Server".to_string()),
//                 ToParticipant("Client".to_string()),
//             ),
//             line_data: "Client -> Server".to_string(),
//             line_number: 1,
//         },
//     ];
//     let data = ParticipantParser::parse(&document);
//     assert_eq!(3, data.len());
//     assert_eq!(0, data.iter().find(|p| p.name == "Client").unwrap().index);
//     assert_eq!(1, data.iter().find(|p| p.name == "Server").unwrap().index);
//     assert_eq!(2, data.iter().find(|p| p.name == "Database").unwrap().index);
//
//     data.iter().for_each(|partic| {
//         if partic.name == "Client" || partic.name == "Server" {
//             assert_eq!(0, partic.active_from);
//             assert_eq!(2, partic.active_to);
//         } else if partic.name == "Database" {
//             assert_eq!(1, partic.active_from);
//             assert_eq!(1, partic.active_to);
//         } else {
//             assert!(false)
//         }
//     });
// }
//
// #[test]
// fn test_participant_parser() {
//     use crate::model::{FromParticipant, InteractionMessage, ToParticipant};
//     let document = vec![
//         Line {
//             line_contents: LineContents::Empty,
//             line_data: "".to_string(),
//             line_number: 0,
//         },
//         Line {
//             line_contents: LineContents::InteractionWithMessage(
//                 FromParticipant("Client".to_string()),
//                 ToParticipant("Server".to_string()),
//                 InteractionMessage("Message".to_string()),
//             ),
//             line_data: "Client -> Server: Message".to_string(),
//             line_number: 1,
//         },
//         Line {
//             line_contents: LineContents::InteractionWithMessage(
//                 FromParticipant("Server".to_string()),
//                 ToParticipant("Database".to_string()),
//                 InteractionMessage("Query".to_string()),
//             ),
//             line_data: "Server -> Database: Query".to_string(),
//             line_number: 2,
//         },
//         Line {
//             line_contents: LineContents::Interaction(
//                 FromParticipant("Server".to_string()),
//                 ToParticipant("Client".to_string()),
//             ),
//             line_data: "Client -> Server".to_string(),
//             line_number: 1,
//         },
//     ];
//     let data = ParticipantParser::parse(&document);
//     assert_eq!(3, data.len());
//
//     data.iter().for_each(|p| {
//         if p.name == "Client" {
//             assert_eq!(0, p.index);
//             assert_eq!(0, p.active_from);
//             assert_eq!(2, p.active_to);
//         } else if p.name == "Server" {
//             assert_eq!(1, p.index);
//             assert_eq!(0, p.active_from);
//             assert_eq!(2, p.active_to);
//         } else if p.name == "Database" {
//             assert_eq!(2, p.index);
//             assert_eq!(1, p.active_from);
//             assert_eq!(1, p.active_to);
//         } else {
//             assert!(false);
//         }
//     });
// }
