use std::collections::HashSet;
use std::ops::Index;
use std::str::Lines;
use std::sync::atomic::{AtomicI32, Ordering};

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

// == Participant Parser ==================================
pub struct ParticipantParser {
    partic_regex: Regex,
}

impl Default for ParticipantParser {
    fn default() -> Self {
        ParticipantParser {
            partic_regex: regex::Regex::new("^(\\w*) -+>+ (\\w*):?.*$").unwrap(),
        }
    }
}

impl ParticipantParser {
    pub fn parse_participants(&self, lines: Lines) -> ParticSet {
        let parsed_participants = lines
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| p.contains("->") && !p.starts_with("#"))
            .filter_map(|p| self.partic_regex.captures(p))
            .map(|p| vec![p.index(1).to_string(), p.index(2).to_string()])
            .flatten()
            .map(|p| Participant { name: p })
            .collect::<ParticSet>();
        debug!("Parsed participants: {:?}", parsed_participants);
        parsed_participants
    }
}

// == Interaction Parser ==================================
pub struct InteractionParser {
    interaction_regex: Regex,
    counter: AtomicI32,
}

impl Default for InteractionParser {
    fn default() -> Self {
        InteractionParser {
            interaction_regex: regex::Regex::new("^(.+)(\\s+-+>+\\s+)([^:]+):?(.*)$").unwrap(),
            counter: AtomicI32::default(),
        }
    }
}

impl InteractionParser {
    fn get_incr(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::Relaxed).unsigned_abs()
    }
}

impl InteractionParser {
    pub fn parse_interactions(&self, lines: Lines) -> InteractionSet {
        let parsed_interactions = lines
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| p.contains("->") && !p.starts_with("#"))
            .filter_map(|p| self.interaction_regex.captures(p))
            .filter_map(|p| {
                if p.len() >= 3 {
                    let from_participant = Participant {
                        name: p.index(1).trim().to_string(),
                    };
                    let to_participant = Participant {
                        name: p.index(3).trim().to_string(),
                    };
                    let message = if p.len() > 3 {
                        if p.index(4).trim().is_empty() {
                            None
                        } else {
                            Some(Message(p.index(4).trim().to_string()))
                        }
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

        debug!("Parsed interactions: {:?}", parsed_interactions);
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

#[test]
fn test_parse_participants() {
    let parser = ParticipantParser::default();

    // happy path
    let set = parser.parse_participants("One -> Two".lines());
    println!("Partics: {:?}", set);
    assert_eq!(2, set.len());

    //
    let set = parser.parse_participants("One -> Two\nFour ->\nThree -> One\nSeven".lines());
    println!("Partics: {:?}", set);
    assert_eq!(3, set.len());
    assert!(set.contains(&Participant {
        name: "One".to_string()
    }));
    assert!(set.contains(&Participant {
        name: "Two".to_string()
    }));
    assert!(set.contains(&Participant {
        name: "Three".to_string()
    }));

    //
    let set =
        parser.parse_participants("One -> Two\n # Comment -> No Parsing \nFour -> Three".lines());
    println!("Partics: {:?}", set);
    assert_eq!(4, set.len());
    assert!(set.contains(&Participant {
        name: "One".to_string()
    }));
    assert!(set.contains(&Participant {
        name: "Two".to_string()
    }));
    assert!(set.contains(&Participant {
        name: "Three".to_string()
    }));
    assert!(set.contains(&Participant {
        name: "Four".to_string()
    }));
}
