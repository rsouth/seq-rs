use std::sync::Arc;
use std::time::Instant;

use euclid::Point2D;
use font_kit::canvas::RasterizationOptions;
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::loaders::directwrite::Font;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use itertools::Itertools;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::vec2f;
use raqote::{Color, DrawOptions, DrawTarget, Path, PathBuilder, Point, SolidSource, Source};

use crate::rendering::RenderingConstants::{
    DiagramPadding, GapBetweenInteractions, ParticipantHeight,
};
use crate::Diagram;
use std::ops::AddAssign;

// todo replace any rendered chars in the input text which don't have a glyph in the font with another char

enum RenderingConstants {
    DiagramPadding,

    // Participant
    ParticipantHeight,

    // Interactions
    GapBetweenInteractions,
}

impl RenderingConstants {
    fn value(&self) -> i32 {
        match *self {
            RenderingConstants::DiagramPadding => 10,
            RenderingConstants::ParticipantHeight => 50,
            RenderingConstants::GapBetweenInteractions => 100,
        }
    }
}

pub struct RenderingContext {
    font: Font,
    participant_font_size: f32,
}

impl Default for RenderingContext {
    fn default() -> Self {
        RenderingContext {
            font: get_system_font(),
            participant_font_size: 40.,
        }
    }
}

pub fn calculate_diagram_height(diagram: &Diagram) -> i32 {
    let start = Instant::now();

    let interaction_count = diagram.0.len() as i32;
    let height = DiagramPadding.value()
        + ParticipantHeight.value()
        + (GapBetweenInteractions.value() * interaction_count);
    debug!(
        "Calculated height {} in {}µs",
        height,
        start.elapsed().as_micros()
    );
    height
}

pub fn calculate_diagram_width(rc: &RenderingContext, diagram: &Diagram) -> i32 {
    let start = Instant::now();

    let partic_width: i32 = diagram
        .0
        .iter()
        .map(|p| vec![&p.from_participant.name, &p.to_participant.name])
        .flatten()
        .map(|p| measure_text(&rc.font, rc.participant_font_size, p).unwrap())
        .map(|p| p.width())
        .sum();

    let width = DiagramPadding.value()
        + (participant_count(diagram) * DiagramPadding.value())
        + partic_width;
    debug!(
        "Calculated width {} in {}µs",
        width,
        start.elapsed().as_micros()
    );
    width
}

fn participant_count(diagram: &Diagram) -> i32 {
    diagram
        .0
        .iter()
        .map(|p| vec![&p.from_participant, &p.to_participant])
        .flatten()
        .unique()
        .count() as i32
}

