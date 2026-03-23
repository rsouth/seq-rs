use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::model::{Config, Source};
use sequencer::parsing::document::DocumentParser;
use sequencer::parsing::interaction::InteractionParser;
use sequencer::parsing::participant::ParticipantParser;
use sequencer::theme::Theme;

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

fn measure_parse_participants(c: &mut Criterion) {
    let config = Config {
        output_path: String::new(),
        input_source: Source::Example,
    };
    let document = DocumentParser::parse(&get_text(), config);
    let theme = Theme::default();
    c.bench_function("parsing participants", |b| {
        b.iter(|| ParticipantParser::parse(black_box(&document.lines), black_box(&theme)))
    });
}

fn measure_parse_interactions(c: &mut Criterion) {
    let config = Config {
        input_source: Source::Example,
        output_path: String::new(),
    };
    let document = DocumentParser::parse(&get_text(), config);
    let theme = Theme::default();
    let participants = ParticipantParser::parse(&document.lines, &theme);

    c.bench_function("parsing interactions", |b| {
        b.iter(|| InteractionParser::parse(black_box(&document.lines), black_box(&participants)))
    });
}

criterion_group!(
    benches,
    measure_parse_participants,
    measure_parse_interactions,
);
criterion_main!(benches);
