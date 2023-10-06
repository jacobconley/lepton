use std::collections::VecDeque;

use super::{DrawHandle, Drawable, Pixel};
use crate::graphics::text::{
    Direction, RichChar, RichString, TextOptions, Wrapping, WORD_WRAP_LENGTH_THRESHOLD,
};
use crate::layout::{Position, Size, SizeConstraint};

use fontdue::Font;

#[derive(PartialEq, Eq)]
enum BreakKind {
    Character,

    /// Non-word character;
    /// **After** this unit, we can perform a line break by word
    WordBoundary,
}

/// A "piece" of text - either whitespace, or a rasterized text segment.
/// If it's a raster, `raster` is `Some`.
///
/// The `width` and `height` described here is the entire block size;
/// `Unit`s are assumed to be placed right next to each other.
struct Unit {
    width: Pixel,
    height: Pixel,
    raster: Option<Raster>,

    /// Information used during the line breaking process;
    /// describes what kind of break can happen **after** this unit
    break_kind: BreakKind,

    font: &'static Font,
}

/* TODO:
   non-breaking spaces in wrap algo
   0 width joiners?
   NEWLINES lmao
   change Raster to represent only bitmap; even spaces will be "rasterized" with `Unit::rasterize`


   //TODODOODODOOOD pushing characters when doesn't fit but it's also a word boundary
   // todo we are assuming we know the max width but what about like several labels side by side
   // also "label" is a good nomenclature for single-line truncate text

    I think I have the wrong idea about the baseline shit
*/
impl Unit {
    fn rasterize(rich_char: RichChar) -> Self {
        let raster = Raster::rasterize(rich_char.font, rich_char.size, rich_char.char);
        Self {
            width: raster.advance,
            height: rich_char.size,
            raster: Some(raster),
            break_kind: match rich_char.char.is_alphabetic() {
                true => BreakKind::Character,
                false => BreakKind::WordBoundary,
            },
            font: rich_char.font,
        }
    }

    fn is_whitespace(&self) -> bool {
        self.raster.is_none()
    }
}

/// Output of rasterization, for a given font and  size.
/// This currently only represents a single glyph, but eventually we will account for grapheme clusters.
///
/// All of the offsets contained here are relative to the top of the font line.
///
/// -----
///
/// Ultimately this stuff comes from `fontdue`,
/// but we're keeping some computed intermediate data here.
/// Particularly noteworthy is the conversions from subpixel floats to integer pixels.
///
/// I'm probably doing some of this wrong, so having the intermediate struct
/// will help when I inevitably have to fix things.  I have no idea what I'm doing.
/// Also, this might provide an opportunity to implement other layouts and future optimizations
#[derive(Clone)]
struct Raster {
    /// We should probably `Rc` this, or maybe implement proper caching
    bitmap: Vec<u8>,
    bitmap_width: usize,
    bitmap_height: usize,

    baseline: Pixel,
    bottom: Pixel,
    top: Pixel,

    advance: Pixel,
}
impl Raster {
    fn rasterize(font: &Font, font_size: Pixel, character: char) -> Self {
        let (metrics, bitmap) = font.rasterize(character, font_size as f32);

        let bottom = (font_size as i32 - metrics.bounds.ymin as i32) as Pixel;
        let top = bottom - metrics.bounds.height as Pixel;
        //` metrics.bounds.ymin` instead of `metrics.ymin`;
        // afaict, `metrics.ymin` is just `bounds.ymin` rounded up, which made it wonky
        // it's still wonky but it's a bit better

        //TODO: Account for xmin!!
        // https://freetype.org/freetype2/docs/glyphs/glyphs-3.html

        Self {
            bitmap,
            bitmap_width: metrics.width,
            bitmap_height: metrics.height,

            baseline: font_size,
            bottom,
            top,

            advance: metrics.advance_width.round() as Pixel,
        }
    }
}
impl Drawable for Raster {
    fn draw_at(&self, handle: &mut DrawHandle, position: Position) {
        (0..self.bitmap.len()).for_each(|bmp_i| {
            let bmp_x = bmp_i % self.bitmap_width;
            let bmp_y = bmp_i / self.bitmap_width;

            //TODO: Account for xmin, here?

            let val = (255.0 * (self.bitmap[bmp_i] as f32 / 255.0)) as u8 as u32;
            let val = val | (val << 8) | (val << 16);

            handle.set(
                Position {
                    x: position.x + bmp_x,
                    y: position.y + self.top + bmp_y,
                },
                val,
            );
        })
    }
}

