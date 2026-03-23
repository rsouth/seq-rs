use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::diagram::Diagram;
use sequencer::model::{Config, Source};
use sequencer::parsing::document::DocumentParser;
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
     Left -> Right"
        .lines()
        .map(|p| p.to_string())
        .collect()
}

fn measure_diagram_parse(c: &mut Criterion) {
    let input = get_text();
    let theme = Theme::default();
    c.bench_function("diagram parse", |b| {
        b.iter(|| {
            let config = Config {
                input_source: Source::Example,
                output_path: String::new(),
            };
            let document = DocumentParser::parse(black_box(&input), config);
            Diagram::parse(document, black_box(theme.clone()))
        })
    });
}

criterion_group!(benches, measure_diagram_parse);
criterion_main!(benches);
