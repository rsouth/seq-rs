use euclid::Point2D;
use font_kit::canvas::RasterizationOptions;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use itertools::Itertools;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::vec2f;
use raqote::Point;
use smallvec::SmallVec;

use crate::v2::{Interaction, Participant};

// todo generify; take a slice of strings rather than an interaction set
pub fn measure_all_participants(
    font: &Font,
    font_size: f32,
    interaction_set: &[Interaction],
) -> i32 {
    let sum_of_width = interaction_set
        .iter()
        .map(|p| SmallVec::from_buf([&p.from_participant, &p.to_participant]))
        .flatten()
        .unique()
        .map(|p: &Participant| {
            let width = measure_text(font, font_size, &p.name).unwrap().width();
            debug!("Width of {} is {}", &p.name, width);
            width
        })
        .sum();

    debug!("Full participant width {}", sum_of_width);
    sum_of_width
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
    let tfm = Transform2F::default();
    for (id, position) in glyphs.into_iter() {
        let bounds = font.raster_bounds(
            id,
            point_size,
            tfm.translate(vec2f(position.x, position.y)),
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

// #[test]
// fn test_measure_text() {
//     use crate::render_context::RenderingContext;
//     // just testing using 'known good' values incase measuring itself gets broken
//
//     let font = RenderingContext::get_font();
//     let result = measure_text(&font, 20., "A");
//     assert_eq!(15, result.unwrap().width());
//     assert_eq!(14, result.unwrap().height());
//
//     let result = measure_text(&font, 20., "AA");
//     assert_eq!(28, result.unwrap().width());
//     assert_eq!(14, result.unwrap().height());
// }
