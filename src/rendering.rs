use std::time::Instant;

use font_kit::family_name::FamilyName;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use itertools::Itertools;
use raqote::{Color, DrawOptions, DrawTarget, Path, PathBuilder, Point, SolidSource, Source};

use crate::parser::{Interaction, InteractionSet, Participant};
use crate::rendering::RenderingConstants::{
    DiagramMargin, DiagramPadding, GapBetweenInteractions, ParticipantHGap, ParticipantHeight,
};
use crate::text::{measure_strings, measure_text};
use crate::Diagram;
use font_kit::font::Font;
use std::sync::Arc;

// todo replace any rendered chars in the input text which don't have a glyph in the font with another char

enum RenderingConstants {
    DiagramPadding,
    DiagramMargin,

    // Participant
    ParticipantHeight,
    ParticipantHGap,

    // Interactions
    GapBetweenInteractions,
}

impl RenderingConstants {
    fn value(&self) -> i32 {
        match *self {
            RenderingConstants::DiagramPadding => 25,
            RenderingConstants::DiagramMargin => 15,
            RenderingConstants::ParticipantHeight => 50,
            RenderingConstants::ParticipantHGap => 20,
            RenderingConstants::GapBetweenInteractions => 50,
        }
    }
}

// == Diagram =============================================
impl Diagram {
    pub fn new(interaction_set: InteractionSet) -> Self {
        let unique_participants = Diagram::participant_count(&interaction_set);
        Diagram {
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

// == Rendering Context ===================================
pub struct RenderingContext {
    diagram_width: i32,
    diagram_height: i32,
    theme: Theme,
    draw_target: DrawTarget,
    //
    _title_font: Font,
    participant_font: Font,
}

impl RenderingContext {
    pub fn new(diagram: &Diagram, theme: Theme) -> Self {
        // load fonts
        let _title_font = RenderingContext::get_system_font(theme.title_font_family.as_str());
        let participant_font =
            RenderingContext::get_system_font(theme.participant_font_family.as_str());

        // calculate diagram size
        let diagram_height = RenderingContext::calculate_diagram_height(diagram);
        let diagram_width = RenderingContext::calculate_diagram_width(
            diagram,
            &participant_font,
            theme.participant_font_pt,
        );

        // create canvas
        let draw_target = DrawTarget::new(diagram_width, diagram_height);
        debug!(
            "Created draw target with {}x{}",
            diagram_width, diagram_height
        );
        RenderingContext {
            diagram_width,
            diagram_height,
            theme,
            draw_target,
            _title_font,
            participant_font,
        }
    }

    pub fn calculate_diagram_height(diagram: &Diagram) -> i32 {
        let interaction_count = diagram.interaction_set.len() as i32;
        let height = (DiagramPadding.value() * 2)
            + (DiagramMargin.value() * 2)
            + ParticipantHeight.value()
            + (interaction_count * GapBetweenInteractions.value());
        debug!("Calculated height {}", height);
        height
    }

    // todo doesn't calculate the width added by interaction messages etc
    pub fn calculate_diagram_width(
        diagram: &Diagram,
        font: &Font,
        participant_font_size: f32,
    ) -> i32 {
        let partic_width = measure_strings(font, participant_font_size, &diagram.interaction_set);
        let width = (DiagramPadding.value() * 2)
            + (DiagramMargin.value() * 2)
            + partic_width
            + (diagram.unique_participants * ParticipantHGap.value());
        debug!("Calculated width {}", width);
        width
    }

    #[allow(dead_code)]
    pub fn get_font() -> Font {
        let start = Instant::now();
        let font_data: &[u8] = include_bytes!("../assets/Roboto-Black.ttf");
        let font = Font::from_bytes(Arc::new(font_data.to_vec()), 0).unwrap();
        info!(
            "Loaded font {} in {}ms",
            font.full_name(),
            start.elapsed().as_millis()
        );
        font
    }

    #[allow(dead_code)]
    pub fn get_system_font(family_name: &str) -> Font {
        let start = Instant::now();
        let font = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title(family_name.to_string())],
                &Properties::new().weight(Weight::NORMAL),
            )
            .unwrap()
            .load()
            .unwrap(); // todo...
        info!(
            "Loaded font {} in {}ms",
            font.full_name(),
            start.elapsed().as_millis()
        );
        font
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

fn draw_participant_names(rc: &mut RenderingContext, d: &Diagram) {
    let src = Source::Solid(SolidSource::from(Color::new(200, 25, 100, 30)));
    let draw_options = DrawOptions::default();

    let mut current_pos_x: i32 = DiagramPadding.value() + DiagramMargin.value();
    d.interaction_set
        .iter()
        .map(|p| smallvec::SmallVec::from_buf([&p.from_participant, &p.to_participant]))
        .flatten()
        .unique()
        .for_each(|p: &Participant| {
            let point = Point::new(current_pos_x as f32, 50.); // todo y = padding + margin + fontHeight...
            info!("drawing {} at {}", &p.name, current_pos_x);

            rc.draw_target.draw_text(
                &rc.participant_font,
                rc.theme.participant_font_pt,
                &p.name,
                point,
                &src,
                &draw_options,
            );

            current_pos_x += ParticipantHGap.value()
                + measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.name)
                    .unwrap()
                    .width();
        });
}

pub struct Theme {
    title_font_family: String,
    _title_font_pt: f32,

    participant_font_family: String,
    participant_font_pt: f32,
}

pub fn do_render(diagram: &Diagram) {
    let theme = Theme {
        title_font_family: "Arial".to_string(),
        _title_font_pt: 40.,
        participant_font_family: "Arial".to_string(),
        participant_font_pt: 20.,
    };
    let mut rendering_context = RenderingContext::new(&diagram, theme);

    let options = DrawOptions::default();

    let start = Instant::now();
    draw_participant_names(&mut rendering_context, diagram);
    debug!(
        "Drew participant names in {}µs",
        start.elapsed().as_micros()
    );

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
