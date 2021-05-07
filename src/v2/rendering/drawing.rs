use std::collections::HashMap;
use std::time::Instant;

use raqote::{DrawOptions, PathBuilder, Point, Source, StrokeStyle};

use crate::v2::rendering::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, GapBetweenInteractions, ParticipantHGap, ParticipantPadding,
};
use crate::v2::rendering::render_context::RenderingContext;
use crate::v2::rendering::shapes::rect_path;
use crate::v2::rendering::text::{draw_text, measure_text};
use crate::v2::{Diagram, Draw, DrawResult, InteractionSet, Participant, ParticipantSet};

// == Drawing Metrics =====================================
pub struct DrawingMetrics {
    data: HashMap<String, f32>,
    data_w: HashMap<String, f32>,
}

impl Default for DrawingMetrics {
    fn default() -> Self {
        DrawingMetrics {
            data: HashMap::new(),
            data_w: HashMap::new(),
        }
    }
}

impl DrawingMetrics {
    fn put_x(&mut self, name: &str, pos: f32) {
        self.data.insert(name.to_string(), pos);
    }

    fn get_x(&self, name: &str) -> f32 {
        *self.data.get(name).unwrap()
    }

    fn put_w(&mut self, name: &str, w: f32) {
        self.data_w.insert(name.to_string(), w);
    }

    fn get_w(&self, name: &str) -> f32 {
        *self.data_w.get(name).unwrap()
    }
}

// == Participant =========================================
impl Draw for ParticipantSet {
    fn draw(&self, rc: &mut RenderingContext, dm: &DrawingMetrics) -> DrawResult {
        let start_partic_set = Instant::now();
        // for text
        let font_color = Source::Solid(rc.theme.solid_black_source);
        let participant_box_height = (ParticipantPadding.value() * 2.0)
            + measure_text(&rc.participant_font, rc.theme.participant_font_pt, "A").height as f32;

        // for both...
        let y_position: f32 = DiagramPadding.value() + DiagramMargin.value();
        let mut x_position: f32 = DiagramPadding.value() + DiagramMargin.value();

        // for rectangles + other drawing
        let mut rect_path = PathBuilder::new();
        let mut activation_box_path = PathBuilder::new();

        trace!(
            "Setup Draw for ParticipantSet in {}us",
            start_partic_set.elapsed().as_micros()
        );
        let start_loop = Instant::now();

        self.iter().for_each(|p: &Participant| {
            let start = Instant::now();
            let partic_string_width = dm.get_w(&p.name);

            // == Participant Boxes ==
            rect_path.rect(
                x_position,
                y_position,
                partic_string_width + (2_f32 * ParticipantPadding.value()),
                participant_box_height,
            );
            trace!(
                "Created {} participant rect in {}us",
                &p.name,
                start.elapsed().as_micros()
            );

            // == NAMES ==
            let start = Instant::now();
            let point = Point::new(
                x_position /*+ ParticipantPadding.value()*/,
                y_position
                    // + ParticipantPadding.value()
                    // + DiagramMargin.value()
                    // + DiagramPadding.value(),
            );
            // rc.draw_target.draw_text(
            //     &rc.participant_font,
            //     rc.theme.participant_font_pt,
            //     &p.name,
            //     point,
            //     &font_color,
            //     &rc.theme.default_draw_options,
            // );
            draw_text(rc, &p.name, point.x, point.y, rc.theme.participant_font_pt);
            trace!(
                "Drew {} text in {}us ({}ms)",
                &p.name,
                start.elapsed().as_micros(),
                start.elapsed().as_millis()
            );

            // == Lifeline ==
            let start = Instant::now();
            rect_path.move_to(
                x_position + (partic_string_width / 2.0) as f32 + ParticipantPadding.value(),
                y_position + participant_box_height as f32,
            );
            rect_path.line_to(
                x_position + (partic_string_width / 2.0) as f32 + ParticipantPadding.value(),
                rc.diagram_height as f32 - y_position,
            );
            trace!(
                "Created lifeline for {} in {}us",
                &p.name,
                start.elapsed().as_micros()
            );

            // == Activation Box
            let start = Instant::now();
            activation_box_path.rect(
                x_position + (partic_string_width / 2.0) + 5.0,
                y_position
                    + ParticipantPadding.value()
                    + participant_box_height
                    + (p.active_from as f32 * GapBetweenInteractions.value()),
                10_f32,
                y_position
                    + ((p.active_until as f32 - p.active_from as f32) - 1.0)
                        * GapBetweenInteractions.value(),
            );
            debug!(
                "Activation Box for {} active from {} to {}",
                &p.name, p.active_from, p.active_until
            );
            trace!(
                "Created activation box for {} in {}us",
                &p.name,
                start.elapsed().as_micros()
            );

            // update X positions for next participant...
            x_position += partic_string_width
                + (ParticipantPadding.value() * 2_f32)
                + ParticipantHGap.value();
        });

        trace!(
            "Completed Draw for ParticipantSet loop in {}us ({}ms)",
            start_loop.elapsed().as_micros(),
            start_loop.elapsed().as_millis()
        );

        // draw
        rc.draw_target.stroke(
            &rect_path.finish(),
            &font_color,
            &rc.theme.default_stroke_style,
            &DrawOptions::new(),
        );

        let path = activation_box_path.finish();
        rc.draw_target.fill(
            &path,
            &Source::Solid(rc.theme.solid_lgrey_source),
            &DrawOptions::new(),
        );
        rc.draw_target.stroke(
            &path,
            &Source::Solid(rc.theme.solid_dgrey_source),
            &rc.theme.default_stroke_style,
            &rc.theme.default_draw_options,
        );

        trace!(
            "Completed Draw for ParticipantSet in {}us ({}ms)",
            start_partic_set.elapsed().as_micros(),
            start_partic_set.elapsed().as_millis()
        );

        Ok(())
    }
}