#[allow(dead_code)]
fn get_font() -> Font {
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
pub fn get_system_font() -> Font {
    let start = Instant::now();
    let font = SystemSource::new()
        .select_best_match(
            &[FamilyName::Title("Arial".into())],
            &Properties::new().weight(Weight::MEDIUM),
        )
        .unwrap()
        .load()
        .unwrap();
    info!(
        "Loaded font {} in {}ms",
        font.full_name(),
        start.elapsed().as_millis()
    );
    font
}

fn rect_path(width: f32, height: f32) -> Path {
    let start = Instant::now();
    let x = DiagramPadding.value() as f32;
    let y = DiagramPadding.value() as f32;
    let w = width as f32 - (2. * DiagramPadding.value() as f32);
    let h = height as f32 - (2. * DiagramPadding.value() as f32);
    let mut r = PathBuilder::new();
    r.rect(x, y, w, h);

    let rpath = r.finish();
    debug!("Created Rect path in {}µs", start.elapsed().as_micros());
    rpath
}

pub fn measure_text(
    font: &Font,
    point_size: f32,
    text: &str,
) -> Result<euclid::Rect<i32, euclid::UnknownUnit>, font_kit::error::GlyphLoadingError> {
    measure_glyphs(
        font,
        point_size,
        text.chars().scan(vec2f(0., 0.), |start, c| {
            let id = font.glyph_for_char(c).unwrap();
            let position = Point::new(start.x(), start.y());
            *start += font.advance(id).unwrap() * point_size / 24. / 96.;

            Some((id, position))
        }),
    )
}

pub fn measure_glyphs(
    font: &Font,
    point_size: f32,
    glyphs: impl IntoIterator<Item = (u32, Point)>,
) -> Result<euclid::Rect<i32, euclid::UnknownUnit>, font_kit::error::GlyphLoadingError> {
    let mut combined_bounds = euclid::Rect::zero();
    for (id, position) in glyphs.into_iter() {
        let bounds = font.raster_bounds(
            id,
            point_size,
            // tform,
            Transform2F::default().translate(vec2f(position.x, position.y)),
            HintingOptions::None,
            RasterizationOptions::SubpixelAa,
        );
        let bounds = bounds?;
        let origin = bounds.origin();
        let size = bounds.size();
        let bounds = euclid::Rect::new(
            Point2D::new(origin.x(), origin.y()),
            euclid::Size2D::new(size.x(), size.y()),
        );
        combined_bounds = combined_bounds.union(&bounds);
    }

    Ok(combined_bounds)
}

fn draw_participant_names(dt: &mut DrawTarget, d: &Diagram) {
    let font = get_system_font();
    let src = Source::Solid(SolidSource::from(Color::new(200, 150, 30, 30)));

    let mut count = 0;
    d.0.iter()
        .map(|p| vec![&p.from_participant, &p.to_participant])
        .flatten()
        .unique()
        .for_each(|p| {
            count.add_assign(1);
            let x = DiagramPadding.value()
                + measure_text(&font, 40., &p.name).unwrap().width()
                + (count * DiagramPadding.value());
            let point = Point::new(x as f32, 10.);
            dt.draw_text(&font, 40., &p.name, point, &src, &DrawOptions::new())
        });
}

pub fn do_render(diagram: &Diagram) {
    let rendering_context = RenderingContext::default();

    let height = calculate_diagram_height(diagram);
    let width = calculate_diagram_width(&rendering_context, diagram);

    let start = Instant::now();
    let mut dt = DrawTarget::new(width, height);
    debug!("Created DrawTarget in {}µs", start.elapsed().as_micros());

    draw_participant_names(&mut dt, diagram);

    let rpath = rect_path(width as f32, height as f32);

    let start = Instant::now();
    dt.fill(
        &rpath,
        &Source::Solid(SolidSource::from(Color::new(200, 150, 30, 30))),
        &DrawOptions::new(),
    );
    debug!(
        "Filled rect path in {}µs ({}ms)",
        start.elapsed().as_micros(),
        start.elapsed().as_millis()
    );

    // let font = get_font();
    let font = get_system_font();
    let result = measure_text(&font, 40.0, "AAA");

    println!("Result is: {:?}", result);

    let start = Instant::now();
    let text = "HCenter";
    let textwidth = measure_text(&font, 24., text);
    let hcenterx = (width as f32 / 2.) - (textwidth.unwrap().width() as f32 / 2.);
    dt.draw_text(
        &font,
        24.,
        text,
        Point::new(hcenterx, 30.),
        &Source::Solid(SolidSource {
            r: 0,
            g: 0,
            b: 0xff,
            a: 0xff,
        }),
        &DrawOptions::new(),
    );
    debug!(
        "Drew text {} in {}µs ({}ms)",
        text,
        start.elapsed().as_micros(),
        start.elapsed().as_millis()
    );

    let start = Instant::now();
    dt.write_png("example.png").unwrap();
    debug!(
        "Wrote image to disk in {}µs ({}ms)",
        start.elapsed().as_micros(),
        start.elapsed().as_millis()
    );
}

#[test]
fn test_measure_text() {
    let font = get_system_font();
    let result = measure_text(&font, 20., "A");
    print!("{:?}", result);
    let result = measure_text(&font, 20., "AA");
    print!("{:?}", result);
    let result = measure_text(&font, 20., "AAA");
    print!("{:?}", result);
    let result = measure_text(&font, 20., "AAAA");
    print!("{:?}", result);
}
