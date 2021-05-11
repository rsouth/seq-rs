use crate::v3::model::{Header, Line};
use crate::v3::theme::Theme;
use crate::v3::{InteractionSet, ParticipantSet};

// == Diagram =============================================
#[derive(Debug)]
pub struct Diagram {
    theme: Theme,
    header: Header,
    interactions: InteractionSet,
    participants: ParticipantSet,
}

#[allow(dead_code)]
impl Diagram {
    pub fn parse(_document: &[Line]) -> Diagram {
        println!("Document: {:?}", _document);

        Diagram {
            theme: Default::default(),
            header: Header {},
            interactions: vec![],
            participants: vec![],
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
