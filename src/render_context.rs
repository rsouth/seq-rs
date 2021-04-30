use crate::render_context::RenderingConstants::{
    DiagramMargin, DiagramPadding, GapBetweenInteractions, ParticipantHGap, ParticipantHeight,
};
use crate::rendering::Theme;
use crate::text::measure_strings;
use crate::Diagram;
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use raqote::DrawTarget;
use smallvec::alloc::sync::Arc;
use std::time::Instant;

// == Rendering Context ===================================
pub struct RenderingContext {
    pub(crate) diagram_width: i32,
    pub(crate) diagram_height: i32,
    pub(crate) theme: Theme,
    pub(crate) draw_target: DrawTarget,
    //
    _title_font: Font,
    pub(crate) participant_font: Font,
}

impl RenderingContext {
    pub fn new(diagram: &Diagram, theme: Theme) -> Self {
        // load fonts
        let _title_font = RenderingContext::get_system_font(theme.title_font_family.as_str());
        let participant_font =
            RenderingContext::get_system_font(theme.participant_font_family.as_str());

        // calculate diagram size
        let diagram_height = RenderingContext::calculate_diagram_height(diagram);
        let diagram_width = RenderingContext::calculate_diagram_width(
            diagram,
            &participant_font,
            theme.participant_font_pt,
        );

        // create canvas
        let draw_target = DrawTarget::new(diagram_width, diagram_height);
        debug!(
            "Created draw target with {}x{}",
            diagram_width, diagram_height
        );
        RenderingContext {
            diagram_width,
            diagram_height,
            theme,
            draw_target,
            _title_font,
            participant_font,
        }
    }

    pub fn calculate_diagram_height(diagram: &Diagram) -> i32 {
        let interaction_count = diagram.interaction_set.len() as i32;
        let height = (DiagramPadding.value() * 2)
            + (DiagramMargin.value() * 2)
            + ParticipantHeight.value()
            + (interaction_count * GapBetweenInteractions.value());
        debug!("Calculated height {}", height);
        height
    }

    // todo doesn't calculate the width added by interaction messages etc
    pub fn calculate_diagram_width(
        diagram: &Diagram,
        font: &Font,
        participant_font_size: f32,
    ) -> i32 {
        let partic_width = measure_strings(font, participant_font_size, &diagram.interaction_set);
        let width = (DiagramPadding.value() * 2)
            + (DiagramMargin.value() * 2)
            + partic_width
            + (diagram.unique_participants * ParticipantHGap.value());
        debug!("Calculated width {}", width);
        width
    }

    #[allow(dead_code)]
    pub fn get_font() -> Font {
        let start = Instant::now();
        let font_data: &[u8] = include_bytes!("../assets/Roboto-Black.ttf");
        let font = Font::from_bytes(Arc::new(font_data.to_vec()), 0).unwrap();
        info!(
            "Loaded font {} in {}ms",
            font.full_name(),
            start.elapsed().as_millis()
        );
        font
    }

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

    // Interactions
    GapBetweenInteractions,
}

impl RenderingConstants {
    pub(crate) fn value(&self) -> i32 {
        match *self {
            RenderingConstants::DiagramPadding => 25,
            RenderingConstants::DiagramMargin => 15,
            RenderingConstants::ParticipantHeight => 50,
            RenderingConstants::ParticipantHGap => 20,
            RenderingConstants::GapBetweenInteractions => 50,
        }
    }
}
