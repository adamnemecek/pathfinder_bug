// pathfinder/examples/canvas_metal_minimal/src/main.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use foreign_types::ForeignTypeRef;
use metal::{CAMetalLayer, CoreAnimationLayerRef};
use pathfinder_canvas::{Canvas, CanvasFontContext, Path2D};
use pathfinder_color::ColorF;
use pathfinder_geometry::vector::{vec2f, Vector2F, vec2i};
use pathfinder_geometry::rect::RectF;
use pathfinder_metal::MetalDevice;
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use sdl2::event::Event;
use sdl2::hint;
use sdl2::keyboard::Keycode;
use sdl2_sys::SDL_RenderGetMetalLayer;
use std::sync::Arc;
use std::time::Instant;

use pathfinder_resources::ResourceLoader;
use pathfinder_resources::fs::FilesystemResourceLoader;
use font_kit::handle::Handle;
use font_kit::sources::mem::MemSource;


mod nanovg;
pub use nanovg::*;

fn main() {
    // Set up SDL2.
    assert!(hint::set("SDL_RENDER_DRIVER", "metal"));
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    // Open a window.
    let width = 1024;
    let height = width * 3 / 4;

    let window_size = vec2i(width, height);
    let window = video.window("Minimal example", window_size.x() as u32, window_size.y() as u32)
                      .opengl()
                      .build()
                      .unwrap();

    // Create a Metal context.
    let canvas = window.into_canvas().present_vsync().build().unwrap();
    let metal_layer = unsafe {
        CoreAnimationLayerRef::from_ptr(SDL_RenderGetMetalLayer(canvas.raw()) as *mut CAMetalLayer)
    };
    let metal_device = metal_layer.device();
    let drawable = metal_layer.next_drawable().unwrap();

    // Create a Pathfinder renderer.
    let device = unsafe {
        MetalDevice::new(metal_device, drawable.clone())
    };
    let mode = RendererMode::default_for_device(&device);
    let options = RendererOptions {
        dest: DestFramebuffer::full_window(window_size),
        background_color: Some(ColorF::white()),
        ..RendererOptions::default()
    };

    // Load demo data.
    let resources = FilesystemResourceLoader::locate();
    let font_data = vec![
        Handle::from_memory(Arc::new(resources.slurp("fonts/Roboto-Regular.ttf").unwrap()), 0),
        Handle::from_memory(Arc::new(resources.slurp("fonts/Roboto-Bold.ttf").unwrap()), 0),
        Handle::from_memory(Arc::new(resources.slurp("fonts/NotoEmoji-Regular.ttf").unwrap()), 0),
    ];
    // let demo_data = DemoData::load(&resources);

    let mut renderer = Renderer::new(device, &EmbeddedResourceLoader, mode, options);
    // Initialize font state.
    let font_source = Arc::new(MemSource::from_fonts(font_data.into_iter()).unwrap());
    let font_context = CanvasFontContext::new(font_source.clone());


    // Make a canvas. We're going to draw a house.
    let canvas = Canvas::new(window_size.to_f32());
    let mut canvas = canvas.get_context_2d(font_context.clone());


    // Set line width.
    canvas.set_line_width(10.0);

    // Draw walls.
    canvas.stroke_rect(RectF::new(vec2f(75.0, 140.0), vec2f(150.0, 110.0)));

    // Draw door.
    canvas.fill_rect(RectF::new(vec2f(130.0, 190.0), vec2f(40.0, 60.0)));

    // Draw roof.
    let mut path = Path2D::new();
    path.move_to(vec2f(50.0, 140.0));
    path.line_to(vec2f(150.0, 60.0));
    path.line_to(vec2f(250.0, 140.0));
    path.close_path();
    canvas.stroke_path(path);

    let mut mouse_position = Vector2F::zero();
    let start_time = Instant::now();
    let frame_start_time = Instant::now();
    let frame_start_elapsed_time = (frame_start_time - start_time).as_secs_f32();
    let hidpi_factor = 1;

    render_demo(&mut canvas,
                mouse_position,
                vec2f(width as f32, height as f32),
                frame_start_elapsed_time,
                hidpi_factor as f32);

    // Render the canvas to screen.
    let mut scene = SceneProxy::from_scene(canvas.into_canvas().into_scene(),
                                           renderer.mode().level,
                                           RayonExecutor);
    scene.build_and_render(&mut renderer, BuildOptions::default());
    renderer.device().present_drawable(drawable);

    // Wait for a keypress.
    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        match event_pump.wait_event() {
            Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return,
            _ => {}
        }
    }
}
