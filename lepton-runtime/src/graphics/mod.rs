pub mod typeface;

use typeface::Typeface;

/// Rich text, no idea what this will look like
pub struct Text {
    pub contents: String,
    pub typeface: &'static Typeface,
    pub size: i32,
}
impl Text {
    pub fn new(contents: String, typeface: &'static Typeface) -> Self {
        Self {
            contents,
            typeface,
            size: 32,
        }
    }
}
