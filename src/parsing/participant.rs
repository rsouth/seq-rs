use std::collections::HashMap;

use log::info;

use crate::model::{Line, LineContents, Participant};
use crate::rendering::text::measure_string;
use crate::rendering::Rect;
use crate::theme::Theme;
use crate::ParticipantSet;

// == Participant Parser ==================================
#[derive(Debug, Default)]
pub struct ParticipantParser;

impl ParticipantParser {
    /// Iterate lines, noting the first and last appearance of each participant
    /// to compute their index, active range, and on-screen rect.
    pub fn parse(document: &[Line], theme: &Theme) -> ParticipantSet {
        let mut current_participant_index: usize = 0;
        let mut current_interaction_index: usize = 0;
        let mut participant_indices: HashMap<String, usize> = HashMap::new();
        let mut first_index_for_participant: HashMap<String, usize> = HashMap::new();
        let mut last_index_for_participant: HashMap<String, usize> = HashMap::new();
        let mut x_position_for_participant: HashMap<String, usize> = HashMap::new();
        let mut rect_for_participant: HashMap<String, Rect> = HashMap::new();

        let mut current_x = theme.document_border_width;
        let partic_h_gap = theme.partic_h_gap;

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
                    _ => unreachable!("filter above only allows Interaction variants"),
                };

                for participant_name in [&f.0, &t.0] {
                    if !participant_indices.contains_key(participant_name) {
                        participant_indices
                            .insert(participant_name.to_string(), current_participant_index);
                        current_participant_index += 1;

                        x_position_for_participant
                            .insert(participant_name.to_string(), current_x);
                        let string_rect =
                            measure_string(theme, participant_name, theme.partic_font_px);
                        rect_for_participant
                            .insert(participant_name.to_string(), string_rect);
                        current_x += partic_h_gap + string_rect.w;
                    }

                    first_index_for_participant
                        .entry(participant_name.to_string())
                        .or_insert(current_interaction_index);

                    last_index_for_participant
                        .insert(participant_name.to_string(), current_interaction_index);
                }

                current_interaction_index += 1;
            });

        info!("After first pass:");
        info!("Participant idx: {:#?}", participant_indices);
        info!("Participant active from: {:#?}", first_index_for_participant);
        info!("Participant active to: {:#?}", last_index_for_participant);

        let max_height = rect_for_participant.values().map(|r| r.h).max().unwrap_or(0);
        let partic_y = theme.document_border_width;

        participant_indices
            .iter()
            .map(|(name, &index)| Participant {
                active_from: *first_index_for_participant.get(name).unwrap(),
                active_to: *last_index_for_participant.get(name).unwrap(),
                rect: Rect {
                    x: *x_position_for_participant.get(name).unwrap(),
                    y: partic_y,
                    w: rect_for_participant.get(name).unwrap().w,
                    h: max_height,
                },
                name: name.clone(),
                index,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FromParticipant, InteractionMessage, ToParticipant};

    fn make_theme() -> Theme {
        Theme::default()
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
        let data = ParticipantParser::parse(&document, &make_theme());
        assert_eq!(2, data.len());
        assert_eq!(0, data.iter().find(|p| p.name == "Client").unwrap().index);
        assert_eq!(1, data.iter().find(|p| p.name == "Server").unwrap().index);
    }

    #[test]
    fn test_parse_participants_active_range() {
        let document = vec![
            Line {
                line_contents: LineContents::Empty,
                line_data: String::new(),
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
                line_data: "Server -> Client".to_string(),
                line_number: 3,
            },
        ];
        let data = ParticipantParser::parse(&document, &make_theme());
        assert_eq!(3, data.len());

        let client = data.iter().find(|p| p.name == "Client").unwrap();
        let server = data.iter().find(|p| p.name == "Server").unwrap();
        let database = data.iter().find(|p| p.name == "Database").unwrap();

        assert_eq!(0, client.index);
        assert_eq!(0, client.active_from);
        assert_eq!(2, client.active_to);

        assert_eq!(1, server.index);
        assert_eq!(0, server.active_from);
        assert_eq!(2, server.active_to);

        assert_eq!(2, database.index);
        assert_eq!(1, database.active_from);
        assert_eq!(1, database.active_to);
    }

    #[test]
    fn test_parse_participants_skips_non_interactions() {
        let document = vec![
            Line {
                line_contents: LineContents::Comment,
                line_data: "# comment".to_string(),
                line_number: 0,
            },
            Line {
                line_contents: LineContents::Interaction(
                    FromParticipant("A".to_string()),
                    ToParticipant("B".to_string()),
                ),
                line_data: "A -> B".to_string(),
                line_number: 1,
            },
        ];
        let data = ParticipantParser::parse(&document, &make_theme());
        assert_eq!(2, data.len());
    }

    #[test]
    fn test_parse_participants_rect_has_max_height() {
        // All participants should share the max height from measure_string
        let document = vec![
            Line {
                line_contents: LineContents::Interaction(
                    FromParticipant("A".to_string()),
                    ToParticipant("LongName".to_string()),
                ),
                line_data: "A -> LongName".to_string(),
                line_number: 0,
            },
        ];
        let data = ParticipantParser::parse(&document, &make_theme());
        let heights: Vec<usize> = data.iter().map(|p| p.rect.h).collect();
        let first = heights[0];
        assert!(heights.iter().all(|&h| h == first), "all participants should share max height");
    }
}
