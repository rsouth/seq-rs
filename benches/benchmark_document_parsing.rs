use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use sequencer::model::{Config, Source};
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

fn str_to_vec(s: &str) -> Vec<String> {
    s.lines().into_iter().map(|p| p.to_string()).collect_vec()
}

fn measure_parse_document(c: &mut Criterion) {
    let input = str_to_vec(get_text());
    let config = Config {
        input_source: Source::Example,
        output_path: "".to_string(),
    };
    c.bench_function("parsing document", |b| {
        b.iter(|| DocumentParser::parse(black_box(&input), black_box(config.clone())))
    });
}

criterion_group!(benches, measure_parse_document);
criterion_main!(benches);