// impl<'a, 'b> DrawHandle<'a, 'b> {
//     pub fn text(&mut self, text: RichString) {
//         let string = text.contents;
//         let font = text
//             .typeface
//             .match_style(FontWeight::Normal.into(), FontStyle::Regular);

//         let mut cursor = 50;
//         let top: i32 = 50;

//         let size = text.size;

//         for char in string.chars() {
//             let (metrics, bitmap) = font.rasterize(char, size as f32);

//             let baseline = top + size as i32;
//             let bottom = baseline - metrics.bounds.ymin as i32;
//             let char_top = bottom - metrics.bounds.height as i32;

//             (0..(bitmap.len())).for_each(|char_i| {
//                 let char_y = char_i / metrics.width;
//                 let char_x = char_i % metrics.width;

//                 let y = char_top as usize + char_y;

//                 let cval = ((255.0 * (bitmap[char_i] as f32 / 255.0)) as u8) as u32;
//                 let buf_i: usize = (y * self.width) + (char_x + cursor);
//                 self.buffer[buf_i] = cval | (cval << 8) | (cval << 16);
//             });

//             let advance = metrics.advance_width.round() as usize;
//             (0..advance).for_each(|line_x| {
//                 // Renders the baseline 👇
//                 self.buffer[top as usize * self.width + (cursor + line_x)] = 0xffffff;
//                 self.buffer[bottom as usize * self.width + (cursor + line_x)] = 0xffffff;
//                 self.buffer[baseline as usize * self.width + (cursor + line_x)] = 0x00ff00;
//                 self.buffer[char_top as usize * self.width + (cursor + line_x)] = 0x00ffff;
//             });

//             cursor += advance;
//         }
//     }
// }

/// Returned by text wrapping functions;
/// contains the resulting `TextLine` and the list of any `Unit`s chopped off by the wrapping operation.
type TextLineSplit = (TextLine, Vec<Unit>);

/// This iterates **backwards** from the end of the line.
/// Instead of calling the main function `next`, we're calling it `advance`, to avoid confusion.
/// The opposite function is `retreat`, which moves to the subsequent character.
/// Those functions return a `LineAt`; see below.
///
/// The `confirm` and `cancel` functions convert this object into our final result.
struct LineSplitIter {
    line: TextLine,
    position: usize,
    width_at_position: Pixel,
}

/// A `Unit` in the line, together with the width of the line
/// **up to and including** that `Unit`, disregarding everything after it.
type LineAt<'a> = (&'a Unit, Pixel);

impl LineSplitIter {
    fn new(line: TextLine) -> Self {
        Self {
            position: line.contents.len(),
            width_at_position: line.width,
            line,
        }
    }

    /// Moves the pointer to the **previous** character in the line.
    /// First call returns the last character in the line,
    /// `None` if we've already reached the beginning of the line.
    fn advance(&mut self) -> Option<LineAt> {
        if self.position == 0 {
            None
        } else {
            self.position -= 1;
            let unit = &self.line.contents[self.position];
            self.width_at_position -= unit.width;
            Some((unit, self.width_at_position))
        }
    }
    /// Moves the pointer to the **subsequent** character in the line.
    /// `None` if we've already reached the end of the line.
    fn retreat(&mut self) -> Option<LineAt> {
        if self.position >= self.line.contents.len() {
            None
        } else {
            self.position += 1;
            let unit = &self.line.contents[self.position];
            self.width_at_position += unit.width;
            Some((unit, self.width_at_position))
        }
    }

    /// Returns the line split by the current position of this iterator.
    fn confirm(mut self) -> TextLineSplit {
        let split: Vec<_> = self.line.contents.drain(self.position..).collect();
        (
            TextLine {
                contents: self.line.contents,
                width: self.width_at_position,
            },
            split,
        )
    }

    /// Ignores the current position of this iterator,
    /// returning the original line and an empty split fragment.
    fn cancel(self) -> TextLineSplit {
        (self.line, Vec::new())
    }
}

pub struct TextLine {
    contents: Vec<Unit>,
    width: Pixel,
}

impl TextLine {
    fn new() -> Self {
        TextLine {
            contents: Vec::new(),
            width: 0,
        }
    }

    fn height(&self) -> Pixel {
        self.contents.iter().fold(0, |res, val| res + val.height)
    }

    fn push(&mut self, unit: Unit) {
        self.width += unit.width;
        self.contents.push(unit)
    }

