use std::str::Chars;

use fontdue::Font;

use crate::render::Pixel;

use super::typeface::{FontStyle, FontWeight, Typeface};

/// Rich text, no idea what this will look like
pub struct RichString {
    contents: String,

    /// The "default" typeface of the text.
    ///
    /// Eventually we will add attribution for rich text,
    /// but the truncation ellipsis will be in the default typeface.
    typeface: &'static Typeface,

    /// This will probably be completely replaced by attribution
    size: Pixel,
}
impl RichString {
    pub fn new(contents: String, typeface: &'static Typeface) -> Self {
        Self {
            contents,
            typeface,
            size: 32,
        }
    }

    pub fn rich_iter(&self) -> RichIterator {
        RichIterator {
            chars: self.contents.chars(),
            string: self,
        }
    }
}

/// An abstract character in rich text.
/// (Right now it's just a single `char`, but this will become a glyph cluster when we implement that)
pub struct RichChar {
    /// This will be replaced by a cluster when we implement segmentation
    pub(crate) char: char,
    pub(crate) font: &'static Font,
    pub size: Pixel,
}

// We use the iterator to deal with rich text attributes, style changes and stuff
pub struct RichIterator<'a> {
    string: &'a RichString,
    chars: Chars<'a>,
}
impl<'a> RichIterator<'a> {
    pub fn next(&mut self) -> Option<RichChar> {
        self.chars.next().map(|char| RichChar {
            char,
            font: self
                .string
                .typeface
                .match_style(FontWeight::Normal.into(), FontStyle::Regular),
            size: self.string.size,
        })
    }
}

/// When using [`Wrapping::Word`], this value is the minimum proportion of the line length that can be left by word wrapping.
/// That is, if the resulting line is less than this value times the maximum length,
/// we will fall back to character wrapping.
///
/// This also handles the strange case where a single word is several lines long,
/// which otherwise would break the whole wrapping algorithm
pub const WORD_WRAP_LENGTH_THRESHOLD: f32 = 0.5;

pub enum Wrapping {
    /// Only a single line.  Does not raise an error if truncation happens.
    /// TODO: Rename this stupid thing
    SingleLineTruncate,

    /// Only a single line.  Raises an error if truncation happens
    SingleLine,

    /// Currently, somewhat naïve word boundary-based wrapping;
    /// **unless** [`WORD_WRAP_LENGTH_THRESHOLD`] is reached, then it breaks by character for that line
    ///
    /// This is very eurocentric; we need to explore internationalization.
    Word,

    /// Line breaks occur between _grapheme clusters_
    Character,
}
pub enum Direction {
    Horizontal,
    // TODO: Vertical text
    // We'll want to make some sort of abstraction that handles Horizontal vs Vertical advance lengths,
    // and maybe make a "main axis" vs "cross axis" size abstraction
    // that can be converted to/from the normal Size struct
}

pub struct TextOptions {
    pub wrapping: Wrapping,
    pub direction: Direction,
}
impl Default for TextOptions {
    fn default() -> Self {
        Self {
            wrapping: Wrapping::Word,
            direction: Direction::Horizontal,
        }
    }
}
