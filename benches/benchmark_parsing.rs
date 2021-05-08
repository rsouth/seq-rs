use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sequencer::v2::{Diagram, InteractionSet, Parse};
use sequencer::v3::ParticipantParser;

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

fn parse_participant_names_v3(c: &mut Criterion) {
    c.bench_function("parsing participant names", |b| {
        let pp = ParticipantParser::default();
        b.iter(|| pp.parse_participant_names(black_box(get_text().lines())))
    });
}

// fn parse_diagram_multiline(c: &mut Criterion) {
//     c.bench_function("parsing interations multiline", |b| {
//         b.iter(|| InteractionSet::parse(black_box(get_text().lines())))
//     });
// }
//
// fn parse_interaction_multiline(c: &mut Criterion) {
//     c.bench_function("parsing diagram multiline", |b| {
//         b.iter(|| Diagram::parse(black_box(get_text().lines())))
//     });
// }

criterion_group!(
    benches,
    // parse_diagram_multiline,
    // parse_interaction_multiline
    parse_participant_names_v3
);
criterion_main!(benches);
