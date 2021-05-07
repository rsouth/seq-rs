use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::rendering::render_context::{RenderingContext, Theme};
use sequencer::v2::{Diagram, Parse};

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

// fn create_diagram_multiline(c: &mut Criterion) {
//     let diagram = Diagram::parse(get_text().lines()).unwrap();
//     let mut d = &diagram;
//
//     c.bench_function("create diagram multiline", |b| {
//         b.iter(|| d.create(black_box(Theme::default())))
//     });
// }

fn render_diagram_multiline(c: &mut Criterion) {
    let diagram = Diagram::parse(get_text().lines()).unwrap();
    let mut diagram = diagram.create(Theme::default()).unwrap();

    c.bench_function("rendering diagram multiline", |b| b.iter(|| diagram.draw()));
}

fn measure_calculate_diagram_width(c: &mut Criterion) {
    let diagram = Diagram::parse(get_text().lines()).unwrap();
    let diagram = diagram.create(Theme::default()).unwrap();
    let font = RenderingContext::get_font("Arial");
    c.bench_function("calculate diagram width", |b| {
        b.iter(|| {
            RenderingContext::calculate_diagram_width(
                black_box(&diagram.interactions),
                black_box(&diagram.participants),
                black_box(&font),
                black_box(40_f32),
            )
        })
    });
}

fn measure_calculate_diagram_height(c: &mut Criterion) {
    let diagram = Diagram::parse(get_text().lines()).unwrap();
    let diagram = diagram.create(Theme::default()).unwrap();
    let font = RenderingContext::get_font("Arial");
    c.bench_function("calculate diagram height", |b| {
        b.iter(|| {
            RenderingContext::calculate_diagram_height(
                black_box(&diagram.interactions),
                black_box(&font),
                black_box(40_f32),
            )
        })
    });
}

criterion_group!(
    benches,
    // create_diagram_multiline,
    render_diagram_multiline,
    measure_calculate_diagram_width,
    measure_calculate_diagram_height
);
criterion_main!(benches);
