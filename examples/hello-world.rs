use glyphon::{
    fontdb::{self, Query}, Attrs, Buffer, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer
};
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use std::{path::Path, sync::Arc};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // Set up window
    let (width, height) = (1600, 900);
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_inner_size(LogicalSize::new(width as f64, height as f64))
        .with_title("glyphon hello world")
        .build(&event_loop)
        .unwrap());
    let size = window.inner_size();
    let scale_factor = window.scale_factor();
    let mut frame_count: u32 = 0;
    // Set up surface
    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::default(), //downlevel_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    let surface = instance.create_surface(window.clone()).expect("Create surface");
    let swapchain_format = TextureFormat::Bgra8UnormSrgb;
    let mut config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);


    //let mut font_db = fontdb::Database::new();
    //let font_path = Path::new("./MartianMono-Regular.ttf"); // Assuming font is in the project directory
    //font_db.load_font_file(font_path).unwrap();



    //font_db.face(id)
    //let font = font_db. .get_font(font_id);
    
    // Set up text renderer
    let mut font_system = FontSystem::new();

    //let ids = font_db.query(&Query::)

    //let font = font_system.get_font(font_db)
    //let font = font_system.get_font(font_id).unwrap();
    let mut cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&device, &queue, swapchain_format);
    let mut text_renderer =
        TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(430.0, 500.0));

    let physical_width = (width as f64 * scale_factor) as f32;
    let physical_height = (height as f64 * scale_factor) as f32;

    buffer.set_size(&mut font_system, physical_width, physical_height);
    buffer.set_text(&mut font_system, "What's up gamers!", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system);

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(size) => {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&device, &config);
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        text_renderer
                            .prepare(
                                &device,
                                &queue,
                                &mut font_system,
                                &mut atlas,
                                Resolution {
                                    width: config.width,
                                    height: config.height,
                                },
                                [TextArea {
                                    buffer: &buffer,
                                    left: 200.0 + frame_count as f32,// + 100.0*f32::sin(frame_count as f32 * 0.01),
                                    top: 200.0 + 100.0*f32::cos(frame_count as f32 * 0.01),
                                    scale: 1.0 + 0.001* frame_count as f32,
                                    bounds: TextBounds {
                                        left: 0 + frame_count as i32,
                                        top: 0,
                                        right: 1600 * 2,
                                        bottom: 900 * 2,
                                    },
                                    default_color: Color::rgb(255, 255, 255),
                                }],
                                &mut cache,
                            )
                            .unwrap();

                        let frame = surface.get_current_texture().unwrap();
                        let view = frame.texture.create_view(&TextureViewDescriptor::default());
                        let mut encoder = device
                            .create_command_encoder(&CommandEncoderDescriptor { label: None });
                        {
                            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: Operations {
                                        load: LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            text_renderer.render(&atlas, &mut pass).unwrap();
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                        frame_count += 1;
                        atlas.trim();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        })
        .unwrap();
}
