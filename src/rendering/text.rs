use super::{Rect, RenderContext};
use crate::theme::Theme;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

#[cfg(debug_assertions)]
use raqote::{DrawOptions, PathBuilder, SolidSource, Source, StrokeStyle};

pub fn measure_string(theme: &Theme, content: &str, px: usize) -> Rect {
    debug_assert!(!content.is_empty());
    debug_assert!(px > 0);
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.0,
        y: 0.0,
        ..LayoutSettings::default()
    });
    let font = &theme.body_font;
    layout.append(&[font], &TextStyle::new(content, px as f32, 0));

    let layout = layout.glyphs();
    let first_glyph = layout.first().unwrap();
    let last_glyph = layout.last().unwrap();
    let y = layout.iter().map(|glyph| glyph.y as usize).min().unwrap();
    let h = layout.iter().map(|glyph| glyph.height).max().unwrap() as usize;
    Rect {
        x: first_glyph.x as usize,
        y,
        w: ((first_glyph.x + last_glyph.x) as usize + last_glyph.width),
        h,
    }
}

pub fn draw_text(rc: &mut RenderContext, content: &str, x: usize, y: usize, px: usize) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: x as f32,
        y: y as f32,
        ..LayoutSettings::default()
    });
    let font = &rc.theme.body_font;
    layout.append(&[font], &TextStyle::new(content, px as f32, 0));
    for glyph in layout.glyphs() {
        let (metrics, coverage) = font.rasterize_indexed(glyph.key.glyph_index as usize, px as f32);
        info!("Metrics: {:?}", glyph);

        //
        #[cfg(debug_assertions)]
        // #[cfg(not(debug_assertions))]
        {
            let mut path = PathBuilder::new();
            path.rect(
                glyph.x,
                glyph.y,
                metrics.width as f32,
                metrics.height as f32,
            );
            rc.draw_target.stroke(
                &path.finish(),
                &Source::Solid(SolidSource::from_unpremultiplied_argb(100, 255, 20, 150)),
                &StrokeStyle::default(),
                &DrawOptions::default(),
            );
        }

        //
        let mut image_data = Vec::with_capacity(coverage.len()); // / 3);
        for cov in &coverage {
            //.chunks(3) {
            // let r = cov[0];
            // let g = cov[1];
            // let b = cov[2];
            // let a = 255;
            let pixel = rgb_to_u32(0_usize, 0_usize, 0_usize, *cov as usize);
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
            &raqote::DrawOptions::new()
            // {
            //     blend_mode: raqote::BlendMode::SrcOver,
            //     alpha: 1.0,
            //     antialias: raqote::AntialiasMode::Gray,
            // },
        );
    }
}

pub fn rgb_to_u32(red: usize, green: usize, blue: usize, alpha: usize) -> u32 {
    let r = red.clamp(0, 255);
    let g = green.clamp(0, 255);
    let b = blue.clamp(0, 255);
    let a = alpha.clamp(0, 255);
    ((a << 24) | (r << 16) | (g << 8) | b) as u32 //  original
                                                  // ((r << 16) | (g << 8) | (b) | a) as u32
                                                  // ((r << 24) | (g << 16) | (b << 8) | a) as u32

    // let mut rgb = red;
    // rgb = (rgb << 16) + green;
    // rgb = (rgb << 16) + blue;
    // rgb as u32

    // ((a << 24) + (r << 16) + (g << 8) + (b)) as u32
}

#[test]
fn test_measure_text() {
    let theme = Theme::default();
    let size = measure_string(&theme, "A", 20);
    assert_eq!(0, size.x);
    assert_eq!(5, size.y);
    assert_eq!(12, size.w);
    assert_eq!(15, size.h);

    let size = measure_string(&theme, "AA", 20);
    assert_eq!(0, size.x);
    assert_eq!(5, size.y);
    assert_eq!(24, size.w);
    assert_eq!(15, size.h);
}
