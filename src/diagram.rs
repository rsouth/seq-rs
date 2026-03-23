use log::info;

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

impl Diagram {
    pub fn parse(document: Document, theme: Theme) -> Diagram {
        info!("Document: {:?}", document);
        let participants = ParticipantParser::parse(&document.lines, &theme);

        info!("Got participants: {:#?}", participants);
        let interactions = InteractionParser::parse(&document.lines, &participants);

        Diagram {
            theme,
            header: Header {},
            interactions,
            participants,
            config: document.config,
        }
    }
}
