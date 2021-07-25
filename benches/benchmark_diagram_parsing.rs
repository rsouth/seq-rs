use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use sequencer::parsing::document::DocumentParser;
use sequencer::parsing::interaction::InteractionParser;
use sequencer::parsing::participant::ParticipantParser;

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

fn measure_parse_participants(c: &mut Criterion) {
    let document = DocumentParser::parse(get_text());

    c.bench_function("parsing participants", |b| {
        b.iter(|| ParticipantParser::parse(black_box(&document)))
    });
}

fn measure_parse_interactions(c: &mut Criterion) {
    let document = DocumentParser::parse(get_text());
    let participants = ParticipantParser::parse(&document);

    c.bench_function("parsing interactions", |b| {
        b.iter(|| InteractionParser::parse(black_box(&document), black_box(&participants)))
    });
}

criterion_group!(
    benches,
    measure_parse_participants,
    measure_parse_interactions,
);
criterion_main!(benches);
