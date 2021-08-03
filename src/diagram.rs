use crate::model::Config;
use crate::parsing::document::Document;
use crate::parsing::interaction::InteractionParser;
use crate::theme::Theme;
use crate::{
    model::Header, parsing::participant::ParticipantParser, InteractionSet, ParticipantSet,
};

// == Diagram =============================================
#[derive(Debug)]
pub struct Diagram {
    pub theme: Theme,
    pub header: Header,
    pub interactions: InteractionSet,
    pub participants: ParticipantSet,
    pub config: Config,
}

// Renderabl diagram - with position coords etc based on the theme
impl Diagram {
    pub fn parse(document: Document, theme: Theme) -> Diagram {
        info!("Document: {:?}", document);
        let participants = ParticipantParser::parse(&document.lines, &theme);

        info!("Got Partics: {:#?}", participants);
        let interactions = InteractionParser::parse(&document.lines, &participants);

        Diagram {
            theme: Default::default(),
            header: Header {},
            interactions,
            participants,
            config: document.config,
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
