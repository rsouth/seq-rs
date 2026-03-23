use log::info;

use crate::model::{Config, Header, LineContents, MetaDataType};
use crate::parsing::document::Document;
use crate::parsing::interaction::InteractionParser;
use crate::theme::Theme;
use crate::{
    parsing::participant::ParticipantParser, InteractionSet, ParticipantSet,
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

impl Diagram {
    pub fn parse(document: Document, theme: Theme) -> Diagram {
        info!("Document: {:?}", document);
        let header = Self::extract_header(&document.lines);
        let participants = ParticipantParser::parse(&document.lines, &theme);

        info!("Got participants: {:#?}", participants);
        let interactions = InteractionParser::parse(&document.lines, &participants);

        Diagram {
            theme,
            header,
            interactions,
            participants,
            config: document.config,
        }
    }

    fn extract_header(lines: &[crate::model::Line]) -> Header {
        let mut title = None;
        let mut author = None;
        for line in lines {
            match &line.line_contents {
                LineContents::MetaData(MetaDataType::Title(t)) => title = Some(t.clone()),
                LineContents::MetaData(MetaDataType::Author(a)) => author = Some(a.clone()),
                _ => {}
            }
        }
        Header { title, author }
    }
}
