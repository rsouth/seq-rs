use super::{Rect, RenderContext};
use crate::v3::theme::Theme;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

#[cfg(debug_assertions)]
use raqote::{DrawOptions, PathBuilder, SolidSource, Source, StrokeStyle};

pub fn measure_string(theme: &Theme, content: &str, px: f32) -> Rect {
    debug_assert!(!content.is_empty());
    debug_assert!(px > 0_f32);
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.0,
        y: 0.0,
        ..LayoutSettings::default()
    });
    let font = &theme.body_font;
    layout.append(&[font], &TextStyle::new(&content, px, 0));

    let layout = layout.glyphs();
    let first_glyph = layout.first().unwrap();
    let last_glyph = layout.last().unwrap();
    Rect {
        x: first_glyph.x,
        _y: layout.iter().map(|glyph| glyph.y as i32).min().unwrap() as f32,
        w: (first_glyph.x + last_glyph.x + last_glyph.width as f32),
        h: layout.iter().map(|glyph| glyph.height).max().unwrap() as f32,
    }
}

pub fn draw_text(rc: &mut RenderContext, content: &str, x: f32, y: f32, px: f32) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x,
        y,
        ..LayoutSettings::default()
    });
    let font = &rc.theme.body_font;
    layout.append(&[font], &TextStyle::new(&content, px, 0));
    for glyph in layout.glyphs() {
        let (metrics, coverage) = font.rasterize(glyph.key.c, px);
        info!("Metrics: {:?}", glyph);

        //
        #[cfg(debug_assertions)]
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
                blend_mode: raqote::BlendMode::Darken,
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
    ((a << 24) | (r << 16) | (g << 8) | b) as u32
}

#[test]
fn test_measure_text() {
    let theme = Theme::default();
    let size = measure_string(&theme, "A", 20_f32);
    assert_eq!(0.0, size.x);
    assert_eq!(5.0, size._y);
    assert_eq!(12.0, size.w);
    assert_eq!(15.0, size.h);

    let size = measure_string(&theme, "AA", 20_f32);
    assert_eq!(0.0, size.x);
    assert_eq!(5.0, size._y);
    assert_eq!(24.0, size.w);
    assert_eq!(15.0, size.h);
}
