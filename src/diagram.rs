use crate::parsing::interaction::InteractionParser;
use crate::theme::Theme;
use crate::{
    model::{Header, Line},
    parsing::participant::ParticipantParser,
    InteractionSet, ParticipantSet,
};

// == Diagram =============================================
#[derive(Debug)]
pub struct Diagram {
    pub theme: Theme,
    pub header: Header,
    pub interactions: InteractionSet,
    pub participants: ParticipantSet,
}

// Renderabl diagram - with position coords etc based on the theme
impl Diagram {
    pub fn parse(document: Vec<Line>, theme: Theme) -> Diagram {
        info!("Document: {:?}", document);
        let participants = ParticipantParser::parse(&document, &theme);

        info!("Got Partics: {:#?}", participants);
        let interactions = InteractionParser::parse(&document, &participants);

        Diagram {
            theme: Default::default(),
            header: Header {},
            interactions,
            participants,
        }
    }
}

// #[test]
// fn test_parse_diagram() {
//     use crate::model::LineContents;
//     let diagram: Vec<Line> = vec![Line {
//         line_number: 0,
//         line_contents: LineContents::Invalid,
//         line_data: "Test".to_string(),
//     }];
//     let _ = Diagram::parse(&diagram);
// }
