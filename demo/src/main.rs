#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use ::camera::{
    controls::{FlatControls, FlatSettings},
    Projection,
};
use backtrace::Backtrace;
use input::{Bindings, FrameTime, InputHandler};
use log::{error, info, warn, Level, LevelFilter, Metadata, Record};
use naga::{front::wgsl, valid::Validator};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    panic,
    path::PathBuf,
};
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

static MY_LOGGER: MyLogger = MyLogger(Level::Debug);

struct MyLogger(pub Level);

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{} - {}\n", record.level(), record.args());
            println!("{}", &msg);

            let mut file = match File::options()
                .append(true)
                .create(true)
                .open("paniclog.txt")
            {
                Ok(v) => v,
                Err(_) => return,
            };

            let _ = file.write(msg.as_bytes());
        }
    }
    fn flush(&self) {}
}

#[tokio::main]
async fn main() -> Result<(), RendererError> {
    log::set_logger(&MY_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    info!("starting up");

    panic::set_hook(Box::new(|panic_info| {
        let bt = Backtrace::new();

        error!("PANIC: {}, BACKTRACE: {:?}", panic_info, bt);
    }));

    //parse_example_wgsl();
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
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            },
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
            wgpu::PresentMode::Fifo,
        )
        .await
        .unwrap();

    println!("{:?}", renderer.adapter().get_info());
    let mut layout_storage = LayoutStorage::new();
    let mut sprite_atlas = Atlas::new(renderer.device(), 2048);
    let texture = Texture::from_file("images/Female_1.png")?;

    let allocation = sprite_atlas
        .upload(&texture, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;
    let mut sprite = [Sprite::new(allocation), Sprite::new(allocation)];

    sprite[0].pos = [32, 32, 5];
    sprite[0].hw = [48, 48];
    sprite[0].uv = [48, 96, 48, 48];
    sprite[0].color = [100, 0, 100, 100];
    sprite[0].changed = true;

    sprite[1].pos = [64, 32, 6];
    sprite[1].hw = [48, 48];
    sprite[1].uv = [48, 96, 48, 48];
    sprite[1].color = [0, 0, 100, 100];
    sprite[1].changed = true;

    let sprite_texture = TextureGroup::from_view(
        renderer.device(),
        &mut layout_storage,
        &sprite_atlas.texture_view,
        TextureLayout,
    );

    let sprite_pipeline = SpriteRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let settings = FlatSettings { zoom: 2.0 };

    let size = renderer.size();
    let controls = FlatControls::new(settings);
    let camera = Camera::new(
        &renderer,
        &mut layout_storage,
        Projection::Orthographic {
            left: 0.0,
            right: size.width as f32,
            bottom: 0.0,
            top: size.height as f32,
            near: 1.0,
            far: -100.0,
        },
        controls,
    );

    let sprite_buffer = GpuBuffer::with_capacity(renderer.device(), 1);

    let mut map = Map::new();

    (0..32).for_each(|x| {
        (0..32).for_each(|y| {
            map.set_tile((x, y, 0), 1, 0, 0, 100);
        });
    });

    map.set_tile((1, 31, 1), 2, 0, 0, 100);
    map.set_tile((1, 30, 6), 2, 0, 0, 80);
    map.set_tile((0, 0, 1), 2, 0, 0, 100);
    map.pos = [32, 32];
    let map_pipeline = MapRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let mut map_atlas = Atlas::new(renderer.device(), 2048);

    for i in 0..3 {
        let texture = Texture::from_file(format!("images/tiles/{}.png", i))?;
        let _ = map_atlas
            .upload(&texture, renderer.device(), renderer.queue())
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
    }

    let mut map_textures = MapTextures::new(renderer.device(), 81);
    let map_group = TextureGroup::from_view(
        renderer.device(),
        &mut layout_storage,
        &map_textures.texture_view,
        MapLayout,
    );
    let map_texture = TextureGroup::from_view(
        renderer.device(),
        &mut layout_storage,
        &map_atlas.texture_view,
        TextureLayout,
    );

    let maplower_buffer = GpuBuffer::with_capacity(renderer.device(), 540);
    let mapupper_buffer = GpuBuffer::with_capacity(renderer.device(), 180);

    map.layer = map_textures
        .get_unused_id()
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let mut animation_atlas = Atlas::new(renderer.device(), 2048);
    let texture = Texture::from_file("images/anim/0.png")?;
    let allocation = animation_atlas
        .upload(&texture, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let animation_pipeline = AnimationRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;
    let animation_buffer = GpuBuffer::new(renderer.device());
    let animation_texture = TextureGroup::from_view(
        renderer.device(),
        &mut layout_storage,
        &animation_atlas.texture_view,
        TextureLayout,
    );

    let mut animation = Animation::new(allocation);
    animation.pos = [0.0, 0.0, 1.0];
    animation.hw = [64; 2];
    animation.anim_hw = [64; 2];
    animation.frames = 8;
    animation.frames_per_row = 4;
    animation.switch_time = 300;

    let time_group = TimeGroup::new(&renderer, &mut layout_storage);

    let shapes_pipeline = ShapeRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let shapes_buffer = GpuBuffer::new(renderer.device());

    let mut shapes = Shape::new();
    shapes.push_point(200.0, 200.0, 1.0);
    shapes.push_point(216.0, 200.0, 1.0);
    shapes.push_point(216.0, 216.0, 1.0);
    shapes.push_point(200.0, 216.0, 1.0);
    shapes.closed = true;
    shapes.set_fill(true);

    let mut state = State {
        layout_storage,
        camera,
        time_group,
        sprite,
        sprite_pipeline,
        sprite_buffer,
        sprite_atlas,
        sprite_texture,
        map,
        map_pipeline,
        maplower_buffer,
        mapupper_buffer,
        map_texture,
        map_group,
        map_atlas,
        map_textures,
        animation,
        animation_buffer,
        animation_pipeline,
        animation_atlas,
        animation_texture,
        shapes,
        shapes_buffer,
        shapes_pipeline,
    };

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
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
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

        let test_size = renderer.size();

        if test_size.width == 0 || test_size.height == 0 {
            size = test_size;
            return;
        }

        if size != test_size {
            size = test_size;

            state.camera.set_projection(Projection::Orthographic {
                left: 0.0,
                right: size.width as f32,
                bottom: 0.0,
                top: size.height as f32,
                near: 1.0,
                far: -100.0,
            });

            let size = wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            };

            let texture =
                renderer.device().create_texture(&wgpu::TextureDescriptor {
                    label: Some("depth texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_DST,
                });
            let view =
                texture.create_view(&wgpu::TextureViewDescriptor::default());

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

        state.time_group.update(&renderer, frame_time.seconds());
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        views.insert("framebuffer".to_string(), view);

        state.sprite[0].update();
        state.sprite[1].update();

        let mut bytes = state.sprite[0].bytes.clone();
        bytes.extend_from_slice(&state.sprite[1].bytes);

        state.sprite_buffer.set_vertices_from(
            renderer.device(),
            renderer.queue(),
            &bytes,
        );

        state.map.update(renderer.queue(), &mut state.map_textures);

        state.maplower_buffer.set_vertices_from(
            renderer.device(),
            renderer.queue(),
            &state.map.lowerbytes,
        );

        state.mapupper_buffer.set_vertices_from(
            renderer.device(),
            renderer.queue(),
            &state.map.upperbytes,
        );

        state.animation.update();

        state.animation_buffer.set_vertices_from(
            renderer.device(),
            renderer.queue(),
            &state.animation.bytes,
        );

        state.shapes.update();

        state.shapes_buffer.set_vertices_from(
            renderer.device(),
            renderer.queue(),
            &state.shapes.buffers.vertices,
        );

        state
            .shapes_buffer
            .set_indices_from(renderer.queue(), &state.shapes.buffers.indices);

        // Start encoding commands.
        let mut encoder = renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            },
        );

        // Run the render pass.
        state.render(&mut encoder, &views);

        // Submit our command queue.
        renderer.queue().submit(std::iter::once(encoder.finish()));

        views.remove("framebuffer");

        input_handler.end_frame();
        frame_time.update();
        frame.present();
    })
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
                    Some(ostr) if ostr == "wgsl" => {
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

            let result = wgsl::parse_str(&shader);

            let module = match result {
                Ok(v) => (v, Some(shader)),
                Err(ref e) => {
                    e.emit_to_stderr(&shader);
                    return;
                }
            };
            // TODO: re-use the validator
            Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            )
            .validate(&module.0)
            .unwrap();
        }
    }
}
