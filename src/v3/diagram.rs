use crate::v3::{
    model::{Header, Line},
    parsing::participant::ParticipantParser,
};
use crate::v3::{parsing::interaction::InteractionParser, theme::Theme};
use crate::v3::{InteractionSet, ParticipantSet};

// == Diagram =============================================
#[derive(Debug)]
pub struct Diagram {
    theme: Theme,
    header: Header,
    interactions: InteractionSet,
    participants: ParticipantSet,
}

impl Diagram {
    pub fn parse(document: &[Line]) -> Diagram {
        info!("Document: {:?}", document);
        let participants = ParticipantParser::parse(document);

        info!("Got Partics: {:#?}", participants);
        let interactions = InteractionParser::parse(document, &participants);

        Diagram {
            theme: Default::default(),
            header: Header {},
            interactions,
            participants,
        }
    }
}

#[test]
fn test_parse_diagram() {
    use crate::v3::model::LineContents;
    let diagram: Vec<Line> = vec![Line {
        line_number: 0,
        line_contents: LineContents::Invalid,
        line_data: "Test".to_string(),
    }];
    let _ = Diagram::parse(&diagram);
}
