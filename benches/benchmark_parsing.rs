use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;

use sequencer::v3::parsing::{DocumentParser, InteractionParser, ParticipantParser};

pub fn get_text_vec() -> Vec<&'static str> {
    get_text().lines().collect_vec()
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

fn measure_parse_document(c: &mut Criterion) {
    let input = get_text();
    c.bench_function("parsing document", |b| {
        b.iter(|| DocumentParser::parse(black_box(input)))
    });
}

criterion_group!(
    benches,
    measure_parse_participants,
    measure_parse_interactions,
    measure_parse_document
);
criterion_main!(benches);
