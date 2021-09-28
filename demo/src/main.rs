#![allow(dead_code, clippy::collapsible_match)]

use backtrace::Backtrace;
use camera::controls::{FlatControls, FlatSettings};
use camera::Projection;
use input::{Bindings, FrameTime, InputHandler};
use lazy_static::lazy_static;
use naga::{front::wgsl, valid::Validator};
use serde::{Deserialize, Serialize};
use slog::{error, info};
use sloggers::file::FileLoggerBuilder;
use sloggers::types::Severity;
use sloggers::Build;
use std::collections::HashMap;
use std::panic;
use std::{fs, path::PathBuf};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod gamestate;
mod graphics;

use gamestate::*;
use graphics::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum Action {
    Quit,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum Axis {
    Forward,
    Sideward,
    Yaw,
    Pitch,
}

lazy_static! {
    static ref LOGGER: slog::Logger = {
        let mut builder = FileLoggerBuilder::new("paniclog.txt");
        builder.level(Severity::Debug);
        builder.build().unwrap()
    };
}

#[tokio::main]
async fn main() -> Result<(), RendererError> {
    info!(LOGGER, "starting up");
    env_logger::init();

    /*panic::set_hook(Box::new(|panic_info| {
        let bt = Backtrace::new();

        error!(LOGGER, "PANIC: {}, BACKTRACE: {:?}", panic_info, bt);
    }));*/

    parse_example_wgsl();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Demo")
        .build(&event_loop)
        .unwrap();

    let backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(backends);

    let mut renderer = instance
        .create_renderer(
            window,
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
            },
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::TEXTURE_BINDING_ARRAY,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
            wgpu::PresentMode::Fifo,
        )
        .await
        .unwrap();

    let mut layout_storage = LayoutStorage::new();
    let mut sprite_atlas = Atlas::new(renderer.device(), 2048);
    let texture = Texture::from_file("images/Tree.png")?;

    let allocation = sprite_atlas
        .upload(&texture, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;
    let mut sprite = Sprite::new(allocation);

    sprite.pos[0] = 0;
    sprite.pos[1] = 0;
    sprite.pos[2] = 1;
    sprite.hw[0] = 64;
    sprite.hw[1] = 64;
    sprite.uv = [0, 0, 80, 64];
    sprite.changed = true;

    let sprite_texture =
        TextureGroup::from_atlas(renderer.device(), &mut layout_storage, &sprite_atlas);

    let sprite_pipeline = SpriteRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let settings = FlatSettings { zoom: 1.5 };

    let controls = FlatControls::new(settings);
    let camera = Camera::new(
        &renderer,
        &mut layout_storage,
        Projection::Orthographic {
            left: 0.0,
            right: 800.0,
            bottom: 0.0,
            top: 600.0,
            near: 1.0,
            far: -100.0,
        },
        controls,
    );

    let sprite_buffer = SpriteBuffer::new(renderer.device());
    let mut state = State {
        sprite,
        sprite_pipeline,
        sprite_atlas,
        layout_storage,
        sprite_texture,
        camera,
        sprite_buffer,
    };

    println!("{:?}", state.camera.projection());

    let mut views = HashMap::new();

    let size = wgpu::Extent3d {
        width: renderer.size().width,
        height: renderer.size().height,
        depth_or_array_layers: 1,
    };

    let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("depth texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut size = renderer.size();

    views.insert("depthbuffer".to_string(), view);

    let mut bindings = Bindings::<Action, Axis>::new();
    bindings.insert_action(
        Action::Quit,
        vec![winit::event::VirtualKeyCode::Q.into()].into_iter(),
    );
    let mut input_handler = InputHandler::new(bindings);

    let mut frame_time = FrameTime::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } if window_id == renderer.window().id() => {
                if let WindowEvent::CloseRequested = *event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }

        if size != renderer.size() {
            size = renderer.size();

            state.camera.set_projection(Projection::Orthographic {
                left: 0.0,
                right: size.width as f32,
                bottom: 0.0,
                top: size.height as f32,
                near: 1.0,
                far: -100.0,
            });

            let size = wgpu::Extent3d {
                width: renderer.size().width,
                height: renderer.size().height,
                depth_or_array_layers: 1,
            };

            let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("depth texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            views.insert("depthbuffer".to_string(), view);
        }

        input_handler.update(renderer.window(), &event, 1.0);

        let frame = match renderer.update(&event).unwrap() {
            Some(frame) => frame,
            _ => return,
        };

        if input_handler.is_action_down(&Action::Quit) {
            *control_flow = ControlFlow::Exit;
        }

        let camera = &mut state.camera;
        let delta = frame_time.delta_seconds();
        camera.update(&renderer, delta);

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        views.insert("framebuffer".to_string(), view);

        state.sprite.update();

        let mut bytes = vec![];
        let count = 6;

        bytes.append(&mut state.sprite.bytes.clone());

        state.sprite_buffer.set_buffer(renderer.queue(), &bytes);
        state.sprite_buffer.set_indice_count(count as u64);
        // Start encoding commands.
        let mut encoder =
            renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command encoder"),
                });

        // Run the render pass.
        state.render(&mut encoder, &views);

        // Submit our command queue.
        renderer.queue().submit(std::iter::once(encoder.finish()));

        views.remove("framebuffer");

        input_handler.end_frame();
        frame_time.update();
    });
}

pub fn parse_example_wgsl() {
    let read_dir = match PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .read_dir()
    {
        Ok(iter) => iter,
        Err(e) => {
            println!("Unable to open the examples folder: {:?}", e);
            return;
        }
    };
    for example_entry in read_dir {
        let read_files = match example_entry {
            Ok(dir_entry) => match dir_entry.path().read_dir() {
                Ok(iter) => iter,
                Err(_) => continue,
            },
            Err(e) => {
                println!("Skipping example: {:?}", e);
                continue;
            }
        };
        for file_entry in read_files {
            let shader = match file_entry {
                Ok(entry) => match entry.path().extension() {
                    Some(ostr) if &*ostr == "wgsl" => {
                        println!("Validating {:?}", entry.path());
                        fs::read_to_string(entry.path()).unwrap_or_default()
                    }
                    _ => continue,
                },
                Err(e) => {
                    println!("Skipping file: {:?}", e);
                    continue;
                }
            };

            let module = wgsl::parse_str(&shader).unwrap();
            //TODO: re-use the validator
            Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            )
            .validate(&module)
            .unwrap();
        }
    }
}
