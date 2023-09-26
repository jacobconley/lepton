use softbuffer::Buffer;

use crate::graphics::{
    typeface::{FontStyle, FontWeight},
    Text,
};

pub struct DrawHandle<'a, 'b> {
    pub buffer: &'a mut Buffer<'b>,
    pub width: usize,
}
impl<'a, 'b> DrawHandle<'a, 'b> {
    pub fn text(&mut self, text: Text) {
        let string = text.contents;
        let font = text
            .typeface
            .match_style(FontWeight::Normal.into(), FontStyle::Regular);

        let mut cursor = 50;
        let top: i32 = 50;

        let size = text.size;

        for char in string.chars() {
            let (metrics, bitmap) = font.rasterize(char, size as f32);

            let bottom = top + size;
            let baseline = bottom - metrics.bounds.ymin as i32;
            let char_top = baseline - metrics.bounds.height as i32;
            //` metrics.bounds.ymin` instead of `metrics.ymin`;
            // afaict, `metrics.ymin` is just `bounds.ymin` rounded up, which made it wonky
            // it's still wonky but it's a bit better

            (0..(bitmap.len())).for_each(|char_i| {
                let char_y = char_i / metrics.width;
                let char_x = char_i % metrics.width;

                let y = char_top as usize + char_y;

                let cval = ((255.0 * (bitmap[char_i] as f32 / 255.0)) as u8) as u32;
                let buf_i: usize = (y * self.width) + (char_x + cursor);
                self.buffer[buf_i] = cval | (cval << 8) | (cval << 16);
            });

            let advance = metrics.advance_width.round() as usize;
            // // Renders the baseline 👇
            // (0..advance).for_each(|line_x| {
            //     self.buffer[baseline as usize * self.width + (cursor + line_x)] = 0xff0000;
            // });

            cursor += advance;
        }
    }
}
