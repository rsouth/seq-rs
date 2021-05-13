use itertools::Itertools;
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source, StrokeStyle};

use crate::v3::rendering::text::{draw_text, measure_text_v3000};

use super::{diagram::Diagram, model::Participant, theme::Theme, ParticipantSet};

mod text;

pub trait RenderSet {
    fn render(&self, context: &mut RenderContext);
}

pub struct Rect {
    x: f32,
    _y: f32,
    w: f32,
    _h: f32,
}

impl Rect {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, _y: y, w, _h: h }
    }
}

pub trait Render {
    fn render(&self, context: &mut RenderContext, x: f32, y: f32) -> Rect;
}

impl Diagram {
    pub fn render(&self, theme: Theme) {
        let size = self.size(&theme);
        let mut rendering_context = RenderContext::new(size, theme);

        self.participants.render(&mut rendering_context);

        rendering_context.draw_target.write_png("v3.png").unwrap();
    }
}

impl RenderSet for ParticipantSet {
    fn render(&self, context: &mut RenderContext) {
        let mut current_x: f32 = 10.0;

        self.iter().sorted_by_key(|k| k.index).for_each(|p| {
            let pos = p.render(context, current_x, 10.0);

            current_x = pos.x + pos.w + 10.0;
        })
    }
}

impl Render for Participant {
    fn render(&self, context: &mut RenderContext, x: f32, y: f32) -> Rect {
        draw_text(context, &self.name, x, y, context.theme.partic_font_px);

        let mut path = PathBuilder::new();
        let boundary = measure_text_v3000(&context.theme, &self.name, context.theme.partic_font_px);
        path.rect(x, y, boundary.width as f32, boundary.height as f32);
        context.draw_target.stroke(
            &path.finish(),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(225, 255, 20, 20)),
            &StrokeStyle::default(),
            &DrawOptions::default(),
        );

        info!(
            "Drawing box for {} x: {}, y: {}, w: {}, h: {}",
            self.name, x, y, boundary.width, boundary.height
        );

        Rect::new(x, y, boundary.width as f32, boundary.height as f32)
    }
}

impl Sizable for Diagram {
    fn size(&self, _theme: &Theme) -> Size {
        let interaction_height = self.interactions.iter().map(|p| p.index).max();
        let height: i32 = 10 + (interaction_height.unwrap() as i32 * 20) + 10;

        let w: Size = self
            .participants
            .iter()
            .map(|p| measure_text_v3000(_theme, &p.name, _theme.partic_font_px))
            .reduce(|a, x| Size {
                width: a.width + x.width,
                height: a.height + x.height,
            })
            .unwrap();
        let width = 10 + w.width + ((self.participants.len() as i32 - 1) * 10) + 10;

        Size { height, width }
    }
}

pub struct RenderContext {
    theme: Theme,
    draw_target: DrawTarget,
}

impl RenderContext {
    fn new(size: Size, theme: Theme) -> Self {
        let mut draw_target = DrawTarget::new(size.width, size.height);
        draw_target.clear(SolidSource::from_unpremultiplied_argb(255, 255, 255, 255));
        RenderContext { theme, draw_target }
    }
}

pub trait Sizable {
    fn size(&self, theme: &Theme) -> Size;
}

pub struct SizeBuilder {
    last_x: i32,
    width: i32,
    height: i32,
}

pub struct Size {
    height: i32,
    width: i32,
}
