use super::{Rect, RenderContext};
use crate::theme::Theme;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

#[cfg(debug_assertions)]
use raqote::{DrawOptions, PathBuilder, SolidSource, Source, StrokeStyle};

/// Measures the bounding box of the rendered string at the given font size.
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

    let glyphs = layout.glyphs();
    let first_glyph = glyphs.first().unwrap();
    let last_glyph = glyphs.last().unwrap();
    let y = glyphs.iter().map(|g| g.y as usize).min().unwrap();
    let h = glyphs.iter().map(|g| g.height).max().unwrap();

    Rect {
        x: first_glyph.x as usize,
        y,
        w: (first_glyph.x as usize + last_glyph.x as usize + last_glyph.width),
        h,
    }
}

/// Draws text into the render context at the given position and font size.
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
        let (metrics, coverage) = font.rasterize(glyph.parent, px as f32);
        log::info!("Metrics: {:?}", glyph);

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

        let image_data: Vec<u32> = coverage
            .iter()
            .map(|&cov| rgb_to_u32(0, 0, 0, cov as usize))
            .collect();

        rc.draw_target.draw_image_at(
            glyph.x,
            glyph.y,
            &raqote::Image {
                width: metrics.width as i32,
                height: metrics.height as i32,
                data: &image_data,
            },
            &raqote::DrawOptions::new(),
        );
    }
}

/// Packs RGBA components into a `u32` pixel value.
pub fn rgb_to_u32(red: usize, green: usize, blue: usize, alpha: usize) -> u32 {
    let r = red.clamp(0, 255);
    let g = green.clamp(0, 255);
    let b = blue.clamp(0, 255);
    let a = alpha.clamp(0, 255);
    ((a << 24) | (r << 16) | (g << 8) | b) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn test_rgb_to_u32_black() {
        assert_eq!(0xFF000000, rgb_to_u32(0, 0, 0, 255));
    }

    #[test]
    fn test_rgb_to_u32_white() {
        assert_eq!(0xFFFFFFFF, rgb_to_u32(255, 255, 255, 255));
    }

    #[test]
    fn test_rgb_to_u32_blue() {
        assert_eq!(0xFF0000FF, rgb_to_u32(0, 0, 255, 255));
    }

    #[test]
    fn test_rgb_to_u32_transparent() {
        assert_eq!(0x00000000, rgb_to_u32(0, 0, 0, 0));
    }

    #[test]
    fn test_rgb_to_u32_clamps_above_255() {
        // Values > 255 are clamped to 255
        assert_eq!(rgb_to_u32(255, 255, 255, 255), rgb_to_u32(300, 400, 500, 600));
    }

    #[test]
    fn test_measure_string_single_char() {
        let theme = Theme::default();
        let size = measure_string(&theme, "A", 20);
        assert_eq!(0, size.x);
        assert!(size.y > 0, "y should be > 0 (baseline offset)");
        assert!(size.w > 0, "width should be > 0");
        assert!(size.h > 0, "height should be > 0");
    }

    #[test]
    fn test_measure_string_two_chars_wider() {
        let theme = Theme::default();
        let single = measure_string(&theme, "A", 20);
        let double = measure_string(&theme, "AA", 20);
        assert!(double.w > single.w, "two chars should be wider than one");
    }

    #[test]
    fn test_measure_string_same_height_for_same_font_size() {
        let theme = Theme::default();
        let a = measure_string(&theme, "A", 20);
        let b = measure_string(&theme, "B", 20);
        assert_eq!(a.h, b.h, "same font size should yield same glyph height");
    }

    #[test]
    fn test_measure_string_larger_px_gives_larger_height() {
        let theme = Theme::default();
        let small = measure_string(&theme, "A", 20);
        let large = measure_string(&theme, "A", 40);
        assert!(large.h > small.h, "larger px should produce taller glyphs");
    }
}
