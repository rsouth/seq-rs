use crate::rendering::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, GapBetweenInteractions, ParticipantHGap, ParticipantPadding,
};
use crate::rendering::text::{measure_all_participants, measure_text};
use crate::v2::{Interaction, Participant};
use raqote::{Color, DrawOptions, DrawTarget, LineCap, LineJoin, SolidSource, StrokeStyle};

use fontdue::Font;
use std::fmt::{Display, Formatter};

// == Rendering Context ===================================
pub struct RenderingContext {
    pub diagram_width: i32,
    pub diagram_height: i32,
    pub theme: Theme,
    pub draw_target: DrawTarget,
    pub participant_font: Font,
}

impl RenderingContext {
    pub fn create(
        interactions: &[Interaction],
        participants: &[Participant],
        theme: Theme,
    ) -> Self {
        let participant_font = RenderingContext::get_font(theme.participant_font_family.as_str());
        let diagram_height = RenderingContext::calculate_diagram_height(
            interactions,
            &participant_font,
            theme.participant_font_pt,
        );
        let diagram_width = RenderingContext::calculate_diagram_width(
            interactions,
            participants,
            &participant_font,
            theme.participant_font_pt,
        );

        let draw_target = DrawTarget::new(diagram_width, diagram_height);

        RenderingContext {
            diagram_width,
            diagram_height,
            draw_target,
            theme,
            participant_font,
        }
    }

    pub fn calculate_diagram_height(
        interactions: &[Interaction],
        font: &Font,
        font_size: f32,
    ) -> i32 {
        let interaction_count = interactions.len() as f32;
        let height = (DiagramPadding.value() * 2_f32)
            + (DiagramMargin.value() * 2_f32)
            + ParticipantPadding.value()
            + measure_text(font, font_size, "A").height as f32
            + (interaction_count * GapBetweenInteractions.value());
        debug!("Calculated height {}", height);
        height as i32
    }

    // todo doesn't calculate the width added by interaction messages etc
    pub fn calculate_diagram_width(
        interactions: &[Interaction],
        participants: &[Participant],
        font: &Font,
        participant_font_size: f32,
    ) -> i32 {
        let partic_width = measure_all_participants(font, participant_font_size, interactions);
        let width = (DiagramPadding.value() * 2_f32)
            + (DiagramMargin.value() * 2_f32)
            + partic_width as f32
            + ((participants.len() - 1) as f32 * ParticipantHGap.value())
            + (participants.len() as f32 * (ParticipantPadding.value() * 2_f32));
        debug!("Calculated width {}", width);
        width as i32
    }

    #[allow(dead_code)]
    pub fn get_font(_file_name: &str) -> Font {
        // Read the font data.
        let font = include_bytes!("../../assets/Roboto-Thin.ttf") as &[u8];
        // Parse it into the font type.
        let settings = fontdue::FontSettings {
            ..fontdue::FontSettings::default()
        };

        let font = fontdue::Font::from_bytes(font, settings).unwrap();
        font
    }

    // #[allow(dead_code)]
    // pub fn get_font() -> Font {
    //     let start = Instant::now();
    //     let font_data: &[u8] = include_bytes!("../assets/Roboto-Black.ttf");
    //     let font = Font::from_bytes(Arc::new(font_data.to_vec()), 0).unwrap();
    //     info!(
    //         "Loaded font {} in {}ms",
    //         font.full_name(),
    //         start.elapsed().as_millis()
    //     );
    //     font
    // }

    // #[allow(dead_code)]
    // pub fn get_system_font(family_name: &str) -> Font {
    //     let start = Instant::now();
    //     let font = SystemSource::new()
    //         .select_best_match(
    //             &[FamilyName::Title(family_name.to_string())],
    //             &Properties::new().weight(Weight::NORMAL),
    //         )
    //         .unwrap()
    //         .load()
    //         .unwrap(); // todo...
    //     info!(
    //         "Loaded font {} in {}ms",
    //         font.full_name(),
    //         start.elapsed().as_millis()
    //     );
    //     font
    // }
}

// == Rendering Constants =================================
pub enum RenderingConstants {
    DiagramPadding,
    DiagramMargin,

    // Participant
    // ParticipantHeight,
    ParticipantHGap,
    ParticipantPadding,

    // Interactions
    GapBetweenInteractions,
}

impl RenderingConstants {
    pub(crate) fn value(&self) -> f32 {
        match *self {
            RenderingConstants::DiagramPadding => 10_f32,
            RenderingConstants::DiagramMargin => 15_f32,
            // RenderingConstants::ParticipantHeight => 50_f32,
            RenderingConstants::ParticipantHGap => 25_f32,
            RenderingConstants::ParticipantPadding => 10_f32,
            RenderingConstants::GapBetweenInteractions => 25_f32,
        }
    }
}

// == Theme ===============================================
#[derive(Debug)]
pub struct Theme {
    // title font
    pub _title_font_family: String,
    pub _title_font_pt: f32,

    // participant font
    pub participant_font_family: String,
    pub participant_font_pt: f32,

    // drawing options
    pub default_stroke_style: StrokeStyle,
    pub dashed_stroke_style: StrokeStyle,
    pub default_draw_options: DrawOptions,

    // colors
    pub solid_black_source: SolidSource,
    pub solid_dgrey_source: SolidSource,
    pub solid_lgrey_source: SolidSource,
    pub solid_red_source: SolidSource,
    pub solid_dark_source: SolidSource,
    pub solid_bg_red: SolidSource,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            _title_font_family: "Arial".to_string(),
            _title_font_pt: 54.0,
            participant_font_family: "Arial".to_string(),
            participant_font_pt: 40.0,
            default_stroke_style: StrokeStyle::default(),
            dashed_stroke_style: StrokeStyle {
                width: 1.,
                cap: LineCap::Butt,
                join: LineJoin::Miter,
                miter_limit: 10.,
                dash_array: vec![10_f32, 18_f32],
                dash_offset: 16.,
            },
            default_draw_options: DrawOptions::default(),
            solid_black_source: SolidSource::from(Color::new(255, 0, 0, 0)),
            solid_dgrey_source: SolidSource::from(Color::new(200, 255, 200, 200)),
            solid_lgrey_source: SolidSource::from(Color::new(255, 20, 20, 20)),
            solid_red_source: SolidSource::from(Color::new(200, 150, 30, 30)),
            solid_dark_source: SolidSource::from(Color::new(200, 200, 200, 200)),
            solid_bg_red: SolidSource::from(Color::new(200, 20, 255, 20)),
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Theme {{ title_font: {{family: {}, size: {}}}, participant_font: {{ family: {}, size: {} }} }} ",
            self._title_font_family, self._title_font_pt,
            self.participant_font_family, self.participant_font_pt
        )
    }
}
