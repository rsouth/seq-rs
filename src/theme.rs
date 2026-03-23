use fontdue::{Font, FontSettings};

// == Theme ===============================================
#[derive(Debug, Clone)]
pub struct Theme {
    pub title_font: Font,
    pub body_font: Font,
    pub title_font_px: usize,
    pub partic_font_px: usize,
    pub message_font_px: usize,
    pub document_border_width: usize,
    pub partic_padding: usize,
    pub partic_h_gap: usize,
    /// Vertical distance between successive interaction arrows in pixels.
    pub interaction_row_height: usize,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title_font: Theme::load_font(include_bytes!("../assets/Roboto-Thin.ttf")),
            body_font: Theme::load_font(include_bytes!("../assets/Roboto-Thin.ttf")),
            title_font_px: 30,
            partic_font_px: 30,
            message_font_px: 16,
            document_border_width: 10,
            partic_padding: 5,
            partic_h_gap: 80,
            interaction_row_height: 50,
        }
    }
}

impl Theme {
    fn load_font(font_data: &[u8]) -> Font {
        let settings = FontSettings {
            collection_index: 0,
            scale: 18.0,
            load_substitutions: true,
        };
        Font::from_bytes(font_data, settings).unwrap()
    }
}
