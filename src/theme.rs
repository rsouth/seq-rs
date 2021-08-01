use core::default::Default;
use fontdue::Font;

// == Theme ===============================================
#[derive(Debug, Clone)]
pub struct Theme {
    pub title_font: Font,
    pub body_font: Font,
    pub title_font_px: f32,
    pub partic_font_px: f32,
    pub message_font_px: f32,
    pub document_border_width: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title_font: Theme::load_font(include_bytes!("../assets/Roboto-Thin.ttf") as &[u8]),
            body_font: Theme::load_font(include_bytes!("../assets/Roboto-Thin.ttf") as &[u8]),
            title_font_px: 30.0,
            partic_font_px: 18.0,
            message_font_px: 16.0,
            document_border_width: 10.0,
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
