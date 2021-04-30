use std::collections::HashSet;
use std::ops::Index;
use std::str::Lines;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use regex::Regex;

pub type ParticSet = HashSet<Participant>;

pub type InteractionSet = Vec<Interaction>;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Participant {
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Interaction {
    pub from_participant: Participant,
    pub to_participant: Participant,
    pub message: Option<Message>,
    pub order: u32,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Message(String);

// == Interaction Parser ==================================
pub struct InteractionParser {
    interaction_regex: Regex,
    counter: AtomicU32,
}

impl Default for InteractionParser {
    fn default() -> Self {
        InteractionParser {
            interaction_regex: regex::Regex::new("^(.+)(\\s+-+>+\\s+)([^:]+):?(.*)$").unwrap(),
            counter: AtomicU32::new(0),
        }
    }
}

impl InteractionParser {
    fn get_incr(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}

impl InteractionParser {
    pub fn parse_interactions(&self, lines: Lines) -> InteractionSet {
        let start_time = Instant::now();
        let parsed_interactions = lines
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| p.contains("->") && !p.starts_with('#'))
            .filter_map(|p| self.interaction_regex.captures(p))
            .filter_map(|p| {
                if p.len() >= 3 {
                    let from_participant = Participant {
                        name: p.index(1).trim().to_string(),
                    };
                    let to_participant = Participant {
                        name: p.index(3).trim().to_string(),
                    };
                    let message = if p.len() == 5 && !p.index(4).trim().is_empty() {
                        Some(Message(p.index(4).trim().to_string()))
                    } else {
                        None
                    };

                    Some(Interaction {
                        from_participant,
                        to_participant,
                        message,
                        order: self.get_incr(),
                    })
                } else {
                    None
                }
            })
            .collect::<InteractionSet>();

        debug!(
            "Parsed {} interactions in {}µs: {:?}",
            parsed_interactions.len(),
            start_time.elapsed().as_micros(),
            parsed_interactions,
        );
        parsed_interactions
    }
}

// == Tests ===============================================

#[test]
fn test_parse_interactions() {
    let parser = InteractionParser::default();

    // happy path
    let set = parser.parse_interactions("One -> Two: Do the kung fu".lines());
    println!("partics are {:?}", set);
    assert_eq!(1, set.len());
    assert_eq!(
        Participant {
            name: "One".to_string()
        },
        set.get(0).unwrap().from_participant
    );
    assert_eq!(
        Participant {
            name: "Two".to_string()
        },
        set.get(0).unwrap().to_participant
    );
    assert_eq!(
        Message("Do the kung fu".to_string()),
        *set.get(0).unwrap().message.as_ref().unwrap()
    );

    let set = parser.parse_interactions("One more -> Two more: Multi words".lines());
    assert_eq!(1, set.len());
    assert_eq!(
        Participant {
            name: "One more".to_string()
        },
        set.get(0).unwrap().from_participant
    );
    assert_eq!(
        Participant {
            name: "Two more".to_string()
        },
        set.get(0).unwrap().to_participant
    );
    assert_eq!(
        Message("Multi words".to_string()),
        *set.get(0).unwrap().message.as_ref().unwrap()
    );

    let set = parser.parse_interactions("One -> Two Do the kung fu".lines());
    assert_eq!(1, set.len());
    assert_eq!(
        Participant {
            name: "One".to_string()
        },
        set.get(0).unwrap().from_participant
    );
    assert_eq!(
        Participant {
            name: "Two Do the kung fu".to_string()
        },
        set.get(0).unwrap().to_participant
    );
    assert_eq!(None, set.get(0).unwrap().message);
    println!("Set is: {:?}", set);
}
