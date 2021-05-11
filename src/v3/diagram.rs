use std::{
    collections::HashMap,
    sync::atomic::{AtomicI32, Ordering},
};

use Ordering::Relaxed;

use crate::v3::model::{Header, Line};
use crate::v3::{model::LineContents, theme::Theme};
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
    pub fn parse(document: &[Line]) -> Diagram {
        println!("Document: {:?}", document);

        // Pass #1 - participants
        // iterate lines, looking only at Interaction types
        // note down the first appearance of a Participant
        //  -> this is it's index, and it's activation_start
        // note down the last appearange of a Participant
        //  -> this is it's activation_end
        let p_idx = AtomicI32::new(0);
        let i_idx = AtomicI32::new(0);
        let mut idx_for_p: HashMap<String, i32> = HashMap::new();
        let mut start_idx_for_p: HashMap<String, i32> = HashMap::new();
        let mut end_idx_for_p: HashMap<String, i32> = HashMap::new();
        document
            .iter()
            .filter(|line| {
                matches!(
                    line.line_contents,
                    LineContents::Interaction(_, _) | LineContents::InteractionWithMessage(_, _, _)
                )
            })
            .for_each(|line| {
                println!("Pass 1: {:#?}", line);
                match &line.line_contents {
                    LineContents::Interaction(f, t)
                    | LineContents::InteractionWithMessage(f, t, _) => {
                        // participant index
                        if !idx_for_p.contains_key(&f.0) {
                            idx_for_p.insert(f.0.to_string(), p_idx.fetch_add(1, Relaxed));
                        }

                        if !idx_for_p.contains_key(&t.0) {
                            idx_for_p.insert(t.0.to_string(), p_idx.fetch_add(1, Relaxed));
                        }

                        // active start index
                        if !start_idx_for_p.contains_key(&f.0) {
                            start_idx_for_p.insert(f.0.to_string(), i_idx.load(Relaxed));
                        }

                        if !start_idx_for_p.contains_key(&t.0) {
                            start_idx_for_p.insert(t.0.to_string(), i_idx.load(Relaxed));
                        }

                        // active end index
                        end_idx_for_p.insert(f.0.to_string(), i_idx.load(Relaxed));
                        end_idx_for_p.insert(t.0.to_string(), i_idx.load(Relaxed));
                    }
                    _ => {
                        panic!("...");
                    }
                }

                i_idx.fetch_add(1, Relaxed);
            });

        info!("After first pass:");
        info!("Participant idx: {:#?}", idx_for_p);
        info!("Participant active from: {:#?}", start_idx_for_p);
        info!("Participant active to: {:#?}", end_idx_for_p);

        // Pass #2 - interactions
        // iterate lines, looking only at Interaction types
        // note if an interaction is L2R, R2L, SelfRef etc.
        // note if an interaction is Message vs Reply
        // note if an interaction is Sync vs Async

        // _document
        //     .iter()
        //     .filter(|line| match line.line_contents {
        //         LineContents::Interaction(_, _) => true,
        //         LineContents::InteractionWithMessage(_, _, _) => true,
        //         _ => false,
        //     })
        //     .for_each(|thing| {
        //         println!("Thing2: {:#?}", thing);
        //     });

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