    /// `true` if the provided `unit` would fit at the end of the current line
    /// subject to the `max_size` constraint.
    #[inline]
    fn would_fit(&self, max_size: SizeConstraint, unit: &Unit) -> bool {
        max_size.fits_width(self.width + unit.width)
    }

    fn pop(&mut self) -> Option<Unit> {
        if let Some(unit) = self.contents.pop() {
            self.width -= unit.width;
            Some(unit)
        } else {
            None
        }
    }

    /// Converts this line into a `TextLineSplit` by trying to split at the last word boundary,
    /// unless [`WORD_WRAP_LENGTH_THRESHOLD`] is reached, in which case the line will be preserved
    /// and the split fragment will be empty.
    ///
    /// This function should be called when the line has already reached its maximum size.
    fn split_at_word_boundary(self, max_size: SizeConstraint) -> TextLineSplit {
        let min_width = max_size
            .width
            .map(|mw| (mw as f32 * WORD_WRAP_LENGTH_THRESHOLD) as Pixel);

        let mut cursor = LineSplitIter::new(self);
        let mut breaking_whitespace = false;

        loop {
            // Iterating backwards from the end of the line.
            // First, look for a character
            let Some((unit, width_at_unit)) = cursor.advance() else {
                // This shouldn't really ever happen, due to the `WORD_WRAP_LENGTH_THRESHOLD`
                log::warn!("ran out of characters");
                return cursor.confirm();
            };

            // Attempting to break a line here
            if breaking_whitespace || unit.break_kind == BreakKind::WordBoundary {
                if unit.is_whitespace() {
                    // We will keep eating whitespace until we find a non-whitespace character to end the line with
                    breaking_whitespace = true;
                    continue;
                } else {
                    // This is a non-whitespace character.  If we
                    //  a) are just now reaching a word boundary for the first time, or
                    //  b) have already skipped past some whitespace,
                    // keep this character in the line and split afterwards
                    cursor.retreat();
                    return cursor.confirm();
                }
            }

            // If we can't break a line here, but there's a minimum width threshold and we've hit it,
            // cancel the entire operation
            if matches!(min_width, Some(min_width) if width_at_unit < min_width) {
                return cursor.cancel();
            }

            // Otherwise we haven't found a line break or hit the minimum width, so just keep going
            continue;
        }
    }
}
impl Drawable for TextLine {
    fn draw_at(&self, handle: &mut DrawHandle, position: Position) {
        let mut cursor = position.x;
        self.contents.iter().for_each(|unit| {
            if let Some(ref raster) = unit.raster {
                raster.draw_at(
                    handle,
                    Position {
                        x: position.x + cursor,
                        y: position.y,
                    },
                );
            }

            cursor += unit.width;
        });
    }
}

pub struct TextBody {
    lines: Vec<TextLine>,

    /// `true` if the contents were **unintentionally** truncated;
    /// only if [`Wrapping::SingleLineTruncate`] was not used.
    ///
    /// Set by the `truncate` function
    truncation_warning: bool,
}

impl TextBody {
    fn height(&self) -> Pixel {
        self.lines.iter().fold(0, |res, val| res + val.height())
    }

    fn size(&self) -> Size {
        Size {
            width: self
                .lines
                .iter()
                .fold(0, |res, val| Pixel::max(res, val.width)),
            height: self.height(),
        }
    }

    fn truncate(&mut self, max_size: SizeConstraint, with_warning: bool) {
        self.truncation_warning = with_warning;

        while !max_size.fits_height(self.height()) {
            if self.lines.pop().is_none() {
                log::warn!("out of lines");
                return;
            }
        }
        if let Some(line) = self.lines.last_mut() {
            // Keep popping units until we can fit an ellipsis in this line
            while let Some(unit) = line.pop() {
                // Make an ellipsis with the same font and size as this last unit
                let period = Raster::rasterize(unit.font, unit.height, '.');
                let ellipsis_width = period.advance * 3;
                let ellipsis_height = period.bitmap_height as Pixel;

                let ellipsis = vec![period.clone(), period.clone(), period.clone()]
                    .into_iter()
                    .map(|raster| Unit {
                        raster: Some(raster),
                        width: period.advance,
                        height: unit.height,
                        break_kind: BreakKind::Character,
                        font: unit.font,
                    })
                    .collect::<Vec<Unit>>();

                let trunc_size = Size {
                    width: line.width + ellipsis_width,
                    height: Pixel::max(line.height(), ellipsis_height),
                };

                if max_size.fits(trunc_size) {
                    ellipsis
                        .into_iter()
                        .for_each(|unit| line.contents.push(unit));

                    return;
                }
            }
            log::warn!("ran out of units trying to fit ellipsis");
        } else {
            log::warn!("no line to add ellipsis");
        }
    }

