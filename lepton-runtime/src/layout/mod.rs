use crate::render::Pixel;

#[derive(Clone, Copy)]
pub struct Size {
    pub width: Pixel,
    pub height: Pixel,
}

/// Position from the top left of the screen
pub struct Position {
    pub x: Pixel,
    pub y: Pixel,
}

#[derive(Clone, Copy)]
pub struct SizeConstraint {
    pub width: Option<Pixel>,
    pub height: Option<Pixel>,
}
impl SizeConstraint {
    pub fn auto() -> Self {
        Self {
            height: None,
            width: None,
        }
    }
    pub fn intrinsic_width(width: Pixel) -> Self {
        Self {
            width: Some(width),
            height: None,
        }
    }
    pub fn intrinsic_height(height: Pixel) -> Self {
        Self {
            height: Some(height),
            width: None,
        }
    }

    pub fn fits(&self, inner: Size) -> bool {
        self.width.map(|cw| inner.width <= cw).unwrap_or(true)
            && self.height.map(|ch| inner.height <= ch).unwrap_or(true)
    }
    pub fn fits_width(&self, width: Pixel) -> bool {
        self.width.map(|cw| width <= cw).unwrap_or(true)
    }
    pub fn fits_height(&self, height: Pixel) -> bool {
        self.height.map(|ch| height <= ch).unwrap_or(true)
    }
}
