#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use backtrace::Backtrace;
use camera::{
    controls::{FlatControls, FlatSettings},
    Projection,
};
use cosmic_text::{
    Action as TextAction, Buffer, FontSystem, Metrics, Style, SwashCache,
};
use input::{Bindings, FrameTime, InputHandler};
use log::{error, info, warn, Level, LevelFilter, Metadata, Record};
use naga::{front::wgsl, valid::Validator};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{prelude::*, Read, Write},
    panic,
    path::PathBuf,
};
use wgpu_profiler::{wgpu_profiler, GpuProfiler, GpuTimerScopeResult};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod error;
mod gamestate;
mod graphics;

use error::*;
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
static FONT_SYSTEM: Lazy<FontSystem> = Lazy::new(FontSystem::new);

struct MyLogger(pub Level);

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.0
    }

    //This logs to a panic file
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
async fn main() -> Result<(), AscendingError> {
    log::set_logger(&MY_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    info!("starting up");

    panic::set_hook(Box::new(|panic_info| {
        let bt = Backtrace::new();

        error!("PANIC: {}, BACKTRACE: {:?}", panic_info, bt);
    }));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Demo")
        .with_inner_size(PhysicalSize::new(800, 600))
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
                features: wgpu::Features::default(),
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

    let mut sprites = Vec::with_capacity(2001);

    let mut x = 0;
    let mut y = 0;

    for i in 0..2 {
        if i % 50 == 0 {
            y += 12;
            x = 0;
        }

        let mut sprite = Sprite::new(allocation);
        sprite.pos = [x, y, 5];
        sprite.hw = [48, 48];
        sprite.uv = [48, 96, 48, 48];
        sprite.color = Color::rgba(255, 255, 255, 255);
        sprites.push(sprite);
        x += 12;
    }

    let sprite_pipeline = SpriteRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let mut size = renderer.size();

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
        [size.width as f32, size.height as f32],
    );

    let sprite_buffer = InstanceBuffer::with_capacity(renderer.device(), 1);

    let mut map = Map::new();

    (0..32).for_each(|x| {
        (0..32).for_each(|y| {
            map.set_tile((x, y, 0), 1, 0, 255);
        });
    });

    map.set_tile((1, 31, 1), 2, 0, 255);
    map.set_tile((1, 30, 6), 2, 0, 180);
    map.set_tile((0, 0, 1), 2, 0, 255);
    map.pos = [0, 0];
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
        let _ = Texture::from_file(format!("images/tiles/{i}.png"))?
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

    let maplower_buffer = InstanceBuffer::with_capacity(renderer.device(), 540);
    let mapupper_buffer = InstanceBuffer::with_capacity(renderer.device(), 180);

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

    let animation_buffer = InstanceBuffer::new(renderer.device());

    let mut animation = Sprite::new(allocation);

    animation.pos = [96, 96, 5];
    animation.hw = [64, 64];
    animation.uv = [0, 0, 64, 64];
    animation.color = Color::rgba(255, 255, 255, 255);
    animation.frames = [8, 4];
    animation.switch_time = 300;
    animation.animate = true;

    let shapes_pipeline = ShapeRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let shapes_buffer = InstanceBuffer::new(renderer.device());

    let mut shapes = Shapes::new();

    shapes.push_shape(Shape::Rect {
        position: [150, 150, 1],
        size: [100, 100],
        border_width: 1,
        border_color: Color::rgba(255, 255, 255, 255),
        color: Color::rgba(255, 0, 0, 255),
        radius: 10.0,
    });

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
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &mut layout_storage,
        GroupType::Textures,
    );

    let text_pipeline = TextRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;
    let text_buffer = InstanceBuffer::new(renderer.device());

    let text = Text::new(&FONT_SYSTEM, None);

    let scale = renderer.window().current_monitor().unwrap().scale_factor();

    let mut textbuffer =
        Buffer::new(&FONT_SYSTEM, Metrics::new(16, 24).scale(scale as i32));

    textbuffer.set_size(size.width as i32, size.height as i32);

    let buffer_object = StaticBufferObject::new(renderer.device());

    let mut state = State {
        layout_storage,
        system,
        sprites,
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
        buffer_object,
    };

    let mut views = HashMap::new();

    views.insert("depthbuffer".to_string(), renderer.create_depth_texture());

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

        let new_size = renderer.size();
        let inner_size = renderer.window().inner_size();

        if new_size.width == 0
            || new_size.height == 0
            || inner_size.width == 0
            || inner_size.height == 0
        {
            return;
        }

        input_handler.update(renderer.window(), &event, 1.0);

        let frame = match renderer.update(&event).unwrap() {
            Some(frame) => frame,
            _ => return,
        };

        if size != new_size {
            size = new_size;

            state.system.set_projection(Projection::Orthographic {
                left: 0.0,
                right: new_size.width as f32,
                bottom: 0.0,
                top: new_size.height as f32,
                near: 1.0,
                far: -100.0,
            });

            views.insert(
                "depthbuffer".to_string(),
                renderer.create_depth_texture(),
            );
        }

        if input_handler.is_action_down(&Action::Quit) {
            *control_flow = ControlFlow::Exit;
        }

        let seconds = frame_time.seconds();
        state.system.update(&renderer, &frame_time);
        state.system.update_screen(
            &renderer,
            [new_size.width as f32, new_size.height as f32],
        );

        views.insert(
            "framebuffer".to_string(),
            frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        let mut update = false;

        for sprite in &mut state.sprites {
            update = sprite.update() || update;
        }

        if update {
            let mut bytes = Vec::with_capacity(state.sprites.len() * 4);

            for sprite in &state.sprites {
                bytes.extend_from_slice(&sprite.bytes);
            }

            state.sprite_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &bytes,
            );
        }

        let update = state.text.update(
            renderer.queue(),
            renderer.device(),
            [0, 0, 1],
            &mut textbuffer,
            &mut state.text_atlas,
            &mut state.emoji_atlas,
        );

        state.text.reset_cleared();

        if update {
            state.text_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.text.text_bytes,
            );
        }

        let update =
            state.map.update(renderer.queue(), &mut state.map_textures);

        if update {
            state.maplower_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.map.lowerbytes,
            );

            state.mapupper_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.map.upperbytes,
            );
        }

        let update = state.animation.update();

        if update {
            state.animation_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.animation.bytes,
            );
        }

        let update = state.shapes.update();

        if update {
            state.shapes_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.shapes.buffers,
            );
        }

        // Start encoding commands.
        let mut encoder = renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            },
        );

        // Run the render pass.
        state.render(&mut encoder, &views, &renderer);

        // Submit our command queue.
        renderer.queue().submit(std::iter::once(encoder.finish()));

        if time < seconds {
            textbuffer.set_text(
                &format!("生活,삶,जिंदगी 😀 FPS: {fps}"),
                cosmic_text::Attrs::new(),
            );
            textbuffer.set_redraw(true);
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
