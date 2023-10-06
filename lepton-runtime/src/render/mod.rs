/// Everything involving text rasterization lives here.
/// This module also handles intrinsic sizing, which takes care of a lot of layout stuff as well.
pub mod text;

use softbuffer::Buffer;

use crate::layout::Position;

/// Simple structure that encapsulates the frame buffer and relevant metadata.
/// Render methods are implemented to take this structure, to keep them separate from the event loop.
pub struct DrawHandle<'a, 'b> {
    pub buffer: &'a mut Buffer<'b>,
    pub width: usize,
}
impl<'a, 'b> DrawHandle<'a, 'b> {
    #[inline]
    fn index(&self, position: Position) -> usize {
        position.y * self.width + position.x
    }

    fn set(&mut self, position: Position, value: u32) {
        let index = self.index(position);
        self.buffer[index] = value;
    }
}

/// We're only dealing with integer pixels for now
pub type Pixel = usize;

pub trait Drawable {
    fn draw_at(&self, handle: &mut DrawHandle, position: Position);
}