impl Draw for Participant {
    fn draw(&self, _rc: &mut RenderingContext, _dm: &DrawingMetrics) -> DrawResult {
        todo!()
    }
}

impl Draw for InteractionSet {
    fn draw(&self, rc: &mut RenderingContext, _dm: &DrawingMetrics) -> DrawResult {
        let mut rect_path = PathBuilder::new();
        let mut h_line = PathBuilder::new();

        let participant_box_height = (ParticipantPadding.value() * 2.0)
            + measure_text(&rc.participant_font, rc.theme.participant_font_pt, "A").height as f32;
        let initial_y = DiagramPadding.value()
            + DiagramMargin.value()
            + ParticipantPadding.value()
            + participant_box_height;

        self.iter().for_each(|interaction| {
            let this_y = initial_y + interaction.count as f32 * GapBetweenInteractions.value();

            let from_x = _dm.get_x(interaction.from_participant.name.as_str());
            let to_x = _dm.get_x(interaction.to_participant.name.as_str());

            let left_to_right = from_x < to_x;
            let from_adj = if left_to_right { 15.0 } else { 5.0 };
            let to_adj = if left_to_right { 5.0 } else { 15.0 };
            rect_path.move_to(
                from_x + _dm.get_w(interaction.from_participant.name.as_str()) / 2_f32 + from_adj,
                this_y,
            );
            rect_path.line_to(
                to_x + _dm.get_w(interaction.to_participant.name.as_str()) / 2_f32 + to_adj,
                this_y,
            );

            h_line.move_to(0.0, this_y);
            h_line.line_to(rc.diagram_width as f32, this_y);

            debug!(
                "drawing interaction from {} {} to {} {}",
                interaction.from_participant.name,
                interaction.from_participant.count,
                interaction.to_participant.name,
                interaction.to_participant.count
            );
        });

        rc.draw_target.stroke(
            &rect_path.finish(),
            &Source::Solid(rc.theme.solid_bg_red),
            &StrokeStyle::default(),
            &DrawOptions::new(),
        );

        rc.draw_target.stroke(
            &h_line.finish(),
            &Source::Solid(rc.theme.solid_dark_source),
            &rc.theme.dashed_stroke_style,
            &DrawOptions::new(),
        );

        Ok(())
    }
}

// == Diagram =============================================
impl Diagram {
    pub fn draw(&mut self) -> DrawResult {
        // Background rectangle
        let rpath = rect_path(
            self.rendering_context.diagram_width,
            self.rendering_context.diagram_height,
        );

        //
        let start = Instant::now();
        self.rendering_context.draw_target.fill(
            &rpath,
            &Source::Solid(self.rendering_context.theme.solid_red_source),
            &self.rendering_context.theme.default_draw_options,
        );
        trace!(
            "Drew background rect in {}µs ({}ms)",
            start.elapsed().as_micros(),
            start.elapsed().as_millis()
        );

        //
        let dm = self.precalculate_x(&self.rendering_context);

        let start = Instant::now();
        self.participants
            .draw(&mut self.rendering_context, &dm)
            .unwrap();
        debug!("Drew participants in {}µs", start.elapsed().as_micros());

        let start = Instant::now();
        self.interactions
            .draw(&mut self.rendering_context, &dm)
            .unwrap();
        debug!("Drew interactions in {}µs", start.elapsed().as_micros());

        Ok(())
    }

    fn precalculate_x(&self, rc: &RenderingContext) -> DrawingMetrics {
        let start = Instant::now();
        let mut x_position: f32 = DiagramPadding.value() + DiagramMargin.value();
        let mut dm = DrawingMetrics::default();
        self.participants.iter().for_each(|p: &Participant| {
            let rect = measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.name);
            dm.put_x(&p.name, x_position); // + (ParticipantPadding.value() * 2_f32));
            dm.put_w(&p.name, rect.width as f32);
            // update X positions for next participant...
            x_position +=
                rect.width as f32 + (ParticipantPadding.value() * 2_f32) + ParticipantHGap.value();
        });
        trace!(
            "Pre-calc x values in {}µs ({}ms)",
            start.elapsed().as_micros(),
            start.elapsed().as_millis()
        );

        dm
    }

    pub fn save_png(&self, file_name: &str) {
        let start = Instant::now();
        self.rendering_context
            .draw_target
            .write_png(file_name)
            .unwrap();
        debug!(
            "Wrote to PNG in {}µs ({}ms)",
            start.elapsed().as_micros(),
            start.elapsed().as_millis()
        );
    }
}
