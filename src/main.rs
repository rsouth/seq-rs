#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::time::Instant;

use sequencer::v3;

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

    let diagram = v3::parsing::DocumentParser::parse(get_text());
    info!("Document: {:#?}", diagram);

    let diagram_parser = v3::diagram::Diagram::parse(&diagram);
    info!("Diagram: {:#?}", diagram_parser);

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
     {AMPS} -> Client: "
}
