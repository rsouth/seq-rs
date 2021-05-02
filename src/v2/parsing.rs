use crate::v2::{
    Diagram, Interaction, InteractionSet, Message, Parse, ParseError, ParseResult, Participant,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::ops::Index;
use std::str::Lines;

lazy_static! {
    static ref INTERACTION_REGEX: Regex = Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap();
}

impl Parse<Diagram> for Diagram {
    fn parse(lines: Lines) -> ParseResult<Diagram> {
        let interactions = InteractionSet::parse(lines)?;
        let participants = interactions
            .iter()
            .map(|interaction| {
                smallvec::SmallVec::from_buf([
                    interaction.from_participant.clone(),
                    interaction.to_participant.clone(),
                ])
            })
            .flatten()
            .unique()
            .collect();

        Ok(Diagram {
            interactions,
            participants,
        })
    }
}

impl Parse<InteractionSet> for InteractionSet {
    fn parse(lines: Lines) -> Result<InteractionSet, ParseError> {
        let parsed_interactions = lines
            .into_iter()
            .map(|line| line.trim())
            .filter(|line| line.contains("->") && !line.starts_with('#'))
            .filter_map(|line| INTERACTION_REGEX.captures(line))
            .filter_map(|captures| {
                if captures.len() >= 3 {
                    let from_participant = Participant {
                        name: captures.index(1).trim().to_string(),
                    };
                    let to_participant = Participant {
                        name: captures.index(2).trim().to_string(),
                    };
                    let message = if captures.len() == 4 && !captures.index(3).trim().is_empty() {
                        Some(Message(captures.index(3).trim().to_string()))
                    } else {
                        None
                    };

                    Some(Interaction {
                        from_participant,
                        to_participant,
                        message,
                    })
                } else {
                    None
                }
            })
            .collect::<InteractionSet>();

        debug!(
            "Parsed {} interactions: {:?}",
            parsed_interactions.len(),
            parsed_interactions,
        );

        Ok(parsed_interactions)
    }
}
