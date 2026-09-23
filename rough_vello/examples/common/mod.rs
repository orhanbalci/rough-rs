//! Minimal winit + vello window shared by the examples.
//!
//! Each example provides a draw callback that fills a [`Scene`] every frame;
//! this module takes care of the window, the wgpu surface and presenting.

use std::sync::Arc;
use std::time::Instant;

use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use vello::wgpu::{self, CurrentSurfaceTexture};
use vello::{AaConfig, AaSupport, Renderer, RendererOptions, Scene};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Frame information passed to the draw callback.
// Not every example reads every field.
#[allow(dead_code)]
pub struct Frame {
    /// Surface width in physical pixels.
    pub width: f64,
    /// Surface height in physical pixels.
    pub height: f64,
    /// Seconds since the window was created.
    pub elapsed: f64,
    /// Seconds since the previous frame.
    pub dt: f64,
}

/// Opens a window and redraws `draw` into it until it is closed.
///
/// When `animate` is true the window is redrawn continuously, otherwise only
/// when the system asks for it (e.g. on resize).
pub fn run(
    title: &str,
    background: Color,
    animate: bool,
    draw: impl FnMut(&mut Scene, &Frame) + 'static,
) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App {
        title: title.to_owned(),
        background,
        animate,
        draw: Box::new(draw),
        context: RenderContext::new(),
        renderer: None,
        state: None,
        scene: Scene::new(),
        start: Instant::now(),
        last_frame: Instant::now(),
    };
    event_loop.run_app(&mut app).expect("event loop failed");
}

type DrawFn = Box<dyn FnMut(&mut Scene, &Frame)>;

struct ActiveState {
    window: Arc<Window>,
    surface: RenderSurface<'static>,
}

struct App {
    title: String,
    background: Color,
    animate: bool,
    draw: DrawFn,
    context: RenderContext,
    renderer: Option<Renderer>,
    state: Option<ActiveState>,
    scene: Scene,
    start: Instant,
    last_frame: Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(LogicalSize::new(800, 600));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );

        let size = window.inner_size();
        let surface = pollster::block_on(self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        ))
        .expect("failed to create surface");

        let device = &self.context.devices[surface.dev_id].device;
        self.renderer.get_or_insert_with(|| {
            let options = RendererOptions {
                antialiasing_support: AaSupport::area_only(),
                ..Default::default()
            };
            Renderer::new(device, options).expect("failed to create renderer")
        });

        self.state = Some(ActiveState { window, surface });
        self.start = Instant::now();
        self.last_frame = self.start;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    self.context
                        .resize_surface(&mut state.surface, size.width, size.height);
                    state.window.request_redraw();
                }
            }
            WindowEvent::Occluded(false) => state.window.request_redraw(),
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let surface = &state.surface;
                let frame = Frame {
                    width: surface.config.width as f64,
                    height: surface.config.height as f64,
                    elapsed: (now - self.start).as_secs_f64(),
                    dt: (now - self.last_frame).as_secs_f64(),
                };
                self.last_frame = now;

                self.scene.reset();
                (self.draw)(&mut self.scene, &frame);

                let device_handle = &self.context.devices[surface.dev_id];
                let surface_texture = match surface.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(texture)
                    | CurrentSurfaceTexture::Suboptimal(texture) => texture,
                    CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                        self.context.configure_surface(surface);
                        state.window.request_redraw();
                        return;
                    }
                    // The window is not visible yet (or anymore); `Occluded(false)`
                    // requests a redraw once it is.
                    CurrentSurfaceTexture::Occluded => return,
                    _ => {
                        state.window.request_redraw();
                        return;
                    }
                };

                self.renderer
                    .as_mut()
                    .expect("renderer is created in resumed")
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
                        &surface.target_view,
                        &vello::RenderParams {
                            base_color: self.background,
                            width: surface.config.width,
                            height: surface.config.height,
                            antialiasing_method: AaConfig::Area,
                        },
                    )
                    .expect("failed to render scene");

                // Vello renders into an intermediate texture; blit it to the surface.
                let surface_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device_handle
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surface Blit"),
                        });
                surface.blitter.copy(
                    &device_handle.device,
                    &mut encoder,
                    &surface.target_view,
                    &surface_view,
                );
                device_handle.queue.submit([encoder.finish()]);
                surface_texture.present();

                if self.animate {
                    state.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
