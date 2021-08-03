use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use sequencer::model::{Config, Source};
use sequencer::parsing::document::DocumentParser;
use sequencer::parsing::interaction::InteractionParser;
use sequencer::parsing::participant::ParticipantParser;
use sequencer::theme::Theme;

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

fn measure_parse_participants(c: &mut Criterion) {
    let config = Config {
        output_path: "".to_string(),
        input_source: Source::Example,
    };
    let document = DocumentParser::parse(&str_to_vec(get_text()), config); // get_text(), config);
    let theme = Theme::default();
    c.bench_function("parsing participants", |b| {
        b.iter(|| ParticipantParser::parse(black_box(&document.lines), black_box(&theme)))
    });
}

fn measure_parse_interactions(c: &mut Criterion) {
    let config = Config {
        input_source: Source::Example,
        output_path: "".to_string(),
    };
    let document = DocumentParser::parse(&str_to_vec(get_text()), config);
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
