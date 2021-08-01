use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::parsing::document::DocumentParser;

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

fn measure_parse_document(c: &mut Criterion) {
    let input = get_text();
    c.bench_function("parsing document", |b| {
        b.iter(|| DocumentParser::parse(black_box(input)))
    });
}

criterion_group!(benches, measure_parse_document);
criterion_main!(benches);
