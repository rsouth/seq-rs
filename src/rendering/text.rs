use itertools::Itertools;
use smallvec::SmallVec;

use crate::rendering::render_context::RenderingContext;
use crate::v2::{Interaction, Participant};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;

pub fn draw_text(rc: &mut RenderingContext, content: &str, x: f32, y: f32, px: f32) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x,
        y,
        ..LayoutSettings::default()
    });
    let font = &rc.participant_font;
    layout.append(&[font], &TextStyle::new(&content, px, 0));
    for glyph in layout.glyphs() {
        let (metrics, coverage) = font.rasterize(glyph.key.c, px);
        debug!("Metrics: {:?}", glyph);

        let mut image_data = Vec::with_capacity(coverage.len());
        for cov in coverage.iter() {
            let pixel = rgb_to_u32(0, 0, 0, *cov as usize);
            image_data.push(pixel);
        }
        rc.draw_target.draw_image_at(
            glyph.x,
            glyph.y,
            &raqote::Image {
                width: metrics.width as i32,
                height: metrics.height as i32,
                data: &image_data,
            },
            &raqote::DrawOptions {
                blend_mode: raqote::BlendMode::Overlay,
                alpha: 1.0,
                antialias: raqote::AntialiasMode::Gray,
            },
        );
    }
}

pub fn rgb_to_u32(red: usize, green: usize, blue: usize, alpha: usize) -> u32 {
    let r = red.clamp(0, 255);
    let g = green.clamp(0, 255);
    let b = blue.clamp(0, 255);
    let a = alpha.clamp(0, 255);
    ((a << 24) | (r << 16) | (g << 8) | (b << 0)) as u32
}

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
            let width = measure_text(font, font_size, &p.name).width;
            debug!("Width of {} is {}", &p.name, width);
            width
        })
        .sum();

    debug!("Full participant width {}", sum_of_width);
    sum_of_width
}

pub struct Size {
    pub height: i32,
    pub width: i32,
}

pub fn measure_text(font: &Font, size: f32, text: &str) -> Size {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0_f32,
        y: 0_f32,
        ..LayoutSettings::default()
    });
    layout.append(&[font], &TextStyle::new(text, size, 0));
    let w = layout.glyphs().iter().map(|g| g.width as i32).sum();
    let h = layout.glyphs().iter().map(|g| g.height as i32).sum();
    // todo merge the above 2 iters
    Size {
        width: w,
        height: h,
    }
}

// pub fn measure_text(
//     font: &Font,
//     point_size: f32,
//     text: &str,
// ) -> Result<euclid::Rect<i32, euclid::UnknownUnit>, font_kit::error::GlyphLoadingError> {
//     measure_glyphs(
//         font,
//         point_size,
//         text.chars().scan(vec2f(0., 0.), |start, c| {
//             let id = font.glyph_for_char(c).unwrap();
//             let position = Point::new(start.x(), start.y());
//             *start += font.advance(id).unwrap() * point_size / 24. / 96.;
//
//             Some((id, position))
//         }),
//     )
// }

// pub fn measure_glyphs(
//     font: &Font,
//     point_size: f32,
//     glyphs: impl IntoIterator<Item = (u32, Point)>,
// ) -> Result<euclid::Rect<i32, euclid::UnknownUnit>, font_kit::error::GlyphLoadingError> {
//     let mut combined_bounds = euclid::Rect::zero();
//     let tfm = Transform2F::default();
//     for (id, position) in glyphs.into_iter() {
//         let bounds = font.raster_bounds(
//             id,
//             point_size,
//             tfm.translate(vec2f(position.x, position.y)),
//             HintingOptions::None,
//             RasterizationOptions::SubpixelAa,
//         );
//         let bounds = bounds?;
//         let origin = bounds.origin();
//         let size = bounds.size();
//         let bounds = euclid::Rect::new(
//             Point2D::new(origin.x(), origin.y()),
//             euclid::Size2D::new(size.x(), size.y()),
//         );
//         combined_bounds = combined_bounds.union(&bounds);
//     }
//
//     Ok(combined_bounds)
// }

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
