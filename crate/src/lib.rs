use std::borrow::Cow;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowExtWebSys,
    window::Window,
};

mod gui;
mod utils;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn spawn(parent: HtmlElement) -> Result<(), JsValue> {
    console_log::init().expect("could not initialize logger");

    // The parent needs to be position: relative.
    parent
        .style()
        .set_property("position", "relative")
        .unwrap_throw();

    let (_gpu_canvas, event_loop, window) = spawn_web_gpu_canvas(&parent)?;
    let (_ui_canvas, ui_ctx) = spawn_ui_canvas(&parent)?;

    wasm_bindgen_futures::spawn_local(run(parent, event_loop, window, ui_ctx));

    Ok(())
}

fn spawn_web_gpu_canvas(
    parent: &web_sys::HtmlElement,
) -> Result<(HtmlCanvasElement, EventLoop<()>, Window), JsValue> {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let canvas = window.canvas();

    parent.append_child(&web_sys::Element::from(canvas.clone()))?;

    canvas.set_width(parent.client_width() as u32);
    canvas.set_height(parent.client_height() as u32);

    let style = canvas.style();
    style.set_property("position", "absolute")?;
    style.set_property("inset", "0")?;

    Ok((canvas, event_loop, window))
}

fn spawn_ui_canvas(
    parent: &web_sys::HtmlElement,
) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let canvas = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.create_element("canvas").ok())
        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| JsValue::from_str("Failed to create canvas"))?;

    parent.append_child(&canvas)?;

    canvas.set_width(parent.client_width() as u32);
    canvas.set_height(parent.client_height() as u32);

    let style = canvas.style();
    style.set_property("position", "absolute")?;
    style.set_property("inset", "0")?;

    let context = canvas
        .get_context("2d")?
        .and_then(|ctx| ctx.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
        .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?;

    Ok((canvas, context))
}

async fn run(
    parent: HtmlElement,
    event_loop: EventLoop<()>,
    window: Window,
    ui_ctx: CanvasRenderingContext2d,
) {
    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can
                // support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    let mut demo_ui = gui::DemoUi::new();

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // let frame = surface
                //     .get_current_texture()
                //     .expect("Failed to acquire next swap chain texture");
                // let view = frame
                //     .texture
                //     .create_view(&wgpu::TextureViewDescriptor::default());
                // let mut encoder =
                //     device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                // {
                //     let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                //         label: None,
                //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                //             view: &view,
                //             resolve_target: None,
                //             ops: wgpu::Operations {
                //                 load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                //                 store: true,
                //             },
                //         })],
                //         depth_stencil_attachment: None,
                //     });
                //     rpass.set_pipeline(&render_pipeline);
                //     rpass.draw(0..3, 0..1);
                // }

                // queue.submit(Some(encoder.finish()));
                // frame.present();

                let canvas = ui_ctx.canvas().unwrap();

                // Resize the canvas to match the parent size, if needed.
                let parent_width = parent.client_width() as u32;
                let parent_height = parent.client_height() as u32;
                if canvas.width() != parent_width || canvas.height() != parent_height {
                    canvas.set_width(parent_width);
                    canvas.set_height(parent_height);
                }

                ui_ctx.set_font("16px Courier New");
                ui_ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

                demo_ui.draw(&ui_ctx);
                // ui_ctx.set_fill_style(&JsValue::from_str("white"));
                // ui_ctx
                //     .fill_text("Hello, UI Canvas!", 100.0, 800.0)
                //     .unwrap_throw();
            }
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
