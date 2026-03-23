use itertools::Itertools;
use log::info;
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source, StrokeStyle};

use super::{
    diagram::Diagram,
    model::{Interaction, InteractionType, Participant},
    theme::Theme,
    ParticipantSet,
};
use crate::rendering::text::{draw_text, measure_string};

pub mod text;

/// Pixels between a message label and the arrow shaft beneath it.
const LABEL_ARROW_GAP: f32 = 2.0;
/// Extra horizontal gap between a self-reference loop and its message label.
const LOOP_LABEL_GAP: f32 = 5.0;
/// Length of an arrowhead along the shaft direction.
const ARROW_HEAD_LEN: f32 = 8.0;
/// Half-width of an arrowhead perpendicular to the shaft.
const ARROW_HEAD_HALF_W: f32 = 4.0;
/// Width of the self-reference loop (how far it extends to the right).
const SELF_REF_LOOP_W: f32 = 30.0;
/// Half the height of the self-reference loop.
const SELF_REF_LOOP_HALF_H: f32 = 12.0;

pub trait RenderSet {
    fn render(&self, context: &mut RenderContext);
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

pub trait Render {
    fn render(&self, context: &mut RenderContext);
}

/// Returns the horizontal center x of a participant's rendered box.
fn participant_cx(p: &Participant, padding: usize) -> f32 {
    let box_x = (p.rect.x + padding) as f32;
    let box_w = (p.rect.w + 2 * padding) as f32;
    box_x + box_w / 2.0
}

/// Returns the y coordinate of the bottom edge of a participant's rendered box.
/// `y_offset` is added to account for a title area above the participants.
fn participant_box_bottom(p: &Participant, padding: usize, y_offset: usize) -> f32 {
    // box_y  = rect.y + padding + y_offset
    // box_h  = rect.h + 2*padding
    // bottom = box_y + box_h
    (p.rect.y + p.rect.h + 3 * padding + y_offset) as f32
}

impl Diagram {
    /// Height (in pixels) reserved for the title above the participant boxes.
    fn title_area_height(&self) -> usize {
        if self.header.title.is_some() {
            self.theme.title_font_px + self.theme.document_border_width
        } else {
            0
        }
    }

    pub fn render(&self) {
        let size = self.size(&self.theme);
        let title_h = self.title_area_height();
        let mut rc = RenderContext::new(size, self.theme.clone());

        // 1. Render title (pinned to top, unaffected by content_y_offset).
        if let Some(title) = &self.header.title {
            let x = rc.theme.document_border_width;
            let y = rc.theme.document_border_width;
            let px = rc.theme.title_font_px;
            draw_text(&mut rc, title, x, y, px);
        }

        // From here on all content is shifted down by the title area.
        rc.content_y_offset = title_h;

        // 2. Render participant boxes.
        self.participants.render(&mut rc);

        // 3. Compute lifeline end y from theme values, independent of any single participant.
        // All participants share the same rect.y (= border) and rect.h (= max_partic_h),
        // so we derive the value directly from the theme rather than using an arbitrary entry.
        let border = rc.theme.document_border_width;
        let padding = rc.theme.partic_padding;
        let row_h = rc.theme.interaction_row_height;
        let n = self.interactions.len();
        let max_partic_h = self.participants.iter().map(|p| p.rect.h).max().unwrap_or(0);
        // lifeline starts at bottom of participant box:
        //   box_bottom = rect.y + rect.h + 3*padding + title_h
        //              = border  + max_partic_h + 3*padding + title_h
        let lifeline_start = (border + title_h + max_partic_h + 3 * padding) as f32;
        let lifeline_end = lifeline_start + ((n + 1) * row_h) as f32;

        for p in &self.participants {
            render_lifeline(p, &mut rc, lifeline_end);
        }

        // 4. Render interactions (sorted so index 0 is drawn first).
        let mut sorted: Vec<&Interaction> = self.interactions.iter().collect();
        sorted.sort_by_key(|i| i.index);
        for interaction in sorted {
            render_interaction(interaction, &mut rc);
        }

        rc.draw_target
            .write_png(&self.config.output_path)
            .unwrap();
        info!("Wrote file...");
    }
}

impl RenderSet for ParticipantSet {
    fn render(&self, context: &mut RenderContext) {
        self.iter().sorted_by_key(|k| k.index).for_each(|p| {
            p.render(context);
        });
    }
}

impl Render for Participant {
    fn render(&self, context: &mut RenderContext) {
        let padding = context.theme.partic_padding;
        let y_offset = context.content_y_offset;

        let mut path = PathBuilder::new();
        path.rect(
            (self.rect.x + padding) as f32,
            (self.rect.y + padding + y_offset) as f32,
            (self.rect.w + 2 * padding) as f32,
            (self.rect.h + 2 * padding) as f32,
        );

        let ss = StrokeStyle {
            width: 1.0,
            ..StrokeStyle::default()
        };

        context.draw_target.stroke(
            &path.finish(),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 50, 100, 200)),
            &ss,
            &DrawOptions::default(),
        );

