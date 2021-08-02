use core::default::Default;
use fontdue::Font;

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
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title_font: Theme::load_font(include_bytes!("../assets/Roboto-Thin.ttf") as &[u8]),
            body_font: Theme::load_font(include_bytes!("../assets/Roboto-Thin.ttf") as &[u8]),
            title_font_px: 30,
            partic_font_px: 30, // 18,
            message_font_px: 16,
            document_border_width: 10,
            partic_padding: 5,
            partic_h_gap: 20,
        }
    }
}

impl Theme {
    fn load_font(font_data: &[u8]) -> Font {
        // Parse it into the font type.
        let settings = fontdue::FontSettings {
            // ..fontdue::FontSettings::default()
            collection_index: 0,
            scale: 18.0,
        };

        fontdue::Font::from_bytes(font_data, settings).unwrap()
    }
}
