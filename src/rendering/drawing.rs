use crate::rendering::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, GapBetweenInteractions, ParticipantHGap, ParticipantPadding,
};
use crate::rendering::render_context::RenderingContext;
use crate::rendering::shapes::rect_path;
use crate::rendering::text::measure_text;
use crate::v2::{Diagram, Draw, DrawResult, InteractionSet, Participant, ParticipantSet};
use raqote::{
    Color, DrawOptions, LineCap, LineJoin, PathBuilder, Point, SolidSource, Source, StrokeStyle,
};
use std::collections::HashMap;
use std::time::Instant;

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

    fn get_x(&self, name: &str) -> Option<&f32> {
        self.data.get(name)
    }

    fn put_w(&mut self, name: &str, w: f32) {
        self.data_w.insert(name.to_string(), w);
    }

    fn get_w(&self, name: &str) -> Option<&f32> {
        self.data_w.get(name)
    }
}

// == Participant =========================================
impl Draw for ParticipantSet {
    fn draw(&self, rc: &mut RenderingContext, _dm: &DrawingMetrics) -> DrawResult {
        // for text
        let font_draw_options = DrawOptions::default();
        let font_color = Source::Solid(SolidSource::from(Color::new(255, 0, 0, 0)));
        let participant_box_height = (ParticipantPadding.value() * 2.0)
            + measure_text(&rc.participant_font, rc.theme.participant_font_pt, "A")
                .unwrap()
                .height() as f32;

        // for both...
        let y_position: f32 = DiagramPadding.value() + DiagramMargin.value();
        let mut x_position: f32 = DiagramPadding.value() + DiagramMargin.value();

        // for rectangles + other drawing
        let mut rect_path = PathBuilder::new();
        let mut activation_box_path = PathBuilder::new();

        self.iter().for_each(|p: &Participant| {
            // == Participant Boxes ==
            let rect =
                measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.name).unwrap();
            rect_path.rect(
                x_position,
                y_position,
                rect.width() as f32 + (2_f32 * ParticipantPadding.value()),
                participant_box_height as f32,
            );
            // == NAMES ==
            // todo y = padding + margin + fontHeight...
            let point = Point::new(
                x_position + ParticipantPadding.value(),
                y_position
                    + (ParticipantPadding.value() + DiagramMargin.value() + DiagramPadding.value()),
            );
            rc.draw_target.draw_text(
                &rc.participant_font,
                rc.theme.participant_font_pt,
                &p.name,
                point,
                &font_color,
                &font_draw_options,
            );

            // == Lifeline ==
            rect_path.move_to(
                x_position + (rect.width() / 2) as f32 + ParticipantPadding.value(),
                y_position + participant_box_height as f32,
            );
            rect_path.line_to(
                x_position + (rect.width() / 2) as f32 + ParticipantPadding.value(),
                rc.diagram_height as f32 - y_position,
            );

            // == Activation Box
            // from first appearance to last?
            println!(
                "{} active from {} to {}",
                p.name, p.active_from, p.active_until
            );
            activation_box_path.rect(
                x_position + (rect.width() / 2) as f32 + 5.0,
                y_position
                    + ParticipantPadding.value()
                    + participant_box_height
                    + (p.active_from as f32 * GapBetweenInteractions.value()),
                10_f32,
                y_position
                    + ((p.active_until as f32 - p.active_from as f32) - 1.0)
                        * GapBetweenInteractions.value(),
            );

            // update X positions for next participant...
            x_position += rect.width() as f32
                + (ParticipantPadding.value() * 2_f32)
                + ParticipantHGap.value();
        });

        // draw
        let stroke = StrokeStyle::default();
        rc.draw_target.stroke(
            &rect_path.finish(),
            &font_color,
            &stroke,
            &DrawOptions::new(),
        );

        let sss = Source::Solid(SolidSource::from(Color::new(255, 20, 20, 20)));

        let path = activation_box_path.finish();
        rc.draw_target.fill(&path, &sss, &DrawOptions::new());
        rc.draw_target.stroke(
            &path,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(200, 255, 200, 200)),
            &StrokeStyle::default(),
            &DrawOptions::new(),
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

        self.iter().for_each(|interaction| {
            let participant_box_height = (ParticipantPadding.value() * 2.0)
                + measure_text(&rc.participant_font, rc.theme.participant_font_pt, "A")
                    .unwrap()
                    .height() as f32;
            let this_y = DiagramPadding.value()
                + DiagramMargin.value()
                + ParticipantPadding.value()
                + participant_box_height
                + interaction.count as f32 * GapBetweenInteractions.value();

            let from_x = *_dm
                .get_x(interaction.from_participant.name.as_str())
                .unwrap();
            let to_x = *_dm.get_x(interaction.to_participant.name.as_str()).unwrap();

            let left_to_right = from_x < to_x;
            let from_adj = if left_to_right { 15.0 } else { 5.0 };
            let to_adj = if left_to_right { 5.0 } else { 15.0 };
            rect_path.move_to(
                from_x
                    + *_dm
                        .get_w(interaction.from_participant.name.as_str())
                        .unwrap()
                        / 2_f32
                    + from_adj,
                this_y,
            );
            rect_path.line_to(
                to_x + *_dm.get_w(interaction.to_participant.name.as_str()).unwrap() / 2_f32
                    + to_adj,
                this_y,
            );

            h_line.move_to(0.0, this_y);
            h_line.line_to(rc.diagram_width as f32, this_y);

            // rect_path.move_to()

            println!(
                "drawing interaction from {} {} to {} {}",
                interaction.from_participant.name,
                interaction.from_participant.count,
                interaction.to_participant.name,
                interaction.to_participant.count
            );
        });

        rc.draw_target.stroke(
            &rect_path.finish(),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(200, 20, 255, 20)),
            &StrokeStyle::default(),
            &DrawOptions::new(),
        );

        let ss = StrokeStyle {
            width: 1.,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 10.,
            dash_array: vec![10_f32, 18_f32],
            dash_offset: 16.,
        };
        rc.draw_target.stroke(
            &h_line.finish(),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(200, 200, 200, 200)),
            &ss,
            &DrawOptions::new(),
        );

        Ok(())
    }
}
// == Diagram =============================================
impl Diagram {
    pub fn draw(&mut self) -> DrawResult {
        let options = DrawOptions::default();
        let src = Source::Solid(SolidSource::from(Color::new(200, 150, 30, 30)));
        let start = Instant::now();
        let rpath = rect_path(
            self.rendering_context.diagram_width,
            self.rendering_context.diagram_height,
        );
        self.rendering_context
            .draw_target
            .fill(&rpath, &src, &options);
        debug!(
            "Filled background rect in {}µs ({}ms)",
            start.elapsed().as_micros(),
            start.elapsed().as_millis()
        );

        let dm = self.precalc_x(&self.rendering_context);

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

    fn precalc_x(&self, rc: &RenderingContext) -> DrawingMetrics {
        let mut x_position: f32 = DiagramPadding.value() + DiagramMargin.value();
        let mut dm = DrawingMetrics::default();
        self.participants.iter().for_each(|p: &Participant| {
            let rect =
                measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.name).unwrap();
            dm.put_x(&p.name, x_position);
            dm.put_w(&p.name, rect.width() as f32);
            // update X positions for next participant...
            x_position += rect.width() as f32
                + (ParticipantPadding.value() * 2_f32)
                + ParticipantHGap.value();
        });

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