    /// Lays out a **single-line** text label (no word wrapping).
    /// Also used to deal with intrinsic sizing
    pub fn new_label(text: RichString, direction: Direction) -> Self {
        let Direction::Horizontal = direction;

        let mut line = TextLine::new();
        let mut iter = text.rich_iter();
        while let Some(rich_char) = iter.next() {
            line.push(Unit::rasterize(rich_char));
        }

        TextBody {
            lines: vec![line],
            truncation_warning: false,
        }
    }

    /// Attempts to fit text into the provided space,
    /// implementing wrapping if applicable according to the [`Wrapping`] value.
    ///
    /// The output of this method is ultimately what is rendered to the buffer.
    /// Whitespace is not preserved when it causes a line break.
    pub fn layout(text: RichString, options: TextOptions, max_size: SizeConstraint) -> Self {
        let Direction::Horizontal = options.direction;

        let mut result = TextBody {
            lines: Vec::new(),
            truncation_warning: false,
        };

        let mut current_line = TextLine::new();

        // We iterate over the text with the rich text iterator,
        // but we also maintain a backtrack queue because of word wrapping.
        let mut iter = text.rich_iter();
        let mut queue = VecDeque::<Unit>::new();

        while let Some(unit) = queue
            .pop_front()
            .or_else(|| iter.next().map(Unit::rasterize))
        {
            if current_line.would_fit(max_size, &unit) {
                current_line.push(unit);
                continue;
            } else if unit.is_whitespace() {
                /*
                   This could be stupid, but we always push whitespace, because:
                   - We never want whitespace to begin a line
                   - `split_at_word_boundary` will handle the case where the line ends on a break point
                   - If the whitespace character caused a line break,
                     `split_at_word_boundary` would go back to the previous break point instead of ending the line here
                   Later on, we will trim excess whitespace at the end of the last line
                */
                current_line.push(unit);
            } else {
                // Before starting a new line, check on our vertical max size,
                // since now we know how tall our current line is.
                // `with_warning` is always `true` here; intentional truncation only applies to the width
                if !max_size.fits_height(result.height()) {
                    result.truncate(max_size, true);
                    return result;
                }

                match options.wrapping {
                    Wrapping::Character => {
                        // new line, continue

                        result
                            .lines
                            .push(std::mem::replace(&mut current_line, TextLine::new()));
                    }
                    Wrapping::Word => {
                        // wrap line
                        let (line_result, backqueue) =
                            current_line.split_at_word_boundary(max_size);

                        result.lines.push(line_result);
                        current_line = TextLine::new();

                        backqueue.into_iter().for_each(|unit| queue.push_back(unit));
                    }
                    Wrapping::SingleLine => {
                        result.truncate(max_size, true);
                        return result;
                    }
                    Wrapping::SingleLineTruncate => {
                        result.truncate(max_size, false);
                        return result;
                    }
                }
            }
        }

        // Handle last line
        /*
           We have to trim whitespace from the end of the text body as a result of our wrapping algorithm,
           especially for the case where the whitespace overflows the max size.
           I'm misusing our "LineSplitIter" construct from above to do this,
           since it does what we need while updating the line's length,
           and I'll just ignore the width and split fragment parts.
        */
        let mut split_iter = LineSplitIter::new(current_line);
        while let Some((unit, _)) = split_iter.advance() {
            if unit.is_whitespace() {
                continue;
            } else {
                split_iter.retreat();
                break;
            }
        }
        // If we cleared the entire line and it was nothing but whitespace;
        // for now, we're going to push an empty line anyways.  Maybe this changes?

        let (current_line, _) = split_iter.confirm();
        result.lines.push(current_line);

        // one last truncation check; this is our final size
        // `with_warning` is true; intentional truncation should have already been handled
        if !max_size.fits(result.size()) {
            result.truncate(max_size, true);
        }

        result
    }
}

impl Drawable for TextBody {
    fn draw_at(&self, handle: &mut DrawHandle, position: Position) {
        //TODO: Multi line
        if let Some(first) = self.lines.first() {
            first.draw_at(handle, position);
        }
    }
}
