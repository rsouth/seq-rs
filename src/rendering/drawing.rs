use crate::rendering::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, ParticipantHGap, ParticipantHeight, ParticipantPadding,
};
use crate::rendering::render_context::RenderingContext;
use crate::rendering::shapes::rect_path;
use crate::rendering::text::measure_text;
use crate::v2::{Diagram, Draw, DrawResult, Participant, ParticipantSet};
use raqote::{Color, DrawOptions, PathBuilder, Point, SolidSource, Source, StrokeStyle};
use std::time::Instant;

// == Participant =========================================
impl Draw for ParticipantSet {
    fn draw(&self, rc: &mut RenderingContext) -> DrawResult {
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

        self.iter().for_each(|p: &Participant| {
            // == Participant Boxes ==
            let rect =
                measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.name).unwrap();
            rect_path.rect(
                x_position,
                y_position,
                rect.width() as f32 + (2_f32 * ParticipantPadding.value()),
                participant_box_height as f32, // ParticipantHeight.value() as f32,
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
            // todo

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

        Ok(())
    }
}

impl Draw for Participant {
    fn draw(&self, _rc: &mut RenderingContext) -> DrawResult {
        todo!()
    }
}

// impl ParticipantSet {}

// == Diagram =============================================
impl Draw for Diagram {
    fn draw(&self, _rc: &mut RenderingContext) -> DrawResult {
        let options = DrawOptions::default();
        let src = Source::Solid(SolidSource::from(Color::new(200, 150, 30, 30)));
        let start = Instant::now();
        let rpath = rect_path(_rc.diagram_width, _rc.diagram_height);
        _rc.draw_target.fill(&rpath, &src, &options);
        debug!(
            "Filled rect path in {}µs ({}ms)",
            start.elapsed().as_micros(),
            start.elapsed().as_millis()
        );

        let start = Instant::now();
        self.participants.draw(_rc).unwrap();
        debug!(
            "Drew participant names in {}µs",
            start.elapsed().as_micros()
        );

        // let start = Instant::now();
        // self.draw_participant_rectangles(_rc);
        // debug!(
        //     "Drew participant names in {}µs",
        //     start.elapsed().as_micros()
        // );

        let start = Instant::now();
        _rc.draw_target.write_png("example.png").unwrap();
        debug!(
            "Wrote to PNG in {}µs ({}ms)",
            start.elapsed().as_micros(),
            start.elapsed().as_millis()
        );
        Ok(())
    }
}

impl Diagram {
    // fn draw_participant_rectangles(&self, rc: &mut RenderingContext) {
    //     let mut rect_path = PathBuilder::new();
    //     let mut x_pos = DiagramPadding.value() + DiagramMargin.value();
    //     let y_pos = DiagramPadding.value() + DiagramMargin.value();
    //     self.interactions
    //         .iter()
    //         .map(|p| smallvec::SmallVec::from_buf([&p.from_participant, &p.to_participant]))
    //         .flatten()
    //         .unique()
    //         .for_each(|p: &Participant| {
    //             let rect =
    //                 measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.0).unwrap();
    //             rect_path.rect(
    //                 x_pos as f32,
    //                 y_pos as f32,
    //                 rect.width() as f32 + 20_f32,
    //                 ParticipantHeight.value() as f32, //rect.height() as f32 + 20 as f32,
    //             );
    //
    //             x_pos += rect.width() + 20 + ParticipantHGap.value();
    //         });
    //
    //     let stroke = StrokeStyle::default();
    //     let color = Source::Solid(SolidSource::from_unpremultiplied_argb(255, 100, 200, 100));
    //     rc.draw_target
    //         .stroke(&rect_path.finish(), &color, &stroke, &DrawOptions::new());
    // }

    // fn draw_participant_names(&self, rc: &mut RenderingContext) {
    //     let src = Source::Solid(SolidSource::from(Color::new(255, 0, 0, 0)));
    //     let draw_options = DrawOptions::default();
    //
    //     let font_height = measure_text(&rc.participant_font, rc.theme.participant_font_pt, "A");
    //     let y_position = DiagramPadding.value() + DiagramMargin.value();
    //     let mut current_pos_x: i32 = DiagramPadding.value() + DiagramMargin.value();
    //     self.interactions
    //         .iter()
    //         .map(|p| smallvec::SmallVec::from_buf([&p.from_participant, &p.to_participant]))
    //         .flatten()
    //         .unique()
    //         .for_each(|p: &Participant| {
    //             let partic_measure =
    //                 measure_text(&rc.participant_font, rc.theme.participant_font_pt, &p.0).unwrap();
    //             let point = Point::new(
    //                 (current_pos_x + ParticipantPadding.value()) as f32,
    //                 y_position as f32
    //                     + ParticipantPadding.value() as f32
    //                     + font_height.unwrap().height() as f32,
    //             ); // todo y = padding + margin + fontHeight...
    //             info!("drawing {} at {}", &p.0, current_pos_x);
    //
    //             rc.draw_target.draw_text(
    //                 &rc.participant_font,
    //                 rc.theme.participant_font_pt,
    //                 &p.0,
    //                 point,
    //                 &src,
    //                 &draw_options,
    //             );
    //
    //             current_pos_x += (ParticipantPadding.value() * 2)
    //                 + ParticipantHGap.value()
    //                 + partic_measure.width();
    //         });
    // }
}
