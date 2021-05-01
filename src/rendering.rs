use std::time::Instant;

use itertools::Itertools;
use raqote::{Color, DrawOptions, Path, PathBuilder, Point, SolidSource, Source, StrokeStyle};

use crate::parser::{Interaction, InteractionSet, Participant};
use crate::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, ParticipantHGap, ParticipantHeight, ParticipantPadding,
};
use crate::render_context::RenderingContext;
use crate::text::measure_text;
use crate::Diagram;

// todo replace any rendered chars in the input text which don't have a glyph in the font with another char

// == Diagram =============================================
impl Diagram {
    pub fn new(theme: Theme, interaction_set: InteractionSet) -> Self {
        let unique_participants = Diagram::participant_count(&interaction_set);
        Diagram {
            theme,
            unique_participants,
            interaction_set,
        }
    }

    fn participant_count(interactions: &[Interaction]) -> i32 {
        let i = interactions
            .iter()
            .map(|p| vec![&p.from_participant, &p.to_participant])
            .flatten()
            .unique()
            .count() as i32;
        debug!("Unique participant count: {}", i);
        i as i32
    }
}

fn rect_path(width: i32, height: i32) -> Path {
    let start = Instant::now();
    let rect_x = DiagramPadding.value() as f32;
    let rect_y = DiagramPadding.value() as f32;
    let rect_w = width as f32 - (2. * DiagramPadding.value() as f32);
    let rect_h = height as f32 - (2. * DiagramPadding.value() as f32);
    let mut rect_path = PathBuilder::new();
    rect_path.rect(rect_x, rect_y, rect_w, rect_h);

    let rpath = rect_path.finish();
    debug!("Created Rect path in {}µs", start.elapsed().as_micros());
    rpath
}

fn draw_participant_rectangles(rc: &mut RenderingContext, d: &Diagram) {
    let mut rect_path = PathBuilder::new();
    let mut x_pos = DiagramPadding.value() + DiagramMargin.value();
    let y_pos = DiagramPadding.value() + DiagramMargin.value();
    d.interaction_set
        .iter()
        .map(|p| smallvec::SmallVec::from_buf([&p.from_participant, &p.to_participant]))
        .flatten()
        .unique()
        .for_each(|p: &Participant| {
            let rect =
                measure_text(&rc.participant_font, d.theme.participant_font_pt, &p.name).unwrap();
            rect_path.rect(
                x_pos as f32,
                y_pos as f32,
                rect.width() as f32 + 20 as f32,
                ParticipantHeight.value() as f32, //rect.height() as f32 + 20 as f32,
            );

            x_pos += rect.width() + 20 + ParticipantHGap.value();
        });

    let mut stroke = StrokeStyle::default();
    stroke.width = 2.;
    let color = Source::Solid(SolidSource::from_unpremultiplied_argb(255, 100, 200, 100));
    rc.draw_target
        .stroke(&rect_path.finish(), &color, &stroke, &DrawOptions::new());
}

fn draw_participant_names(rc: &mut RenderingContext, d: &Diagram) {
    let src = Source::Solid(SolidSource::from(Color::new(255, 0, 0, 0)));
    let draw_options = DrawOptions::default();

    let font_height = measure_text(&rc.participant_font, d.theme.participant_font_pt, "A");
    let y_position = DiagramPadding.value() + DiagramMargin.value(); // + font_height.unwrap().height();
    let mut current_pos_x: i32 = DiagramPadding.value() + DiagramMargin.value();
    d.interaction_set
        .iter()
        .map(|p| smallvec::SmallVec::from_buf([&p.from_participant, &p.to_participant]))
        .flatten()
        .unique()
        .for_each(|p: &Participant| {
            let partic_measure =
                measure_text(&rc.participant_font, d.theme.participant_font_pt, &p.name).unwrap();
            let point = Point::new(
                (current_pos_x + ParticipantPadding.value()) as f32,
                y_position as f32
                    + ParticipantPadding.value() as f32
                    + font_height.unwrap().height() as f32,
            ); // todo y = padding + margin + fontHeight...
            info!("drawing {} at {}", &p.name, current_pos_x);

            rc.draw_target.draw_text(
                &rc.participant_font,
                d.theme.participant_font_pt,
                &p.name,
                point,
                &src,
                &draw_options,
            );

            current_pos_x +=
                (ParticipantPadding.value() * 2) + ParticipantHGap.value() + partic_measure.width();
        });
}

struct RenderableParticipant {
    participant: Participant,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

#[derive(Debug)]
pub struct Theme {
    pub(crate) title_font_family: String,
    pub(crate) _title_font_pt: f32,

    pub(crate) participant_font_family: String,
    pub(crate) participant_font_pt: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title_font_family: "Arial".to_string(),
            _title_font_pt: 54.0,
            participant_font_family: "Arial".to_string(),
            participant_font_pt: 40.0,
        }
    }
}

pub fn do_render(diagram: &Diagram) {
    let mut rendering_context = RenderingContext::new(&diagram);

    let options = DrawOptions::default();

    let start = Instant::now();
    let rpath = rect_path(
        rendering_context.diagram_width,
        rendering_context.diagram_height,
    );
    rendering_context.draw_target.fill(
        &rpath,
        &Source::Solid(SolidSource::from(Color::new(200, 150, 30, 30))),
        &options,
    );
    debug!(
        "Filled rect path in {}µs ({}ms)",
        start.elapsed().as_micros(),
        start.elapsed().as_millis()
    );

    let start = Instant::now();
    draw_participant_names(&mut rendering_context, diagram);
    debug!(
        "Drew participant names in {}µs",
        start.elapsed().as_micros()
    );

    draw_participant_rectangles(&mut rendering_context, diagram);

    let start = Instant::now();
    rendering_context
        .draw_target
        .write_png("example.png")
        .unwrap();
    debug!(
        "Wrote image to disk in {}µs ({}ms)",
        start.elapsed().as_micros(),
        start.elapsed().as_millis()
    );
}
