// use crate::v2::rendering::render_context::{RenderingContext, Theme};
// use crate::v2::{
//     Diagram, Interaction, InteractionSet, Message, Parse, ParseError, ParseResult, Participant,
//     ParticipantSet,
// };
// use itertools::{Itertools, MinMaxResult};
// use lazy_static::lazy_static;
// use regex::Regex;
// use std::collections::HashMap;
// use std::ops::Index;
// use std::str::Lines;
// use std::sync::atomic::{AtomicU32, Ordering};
//
// lazy_static! {
//     static ref INTERACTION_REGEX: Regex = Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap();
// }
//
// // == Diagram Builder =====================================
// pub struct DiagramBuilder {
//     pub(crate) interactions: InteractionSet,
//     pub(crate) participants: ParticipantSet,
// }
//
// impl DiagramBuilder {
//     pub fn create(self, theme: Theme) -> Result<Diagram, std::io::Error> {
//         let rendering_context =
//             RenderingContext::create(&self.interactions, &self.participants, theme);
//         Ok(Diagram {
//             interactions: self.interactions,
//             participants: self.participants,
//             rendering_context,
//         })
//     }
// }
//
// // == Participant Record ==================================
// struct ParticipantRecord {
//     record: HashMap<String, u32>,
//     partipants: AtomicU32,
// }
//
// impl Default for ParticipantRecord {
//     fn default() -> Self {
//         ParticipantRecord {
//             record: HashMap::new(),
//             partipants: AtomicU32::new(0),
//         }
//     }
// }
//
// impl ParticipantRecord {
//     fn get_for(&mut self, name: &str) -> u32 {
//         if self.record.contains_key(name) {
//             *self.record.get(name).unwrap()
//         } else {
//             self.record.insert(
//                 name.to_string(),
//                 self.partipants.fetch_add(1, Ordering::Relaxed),
//             );
//             *self.record.get(name).unwrap()
//         }
//     }
// }
//
// // == Parse for Diagram ===================================
// impl Parse<DiagramBuilder> for Diagram {
//     fn parse(lines: Lines) -> ParseResult<DiagramBuilder> {
//         let interactions: InteractionSet = InteractionSet::parse(lines)?;
//         let mut participants: ParticipantSet = interactions
//             .iter()
//             .map(|interaction| {
//                 smallvec::SmallVec::from_buf([
//                     interaction.from_participant.clone(),
//                     interaction.to_participant.clone(),
//                 ])
//             })
//             .flatten()
//             .unique()
//             .collect();
//
//         // figure out the first and last appearance of the participant in interactions
//         for x in &mut participants {
//             if let MinMaxResult::MinMax(min, max) = interactions
//                 .iter()
//                 .filter_map(|i| {
//                     match i.from_participant.name == x.name || i.to_participant.name == x.name {
//                         true => Option::Some(i.count),
//                         false => Option::None,
//                     }
//                 })
//                 .minmax()
//             {
//                 x.active_from = min;
//                 x.active_until = max;
//             };
//         }
//
//         info!("Partics: {:?}", participants);
//
//         Ok(DiagramBuilder {
//             interactions,
//             participants,
//         })
//     }
// }
//
// // == Parse for InteractionSet ============================
// impl Parse<InteractionSet> for InteractionSet {
//     fn parse(lines: Lines) -> Result<InteractionSet, ParseError> {
//         let mut partic_record = ParticipantRecord::default();
//
//         let mut count = 0;
//         let parsed_interactions = lines
//             .into_iter()
//             .map(|line| line.trim())
//             .filter(|line| line.contains("->") && !line.starts_with('#'))
//             .filter_map(|line| INTERACTION_REGEX.captures(line))
//             .filter_map(|captures| {
//                 if captures.len() >= 3 {
//                     let from_name = captures.index(1).trim();
//                     let from_participant =
//                         Participant::new(from_name, partic_record.get_for(from_name));
//                     let to_name = captures.index(2).trim();
//                     let to_participant = Participant::new(to_name, partic_record.get_for(to_name));
//                     let message = if captures.len() == 4 && !captures.index(3).trim().is_empty() {
//                         Some(Message(captures.index(3).trim().to_string()))
//                     } else {
//                         None
//                     };
//
//                     let interaction = Interaction {
//                         from_participant,
//                         to_participant,
//                         message,
//                         count,
//                     };
//                     count += 1;
//                     Some(interaction)
//                 } else {
//                     None
//                 }
//             })
//             .collect::<InteractionSet>();
//
//         info!(
//             "Parsed {} interactions: {:?}",
//             parsed_interactions.len(),
//             parsed_interactions,
//         );
//
//         Ok(parsed_interactions)
//     }
// }
