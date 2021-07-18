pub mod render;
pub mod math;
pub mod events;
pub mod components;
pub mod input;
pub mod texture_packer;
pub mod render_batch;
pub mod systems;
pub mod ai;
pub mod world_setup;
pub mod map;
pub mod text_render;
pub mod ui;
pub mod collision;

use std::time::Instant;
use crate::input::Input;

#[derive(Debug)]
pub struct TimeStep {
    last_time:   Instant,
    delta_time:  f64,
    frame_count: u32,
    frame_time:  f64,
    last_frame_count: u32,
}

impl TimeStep {
    pub fn new() -> TimeStep {
        TimeStep {
            last_time:   Instant::now(),
            delta_time:  0.0,
            frame_count: 0,
            frame_time:  0.0,
            last_frame_count: 0,
        }
    }

    pub fn delta(&mut self) -> f64 {
        let current_time = Instant::now();
        let delta = current_time.duration_since(self.last_time).as_micros()
            as f64
            * 0.001;
        self.last_time = current_time;
        self.delta_time = delta;
        delta
    }

    // provides the framerate in FPS
    pub fn frame_rate(&mut self) -> u32 {
        self.frame_count += 1;
        self.frame_time += self.delta_time;
        let tmp;
        // per second
        if self.frame_time >= 1000.0 {
            tmp = self.frame_count;
            self.frame_count = 0;
            self.frame_time = 0.0;
            self.last_frame_count = tmp;
            return tmp;
        }
        self.last_frame_count
    }
}

pub mod core {
    use super::*;
    use glyph_brush::ab_glyph::Rect;
    use lazy_static::lazy_static;
    use crate::render::*;
    use crate::math::*;
    use crate::render::Color;
    use crate::text_render::TextRenderer;
    use sdl2::video::SwapInterval;
    use sdl2::video::Window;
    use sdl2::EventPump;
    use sdl2::event::Event;
    use sdl2::video::GLContext;
    use sdl2::video::GLProfile;
    use std::sync::Mutex;
    use sdl2::event::WindowEvent;

    pub type Keycode = sdl2::keyboard::Keycode;
    pub type MouseButton = sdl2::mouse::MouseButton;

    pub const DEFAULT_SCALE: i32 = 3;

    pub const RENDER_RES_W: i32 = 320;
    pub const RENDER_RES_H: i32 = 240;

    const W: i32 = RENDER_RES_W * DEFAULT_SCALE;
    const H: i32 = RENDER_RES_H * DEFAULT_SCALE;

    

    lazy_static! {
        static ref RENDER_CONTEXT: Mutex<Renderer> = {
            Mutex::new(Renderer::default(W, H))
        };
    }

    lazy_static! {
        static ref TEXT_RENDER_CONTEXT: Mutex<TextRenderer> = {
            Mutex::new(TextRenderer::new(W, H))
        };
    }

    pub static mut CONTEXT: Option<Smol> = None;


    pub fn get_context() -> &'static mut Smol {
        unsafe { CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
    }

