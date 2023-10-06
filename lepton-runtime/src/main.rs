/// Graphics primitives, such as typeface and color
pub mod graphics;

/// Rendering engine implementation
///
/// This also includes `text`, which has some layout implementation,
/// because the layout is driven by rasterization which happens in here
pub mod render;

pub mod layout;

use std::{num::NonZeroU32, sync::OnceLock};

use crate::graphics::{text::RichString, typeface::Typeface};
use crate::layout::Position;
use crate::render::{DrawHandle, Drawable, text::TextBody};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn typeface() -> &'static Typeface {
    static MEM: OnceLock<Typeface> = OnceLock::new();
    MEM.get_or_init(|| Typeface::try_default().unwrap())
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Lepton Runtime")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    let sentence = 
        // "The quick brown fox jumps over the lazy dog"
        "Sphinx of black quartz, judge my vow"
        // "Pack my box with five dozen liquor jugs"
        ;

    let _paragraph = concat!(
        "I'm on the highway, life in the dark - still it's your face, won't leave me alone.",
        "I'm in that fast lane, riding from my wrongs,",
        "and when I lose my faith, I'm hopeless but I'm yours.",
        "Staring at Beretta Lake",

        "Climbing branches up to brick red rooftops, Lilly keeps her secrets to herself.",
        "Longing for the low end, hearing nothing much worth hearing, until she heard something else -",
        "locked and waiting for the key...",
        "Watch the morning into sundown, what Lilly says is there inside her stare",
        "Chase the sun across the pavement, where the cars are warm and parked inside the lines.",
        "Knowing nothing but exactly what he knows, and he knows what's better.",
        "Up the arm rest, scale the branches of the olive tree",
        "Wait until the gradient begins to form, Simon says to let him in",
    );

    event_loop
        .run(move |event, _elwt, ctrl| {
            // elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => ctrl.set_exit(),

                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width, size.height)
                    };
                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();

                    let mut buffer = surface.buffer_mut().unwrap();
                    for index in 0..(width * height) {
                        buffer[index as usize] = 0x0;
                    }

                    let mut handle = DrawHandle {
                        buffer: &mut buffer,
                        width: width as usize,
                    };

                    let text = RichString::new(sentence.to_owned(), typeface());

                    TextBody::new_label(text, graphics::text::Direction::Horizontal).draw_at(&mut handle, Position {
                        x: 15,
                        y: 15
                    });

                    buffer.present().unwrap();
                }
                _ => (),
            }
        })
        .unwrap();
}
