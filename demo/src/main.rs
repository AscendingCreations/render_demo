#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use ::camera::{
    controls::{FlatControls, FlatSettings},
    Projection,
};
use backtrace::Backtrace;
use cosmic_text::{
    FontSystem, Style, SwashCache, TextAction, TextBuffer, TextMetrics,
};
use fontdue::{Font, FontSettings};
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
static FONT_SYSTEM: Lazy<FontSystem<'static>> = Lazy::new(FontSystem::new);

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
            wgpu::PresentMode::AutoVsync,
        )
        .await
        .unwrap();

    println!("{:?}", renderer.adapter().get_info());
    let mut layout_storage = LayoutStorage::new();

    let mut sprite_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &mut layout_storage,
        GroupType::Textures,
    );

    let allocation = Texture::from_file("images/Female_1.png")?
        .group_upload(&mut sprite_atlas, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;
    let mut sprite = [Sprite::new(allocation), Sprite::new(allocation)];

    sprite[0].pos = [32, 32, 5];
    sprite[0].hw = [48, 48];
    sprite[0].uv = [48, 96, 48, 48];
    sprite[0].color = [255, 255, 255, 255];

    sprite[1].pos = [64, 32, 6];
    sprite[1].hw = [48, 48];
    sprite[1].uv = [48, 96, 48, 48];
    sprite[1].color = [100, 100, 100, 255];

    let sprite_pipeline = SpriteRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let size = renderer.size();

    let system = System::new(
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
        FlatControls::new(FlatSettings::default()),
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

    let mut map_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &mut layout_storage,
        GroupType::Textures,
    );

    for i in 0..3 {
        let _ = Texture::from_file(format!("images/tiles/{}.png", i))?
            .group_upload(&mut map_atlas, renderer.device(), renderer.queue())
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
    }

    let mut map_textures = MapTextures::new(renderer.device(), 81);
    let map_group = TextureGroup::from_view(
        renderer.device(),
        &mut layout_storage,
        &map_textures.texture_view,
        MapLayout,
        GroupType::Textures,
    );

    let maplower_buffer = GpuBuffer::with_capacity(renderer.device(), 540);
    let mapupper_buffer = GpuBuffer::with_capacity(renderer.device(), 180);

    map.layer = map_textures
        .get_unused_id()
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let mut animation_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &mut layout_storage,
        GroupType::Textures,
    );

    let allocation = Texture::from_file("images/anim/0.png")?
        .group_upload(&mut animation_atlas, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let animation_buffer = GpuBuffer::new(renderer.device());

    let mut animation = Sprite::new(allocation);

    animation.pos = [96, 96, 5];
    animation.hw = [64, 64];
    animation.uv = [0, 0, 64, 64];
    animation.color = [255, 255, 255, 255];
    animation.frames = [8, 4];
    animation.switch_time = 300;
    animation.animate = true;

    let text_colored_group =
        TextColoredGroup::new(&renderer, &mut layout_storage);
    let screen_group = ScreenGroup::new(
        &renderer,
        &mut layout_storage,
        ScreenUniform {
            width: size.width,
            height: size.height,
        },
    );
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

    let text_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::R8Unorm,
        &mut layout_storage,
        GroupType::Fonts,
    );

    let emoji_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::R8Unorm,
        &mut layout_storage,
        GroupType::Textures,
    );

    let text_pipeline = TextRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;
    let text_buffer = GpuBuffer::new(renderer.device());
    let emoji_buffer = GpuBuffer::new(renderer.device());

    let text = Text::new(&FONT_SYSTEM);
    let attr = cosmic_text::Attrs::new().style(Style::Italic);
    let mut textbuffer = TextBuffer::new(
        &FONT_SYSTEM,
        attr,
        TextMetrics::new(14, 20)
            .scale(((renderer.size().height / 1600) + 1) as i32),
    );

    textbuffer
        .set_size(renderer.size().width as i32, renderer.size().height as i32);

    let mut state = State {
        layout_storage,
        system,
        text_colored_group,
        screen_group,
        sprite,
        sprite_pipeline,
        sprite_buffer,
        sprite_atlas,
        map,
        map_pipeline,
        maplower_buffer,
        mapupper_buffer,
        map_group,
        map_atlas,
        map_textures,
        animation,
        animation_buffer,
        animation_atlas,
        shapes,
        shapes_buffer,
        shapes_pipeline,
        text,
        text_atlas,
        emoji_atlas,
        text_pipeline,
        text_buffer,
        emoji_buffer,
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
    let mut time = 0.0f32;
    let mut fps = 0u32;

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

            state.screen_group.update(
                &renderer,
                ScreenUniform {
                    width: test_size.width,
                    height: test_size.height,
                },
            );

            state.system.set_projection(Projection::Orthographic {
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

        let seconds = frame_time.seconds();
        state.system.update(&renderer, &frame_time);

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        views.insert("framebuffer".to_string(), view);

        let update = state.sprite[0].update();
        let update = state.sprite[1].update() || update;

        if update {
            let mut bytes = state.sprite[0].bytes.clone();
            bytes.extend_from_slice(&state.sprite[1].bytes);

            state.sprite_buffer.set_vertices_from(
                renderer.device(),
                renderer.queue(),
                &bytes,
            );
        }

        let update = state.text.update(
            renderer.queue(),
            renderer.device(),
            [100, 100, 1],
            &mut textbuffer,
            &mut state.text_atlas,
            &mut state.emoji_atlas,
        );

        if update {
            state.text_buffer.set_vertices_from(
                renderer.device(),
                renderer.queue(),
                &state.text.text_bytes,
            );
            state.emoji_buffer.set_vertices_from(
                renderer.device(),
                renderer.queue(),
                &state.text.emoji_bytes,
            );
        }

        let update =
            state.map.update(renderer.queue(), &mut state.map_textures);

        if update {
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
        }

        let update = state.animation.update();

        if update {
            state.animation_buffer.set_vertices_from(
                renderer.device(),
                renderer.queue(),
                &state.animation.bytes,
            );
        }

        let update = state.shapes.update();

        if update {
            state.shapes_buffer.set_vertices_from(
                renderer.device(),
                renderer.queue(),
                &state.shapes.buffers.vertices,
            );

            state.shapes_buffer.set_indices_from(
                renderer.queue(),
                &state.shapes.buffers.indices,
            );
        }

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

        if time < seconds {
            textbuffer.set_text(&format!("FPS: {}", fps));
            textbuffer.redraw = true;
            fps = 0u32;
            time = seconds + 1.0;
        }

        fps += 1;

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
