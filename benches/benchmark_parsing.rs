use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;

use sequencer::v3::parsing::{DocumentParser, ParticipantParser};

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

fn parse_participant_names_v3(c: &mut Criterion) {
    let parser = ParticipantParser::default();
    let input = get_text_vec();

    c.bench_function("parsing participant names", |b| {
        b.iter(|| parser.parse_participant_names(black_box(&input)))
    });
}

fn parse_participants_v3(c: &mut Criterion) {
    let parser = ParticipantParser::default();
    let input = get_text_vec();
    let participant_names = parser.parse_participant_names(&input);

    c.bench_function("parsing participants", |b| {
        b.iter(|| parser.parse_participants(black_box(&input), black_box(&participant_names)))
    });
}

fn participant_parser_v3(c: &mut Criterion) {
    let parser = ParticipantParser::default();
    let input = get_text();

    c.bench_function("participant parser", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
}

fn document_parser_v3(c: &mut Criterion) {
    let input = get_text();

    let dp = DocumentParser::default();

    c.bench_function("document parser", |b| b.iter(|| dp.parse(black_box(input))));
}

criterion_group!(
    benches,
    parse_participant_names_v3,
    parse_participants_v3,
    participant_parser_v3,
    document_parser_v3
);
criterion_main!(benches);
