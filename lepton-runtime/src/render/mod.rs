pub mod text;

use softbuffer::Buffer;

/// Simple structure that encapsulates the frame buffer and relevant metadata.
/// Render methods are implemented to take this structure, to keep them separate from the event loop.
pub struct DrawHandle<'a, 'b> {
    pub buffer: &'a mut Buffer<'b>,
    pub width: usize,
}
