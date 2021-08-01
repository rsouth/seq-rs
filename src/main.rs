#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use sequencer::diagram::Diagram;
use sequencer::parsing::document::DocumentParser;
use sequencer::theme::Theme;
use std::time::Instant;

fn main() {
    pretty_env_logger::init();

    let instant = Instant::now();

    // let diagram = Diagram::parse(get_text().lines());
    // let diagram = diagram.unwrap();

    // let theme = Theme::default();
    // let mut diagram = diagram.create(theme).unwrap();

    // let mut rendering_context = RenderingContext::create(&diagram, theme);

    // diagram.draw().unwrap();

    // diagram.save_png("example.png");

    let document = DocumentParser::parse(get_text());
    info!("Document: {:#?}", document);

    let theme = Theme::default();

    let diagram = Diagram::parse(document, theme);
    info!("Diagram: {:#?}", diagram);

    diagram.render();

    info!(
        "Finished in {} micros ({}ms)",
        instant.elapsed().as_micros(),
        instant.elapsed().as_millis()
    );
}

pub fn get_text() -> &'static str {
    ":theme Default
     :title Example Sequence Diagram
     :author Mr. Sequence Diagram
     :date
    
     # diagram
     Client -> Server: Request
     Server -> Server: Parses request
     Server ->> Service: Query
     Service -->> Server: Data
     Server --> Client: Response
     Left -> Right
     # {AMPS} -> Client: "
}
