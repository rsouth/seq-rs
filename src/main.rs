#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::time::Instant;

use sequencer::parser::InteractionParser;
use sequencer::{get_text, parse_diagram};

fn main() {
    pretty_env_logger::init();

    let instant = Instant::now();

    // use_interaction_parser();

    let diagram = parse_diagram(get_text().lines());
    info!("{:?}", diagram);

    sequencer::rendering::do_render(&diagram);

    info!(
        "Finished in {} micros ({}ms)",
        instant.elapsed().as_micros(),
        instant.elapsed().as_millis()
    );
}

#[allow(dead_code)]
fn use_interaction_parser() {
    let interaction_parser = InteractionParser::default();
    let interactions = interaction_parser.parse_interactions(get_text().lines());
    debug!("Got interactions: {:?}", interactions);
}
