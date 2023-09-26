use std::fmt::Display;

use eyre::WrapErr;
use fontdue::{Font as FontData, FontSettings};

pub type Weight = u16;

/**
 * https://learn.microsoft.com/en-us/typography/opentype/spec/os2#usweightclass
 */
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}
impl From<FontWeight> for Weight {
    fn from(value: FontWeight) -> Self {
        value as Self
    }
}

/// Analogous to the CSS `font-style` property
pub enum FontStyle {
    Regular = 0,
    Italic = 1,
}

pub struct Variant {
    pub weight: Weight,
    pub style: FontStyle,
    pub data: FontData,
}

/// Simple wrapper for the `&'static str` returned by `fontdue`;
/// we need something that implements `Error` for `eyre`
#[derive(Debug)]
pub struct FontError(&'static str);
impl Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FontError: {}", self.0)
    }
}
impl std::error::Error for FontError {}

const FONT_DATA: &[u8] = include_bytes!("../../../assets/Roboto/Roboto-Regular.ttf");

pub struct Typeface {
    /// For now, constructor guarantees non-empty, first one is default
    variants: Vec<Variant>,
}
impl Typeface {
    pub fn try_default() -> eyre::Result<Self> {
        let data = FontData::from_bytes(FONT_DATA, FontSettings::default())
            .map_err(FontError)
            .wrap_err("processing default font")?;

        Ok(Self {
            variants: vec![Variant {
                weight: FontWeight::Normal.into(),
                style: FontStyle::Regular,
                data,
            }],
        })
    }

    pub fn match_style(&self, _weight: Weight, _style: FontStyle) -> &FontData {
        //TODO: Implement this!
        &self.variants.first().unwrap().data
    }
}
