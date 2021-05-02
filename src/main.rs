#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::time::Instant;

use sequencer::get_text;
use sequencer::rendering::render_context::{RenderingContext, Theme};
use sequencer::v2::*;

fn main() {
    pretty_env_logger::init();

    let instant = Instant::now();

    let diagram = Diagram::parse(get_text().lines());
    let diagram = diagram.unwrap();

    let theme = Theme::default();
    let mut rendering_context = RenderingContext::create(&diagram, theme);

    diagram.draw(&mut rendering_context).unwrap();

    info!(
        "Finished in {} micros ({}ms)",
        instant.elapsed().as_micros(),
        instant.elapsed().as_millis()
    );
}
