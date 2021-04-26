use sequencer::{get_text, InteractionParser};
use std::time::Instant;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

fn main() {
    pretty_env_logger::init();

    let instant = Instant::now();

    // let parser = ParticipantParser::default();
    // debug!(
    //     "Initialised participant parser in {}ms",
    //     instant.elapsed().as_millis()
    // );
    // let set = parser.parse_participants(get_text().lines());
    //
    // debug!("Got participants: {:?}", set);

    let interaction_parser = InteractionParser::default();
    debug!(
        "Initialised interaction parser in {}ms",
        instant.elapsed().as_millis()
    );
    let interactions = interaction_parser.parse_interactions(get_text().lines());
    debug!("Got interactions: {:?}", interactions);

    info!(
        "Finished in {} micros ({}ms)",
        instant.elapsed().as_micros(),
        instant.elapsed().as_millis()
    );
}