    pub fn get_render_context() -> std::sync::MutexGuard<'static, render::Renderer> {
        RENDER_CONTEXT.lock().unwrap()
    }

    pub fn get_text_render_context() -> std::sync::MutexGuard<'static, TextRenderer> {
        TEXT_RENDER_CONTEXT.lock().unwrap()
    }

    pub fn queue_text(text: &str, position: Vector2, font_size: f32, color: Color) -> Option<Rect> {
        get_text_render_context().queue_text(text, position, font_size, color)
    }

    pub fn render_text_queue() {
        get_text_render_context().render_queue();
    }

    pub fn clear(color: Color) {
        Renderer::clear(color);
    }

    pub fn render_framebuffer_scale(texture: &Texture, position: Vector2, scale: Vector2) {
        get_render_context().framebuffer_texture_scale(texture, position, scale);
    }

    pub fn render_rect(x: f32, y: f32, width: f32, height: f32, color: Color) {
        get_render_context().rect(
           width, height, x, y, color
        );
    }

    pub fn load_texture(src: &str) -> Texture {
        Texture::load_from_file(src)
    }

    pub fn load_texture_from_bytes(bytes: &[u8]) -> Texture {
        Texture::load_from_bytes(bytes)
    }

    pub fn render_texture(texture: &Texture, position: Vector2) {
        get_render_context().texture(texture, position);
    }

    pub fn render_texture_scale(texture: &Texture, position: Vector2, scale: f32) {
        get_render_context().texture_scale(texture, position, scale);
    }

    
    pub fn render_texture_partial(texture: &PartialTexture, position: Vector2) {
        get_render_context().render_texture_partial(&texture, position);
    }

    // pub fn render_texture_to_rect(texture: &Texture, position: Vector2, ) {
    //     get_render_context().texture_rect_scale(&texture, )
    // }

    pub fn capture_framebuffer() {
        get_render_context().frame_buffer.bind();
    }

    pub fn stop_capture_framebuffer() {
        get_render_context().frame_buffer.unbind();
    }

    pub fn render_framebuffer(position: Vector2, scale: f32) {
        let render_context = get_render_context();
        let texture = &render_context.frame_buffer.texture;
        
        render_context.texture_scale(texture, position, scale);
    }

    pub fn get_window_size() -> Vector2Int {
        get_context().window_size
    }


    pub fn get_window_scale() -> f32 {
        let pixel_size = Vector2 { x: RENDER_RES_W as f32, y: RENDER_RES_H as f32 };
        let window_size: Vector2 = get_context().window_size.into();
        let value = (window_size / pixel_size).x;
        f32::min(f32::max(2., value), 5.)
    }

    pub fn get_screen_center() -> Vector2Int {
        let ctx = get_context();
        let (x, y) = ctx.window.size();
        Vector2Int::new(x as i32 / 2, y as i32 / 2)
    }

    pub fn is_running() -> bool {
        let ctx = get_context();
        ctx.running
    }

    pub fn end_render() {
        render_text_queue();
        let ctx = get_context();
        ctx.delta_time = ctx.time_step.delta() as f32;
        ctx.window.gl_swap_window();
        for event in ctx.event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    ctx.running = false;
                },
                Event::Window {
                    win_event,
                    ..
                } => {
                    match win_event {
                        WindowEvent::Resized(w, h) => {
                            get_render_context().set_viewport(0.0, 0.0, w as u32, h as u32);
                            get_render_context().set_projection(w as f32, h as f32);
                            ctx.window_size.x = w;
                            ctx.window_size.y = h;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        ctx.input.set_keys(&ctx.event_pump);
        ctx.input.set_mouse_state(&ctx.event_pump);
    }

    pub fn delta_time() -> f32 {
        get_context().delta_time
    }


    pub fn init() {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        
       
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(4, 1);

        let screen_width = W;  
        let screen_height = H; 
        let window = video_subsystem.window("Window", W as u32, H as u32)
            .opengl()
            .resizable()
            .build()
            .unwrap();
    
        // Unlike the other example above, nobody created a context for your window, so you need to create one.
        let _gl_context = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        let _ = video_subsystem.gl_set_swap_interval(SwapInterval::VSync);
        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        let event_pump = sdl_context.event_pump().unwrap();
       
        get_render_context().set_viewport(0.0, 0.0, screen_width as u32, screen_height as u32);
        get_render_context().set_projection(screen_width as f32, screen_height as f32);
        
        unsafe {
            CONTEXT = Option::from(
                Smol { 
                    running: true,
                    window,
                    event_pump,
                    _gl_context,
                    time_step: TimeStep::new(),
                    delta_time: 0.0,
                    input: Input::new(),
                    window_size: Vector2Int {
                        x: screen_width as i32,
                        y: screen_height as i32
                    }
                }
            )
        };
    }

    pub struct Smol {
        pub running: bool,
        window: Window,
        event_pump: EventPump,
        _gl_context: GLContext,
        time_step: TimeStep,
        pub input: Input,
        delta_time: f32,
        window_size: Vector2Int
    }
}


