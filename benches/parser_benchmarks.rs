use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sequencer::parser::InteractionParser;

pub fn get_text() -> String {
    String::from(
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
 {AMPS} -> Client: ",
    )
}

fn interaction_parser_benchmark_multiline(c: &mut Criterion) {
    let parser = InteractionParser::default();
    c.bench_function("interaction_parser multiline", |b| {
        b.iter(|| parser.parse_interactions(black_box(get_text().lines())))
    });
}

criterion_group!(benches, interaction_parser_benchmark_multiline);
criterion_main!(benches);
