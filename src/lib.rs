#[macro_use]
extern crate log;

use std::collections::HashSet;

use model::{Interaction, Participant};

pub mod diagram;
pub mod model;
pub mod parsing;
pub mod rendering;
pub mod theme;

type InteractionSet = Vec<Interaction>;
type ParticipantSet = HashSet<Participant>;