        draw_text(
            context,
            &self.name,
            self.rect.x + 2 * padding,
            self.rect.y + padding + y_offset,
            context.theme.partic_font_px,
        );

        info!(
            "Drawing box for {} x: {}, y: {}, w: {}, h: {}",
            self.name, self.rect.x, self.rect.y, self.rect.w, self.rect.h
        );
    }
}

/// Draws a dashed vertical lifeline for a participant from the bottom of its
/// box down to `end_y`.
fn render_lifeline(p: &Participant, context: &mut RenderContext, end_y: f32) {
    let padding = context.theme.partic_padding;
    let y_offset = context.content_y_offset;
    let cx = participant_cx(p, padding);
    let top_y = participant_box_bottom(p, padding, y_offset);

    let mut path = PathBuilder::new();
    path.move_to(cx, top_y);
    path.line_to(cx, end_y);

    let ss = StrokeStyle {
        width: 1.0,
        dash_array: vec![5.0, 5.0],
        dash_offset: 0.0,
        ..StrokeStyle::default()
    };

    context.draw_target.stroke(
        &path.finish(),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 150, 150, 150)),
        &ss,
        &DrawOptions::default(),
    );
}

/// Dispatches rendering for a single interaction (arrow or self-reference loop).
fn render_interaction(interaction: &Interaction, context: &mut RenderContext) {
    let padding = context.theme.partic_padding;
    let y_offset = context.content_y_offset;
    let row_h = context.theme.interaction_row_height;

    let from_p = &interaction.from_participant;
    let to_p = &interaction.to_participant;

    // Y position of this arrow on the lifeline.
    let box_bottom = participant_box_bottom(from_p, padding, y_offset);
    let arrow_y = box_bottom + ((interaction.index as usize + 1) * row_h) as f32;
    let msg = interaction.message.as_ref().map(|m| m.0.as_str());

    match interaction.interaction_type {
        InteractionType::SelfRef => {
            let cx = participant_cx(from_p, padding);
            render_self_ref(context, cx, arrow_y, msg);
        }
        InteractionType::L2R | InteractionType::R2L => {
            let from_cx = participant_cx(from_p, padding);
            let to_cx = participant_cx(to_p, padding);
            render_arrow(context, from_cx, to_cx, arrow_y, msg);
        }
    }
}

/// Draws a horizontal arrow from `from_x` to `to_x` at height `y`, with an
/// optional message label centred above the shaft.
fn render_arrow(
    context: &mut RenderContext,
    from_x: f32,
    to_x: f32,
    y: f32,
    message: Option<&str>,
) {
    // Arrow shaft
    let mut path = PathBuilder::new();
    path.move_to(from_x, y);
    path.line_to(to_x, y);

    context.draw_target.stroke(
        &path.finish(),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 50, 50, 50)),
        &StrokeStyle {
            width: 1.5,
            ..StrokeStyle::default()
        },
        &DrawOptions::default(),
    );

    // Filled arrowhead triangle.
    let dir: f32 = if to_x >= from_x { 1.0 } else { -1.0 };

    let mut arrow_path = PathBuilder::new();
    arrow_path.move_to(to_x, y);
    arrow_path.line_to(to_x - dir * ARROW_HEAD_LEN, y - ARROW_HEAD_HALF_W);
    arrow_path.line_to(to_x - dir * ARROW_HEAD_LEN, y + ARROW_HEAD_HALF_W);
    arrow_path.close();

    context.draw_target.fill(
        &arrow_path.finish(),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 50, 50, 50)),
        &DrawOptions::default(),
    );

    // Message label centred above the shaft
    if let Some(msg) = message {
        if !msg.is_empty() {
            let msg_px = context.theme.message_font_px;
            let theme = context.theme.clone();
            let measured = measure_string(&theme, msg, msg_px);
            let mid_x = (from_x + to_x) / 2.0;
            let msg_x = (mid_x - measured.w as f32 / 2.0).max(0.0) as usize;
            let msg_y = (y - msg_px as f32 - LABEL_ARROW_GAP).max(0.0) as usize;
            draw_text(context, msg, msg_x, msg_y, msg_px);
        }
    }
}

