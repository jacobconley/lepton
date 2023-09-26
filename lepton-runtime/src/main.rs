use std::{num::NonZeroU32, sync::OnceLock};

use graphics::{typeface::Typeface, Text};
use render::DrawHandle;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

/// Graphics primitives, such as typeface and color
pub mod graphics;

/// Rendering engine implementation
pub mod render;

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

                    DrawHandle {
                        buffer: &mut buffer,
                        width: width as usize,
                    }
                    .text(Text::new(
                        "Sphinx of black quartz, judge my vow".to_owned(),
                        typeface(),
                    ));

                    buffer.present().unwrap();
                }
                _ => (),
            }
        })
        .unwrap();
}
