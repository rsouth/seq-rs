use itertools::Itertools;
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source, StrokeStyle};

use super::{diagram::Diagram, model::Participant, theme::Theme, ParticipantSet};
use crate::rendering::text::draw_text;
use ordered_float::OrderedFloat;

pub mod text;

pub trait RenderSet {
    fn render(&self, context: &mut RenderContext);
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Rect {
    pub x: OrderedFloat<f32>,
    pub _y: OrderedFloat<f32>,
    pub w: OrderedFloat<f32>,
    pub h: OrderedFloat<f32>,
}

impl Rect {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect {
            x: x.into(),
            _y: y.into(),
            w: w.into(),
            h: h.into(),
        }
    }
}

pub trait Render {
    fn render(&self, context: &mut RenderContext) -> Rect;
}

impl Diagram {
    pub fn render(&self) {
        let size = self.size(&self.theme);
        let mut rendering_context = RenderContext::new(size, self.theme.clone());

        self.participants.render(&mut rendering_context);

        rendering_context.draw_target.write_png("v3.png").unwrap();
        info!("Wrote file...");
    }
}

impl RenderSet for ParticipantSet {
    fn render(&self, context: &mut RenderContext) {
        self.iter().sorted_by_key(|k| k.index).for_each(|p| {
            p.render(context);
        })
    }
}

impl Render for Participant {
    fn render(&self, context: &mut RenderContext) -> Rect {
        let participant_padding = 5f32; // todo get from theme.

        let mut path = PathBuilder::new();
        path.rect(
            self.x.0 + participant_padding,
            self.y.0 + participant_padding,
            self.w.0 + (participant_padding * 2.0),
            self.h.0 + (participant_padding * 2.0),
        );

        let ss = StrokeStyle {
            width: 0.5,
            ..StrokeStyle::default()
        };

        context.draw_target.stroke(
            &path.finish(),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(225, 255, 20, 20)),
            &ss,
            &DrawOptions::default(),
        );

        draw_text(
            context,
            &self.name,
            self.x.0 + (2.0 * participant_padding),
            self.y.0 + participant_padding,
            context.theme.partic_font_px,
        );

        info!(
            "Drawing box for {} x: {}, y: {}, w: {}, h: {}",
            self.name, self.x.0, self.y.0, self.w.0, self.h.0
        );

        Rect::new(self.x.0, self.y.0, self.w.0, self.h.0)
    }
}

impl Sizable for Diagram {
    fn size(&self, _theme: &Theme) -> Size {
        let interaction_height = self.interactions.iter().map(|p| p.index).max();
        let height: i32 =
            (2 * _theme.document_border_width as i32) + (interaction_height.unwrap() as i32 * 20);

        let w = self.participants.iter().max_by_key(|p| p.x).unwrap();
        let width: i32 =
            (w.x.0 + w.w.0 + (2.0 * 5f32) + (2.0 * _theme.document_border_width)) as i32;

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

#[derive(Debug)]
pub struct Size {
    height: i32,
    width: i32,
}
