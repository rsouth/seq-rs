use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sequencer::parser::{Interaction, InteractionSet, Participant};
use sequencer::render_context::RenderingContext;
use sequencer::rendering::do_render;
use sequencer::text::measure_text;
use sequencer::Diagram;

fn measure_text_single_char(c: &mut Criterion) {
    let font = RenderingContext::get_system_font("Arial");
    c.bench_function("measure_text single", |b| {
        b.iter(|| measure_text(black_box(&font), black_box(20.), black_box("A")))
    });
}

fn measure_calculate_diagram_height(c: &mut Criterion) {
    let p: InteractionSet = vec![Interaction {
        from_participant: Participant {
            name: "One".to_string(),
        },
        to_participant: Participant {
            name: "Two".to_string(),
        },
        message: None,
        order: 0,
    }];
    let diagram: Diagram = Diagram::new(p);
    c.bench_function("calculate_diagram_height", |b| {
        b.iter(|| RenderingContext::calculate_diagram_height(black_box(&diagram)));
    });
}

fn measure_calculate_diagram_width(c: &mut Criterion) {
    let p: InteractionSet = vec![Interaction {
        from_participant: Participant {
            name: "One".to_string(),
        },
        to_participant: Participant {
            name: "Two".to_string(),
        },
        message: None,
        order: 0,
    }];
    let diagram: Diagram = Diagram::new(p);
    let font = &RenderingContext::get_system_font("Arial");
    c.bench_function("calculate_diagram_width", |b| {
        b.iter(|| {
            RenderingContext::calculate_diagram_width(
                black_box(&diagram),
                black_box(&font),
                black_box(40.),
            )
        })
    });
}

fn measure_draw_partic_names(c: &mut Criterion) {
    let p: InteractionSet = vec![Interaction {
        from_participant: Participant {
            name: "One".to_string(),
        },
        to_participant: Participant {
            name: "Two".to_string(),
        },
        message: None,
        order: 0,
    }];
    let diagram: Diagram = Diagram::new(p);
    c.bench_function("measure_draw_partic_names", |b| {
        b.iter(|| do_render(black_box(&diagram)))
    });
}

criterion_group!(
    benches,
    measure_text_single_char,
    measure_calculate_diagram_height,
    measure_calculate_diagram_width
    measure_draw_partic_names
);
criterion_main!(benches);
