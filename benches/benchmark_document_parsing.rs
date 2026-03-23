use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::model::{Config, Source};
use sequencer::parsing::document::DocumentParser;

fn get_text() -> Vec<String> {
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
        .lines()
        .map(|p| p.to_string())
        .collect()
}

fn measure_parse_document(c: &mut Criterion) {
    let input = get_text();
    let config = Config {
        input_source: Source::Example,
        output_path: String::new(),
    };
    c.bench_function("parsing document", |b| {
        b.iter(|| DocumentParser::parse(black_box(&input), black_box(config.clone())))
    });
}

criterion_group!(benches, measure_parse_document);
criterion_main!(benches);