/// Draws a small rectangular self-reference loop to the right of the participant,
/// with an optional message label to the right of the loop.
fn render_self_ref(context: &mut RenderContext, cx: f32, y: f32, message: Option<&str>) {
    // U-shaped open loop to the right.
    let mut path = PathBuilder::new();
    path.move_to(cx, y - SELF_REF_LOOP_HALF_H);
    path.line_to(cx + SELF_REF_LOOP_W, y - SELF_REF_LOOP_HALF_H);
    path.line_to(cx + SELF_REF_LOOP_W, y + SELF_REF_LOOP_HALF_H);
    path.line_to(cx, y + SELF_REF_LOOP_HALF_H);

    context.draw_target.stroke(
        &path.finish(),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 50, 50, 50)),
        &StrokeStyle {
            width: 1.5,
            ..StrokeStyle::default()
        },
        &DrawOptions::default(),
    );

    // Arrowhead pointing left at the return end, using the shared ARROW_HEAD_* constants.
    let mut arrow_path = PathBuilder::new();
    arrow_path.move_to(cx, y + SELF_REF_LOOP_HALF_H);
    arrow_path.line_to(cx + ARROW_HEAD_LEN, y + SELF_REF_LOOP_HALF_H - ARROW_HEAD_HALF_W);
    arrow_path.line_to(cx + ARROW_HEAD_LEN, y + SELF_REF_LOOP_HALF_H + ARROW_HEAD_HALF_W);
    arrow_path.close();

    context.draw_target.fill(
        &arrow_path.finish(),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 50, 50, 50)),
        &DrawOptions::default(),
    );

    // Message to the right of the loop.
    if let Some(msg) = message {
        if !msg.is_empty() {
            let msg_px = context.theme.message_font_px;
            let msg_x = (cx + SELF_REF_LOOP_W + LOOP_LABEL_GAP) as usize;
            let msg_y = (y - msg_px as f32 / 2.0).max(0.0) as usize;
            draw_text(context, msg, msg_x, msg_y, msg_px);
        }
    }
}

impl Sizable for Diagram {
    fn size(&self, theme: &Theme) -> Size {
        if self.participants.is_empty() {
            return Size {
                width: (4 * theme.document_border_width) as i32,
                height: (4 * theme.document_border_width) as i32,
            };
        }

        let border = theme.document_border_width;
        let padding = theme.partic_padding;
        let title_h = self.title_area_height();

        // Tallest participant box (text height + top/bottom padding).
        let max_partic_box_h = self
            .participants
            .iter()
            .map(|p| p.rect.h + 2 * padding)
            .max()
            .unwrap_or(0);

        // Vertical space for the interaction arrows (at least one row even if empty).
        let n = self.interactions.len().max(1);
        let lifeline_h = (n + 1) * theme.interaction_row_height;

        let height = (2 * border + title_h + max_partic_box_h + lifeline_h) as i32;

        // Width: right edge of the rightmost participant box + right border.
        // Box spans [rect.x + padding, rect.x + padding + rect.w + 2*padding]
        // so its right edge = rect.x + rect.w + 3*padding.
        let max_p = self
            .participants
            .iter()
            .max_by_key(|p| p.rect.x)
            .unwrap();
        let width = (max_p.rect.x + max_p.rect.w + 3 * padding + border) as i32;

        Size { height, width }
    }
}

pub struct RenderContext {
    pub theme: Theme,
    pub draw_target: DrawTarget,
    /// Vertical offset applied to all diagram content below the title.
    pub content_y_offset: usize,
}

impl RenderContext {
    fn new(size: Size, theme: Theme) -> Self {
        let mut draw_target = DrawTarget::new(size.width, size.height);
        draw_target.clear(SolidSource::from_unpremultiplied_argb(255, 255, 255, 255));
        RenderContext {
            theme,
            draw_target,
            content_y_offset: 0,
        }
    }
}

pub trait Sizable {
    fn size(&self, theme: &Theme) -> Size;
}

#[derive(Debug)]
pub struct Size {
    pub height: i32,
    pub width: i32,
}
