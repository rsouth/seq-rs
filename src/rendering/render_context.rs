use crate::rendering::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, GapBetweenInteractions, ParticipantHGap, ParticipantHeight,
    ParticipantPadding,
};
use crate::rendering::text::measure_all_participants;
use crate::v2::{Interaction, Participant};
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use raqote::DrawTarget;

use std::fmt::{Display, Formatter};
use std::time::Instant;

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
        let participant_font =
            RenderingContext::get_system_font(theme.participant_font_family.as_str());
        let diagram_height = RenderingContext::calculate_diagram_height(interactions);
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

    pub fn calculate_diagram_height(interactions: &[Interaction]) -> i32 {
        let interaction_count = interactions.len() as f32;
        let height = (DiagramPadding.value() * 2_f32)
            + (DiagramMargin.value() * 2_f32)
            + ParticipantHeight.value()
            + ParticipantPadding.value()
            + (interaction_count * 10.0)
            + (interaction_count * ParticipantPadding.value())
            + (interaction_count * GapBetweenInteractions.value());
        debug!("Calculated height {}", height);
        height as i32

        // //
        // y_position
        //     + ParticipantPadding.value()
        //     + participant_box_height
        //     + (p.active_from as f32 * GapBetweenInteractions.value()),
        // 10_f32,
        // p.active_until as f32 * RenderingConstants::GapBetweenInteractions.value(),
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

    #[allow(dead_code)]
    pub fn get_system_font(family_name: &str) -> Font {
        let start = Instant::now();
        let font = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title(family_name.to_string())],
                &Properties::new().weight(Weight::NORMAL),
            )
            .unwrap()
            .load()
            .unwrap(); // todo...
        info!(
            "Loaded font {} in {}ms",
            font.full_name(),
            start.elapsed().as_millis()
        );
        font
    }
}

// == Rendering Constants =================================
pub enum RenderingConstants {
    DiagramPadding,
    DiagramMargin,

    // Participant
    ParticipantHeight,
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
            RenderingConstants::ParticipantHeight => 50_f32,
            RenderingConstants::ParticipantHGap => 25_f32,
            RenderingConstants::ParticipantPadding => 10_f32,
            RenderingConstants::GapBetweenInteractions => 25_f32,
        }
    }
}

#[derive(Debug)]
pub struct Theme {
    pub _title_font_family: String,
    pub _title_font_pt: f32,

    pub participant_font_family: String,
    pub participant_font_pt: f32,
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

impl Default for Theme {
    fn default() -> Self {
        Theme {
            _title_font_family: "Arial".to_string(),
            _title_font_pt: 54.0,
            participant_font_family: "Arial".to_string(),
            participant_font_pt: 40.0,
        }
    }
}
