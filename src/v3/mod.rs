use std::collections::HashSet;

use model::{Interaction, Participant};

pub mod diagram;
mod model;
pub mod parsing;
mod theme;

type InteractionSet = Vec<Interaction>;
type ParticipantSet = HashSet<Participant>;
