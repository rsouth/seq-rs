use core::default::Default;
use fontdue::Font;

// == Theme ===============================================
#[derive(Debug)]
pub struct Theme {
    title_font: Font,
    body_font: Font,
    title_font_px: f32,
    partic_font_px: f32,
    message_font_px: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            title_font: Theme::load_font(include_bytes!("../../assets/Roboto-Thin.ttf") as &[u8]),
            body_font: Theme::load_font(include_bytes!("../../assets/Roboto-Thin.ttf") as &[u8]),
            title_font_px: 30.0,
            partic_font_px: 18.0,
            message_font_px: 16.0,
        }
    }
}

impl Theme {
    fn load_font(font_data: &[u8]) -> Font {
        // Parse it into the font type.
        let settings = fontdue::FontSettings {
            ..fontdue::FontSettings::default()
        };

        fontdue::Font::from_bytes(font_data, settings).unwrap()
    }
}
