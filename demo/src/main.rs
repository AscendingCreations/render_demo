#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use backtrace::Backtrace;
use camera::{
    controls::{FlatControls, FlatSettings},
    Projection,
};
use cosmic_text::{
    Action as TextAction, Buffer, FontSystem, Metrics, Style, SwashCache,
};
use graphics::*;
use input::{Bindings, FrameTime, InputHandler};
use log::{error, info, warn, Level, LevelFilter, Metadata, Record};
use naga::{front::wgsl, valid::Validator};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{prelude::*, Read, Write},
    panic,
    path::PathBuf,
    rc::Rc,
    time::Duration,
};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
mod gamestate;
mod gui;

use gamestate::*;
use gui::*;

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
    let instance = wgpu::Instance::default();

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
        256,
        256,
    );

    let allocation = Texture::from_file("images/Female_1.png")?
        .group_upload(&mut sprite_atlas, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let mut sprites = Vec::with_capacity(2001);

    let mut x = 0.0;
    let y = 0.0;

    for _i in 0..2 {
        let mut sprite = Image::new(allocation);
        sprite.pos = Vec3::new(x, y, 5.0);
        sprite.hw = Vec2::new(48.0, 48.0);
        sprite.uv = Vec4::new(48.0, 96.0, 48.0, 48.0);
        sprite.color = Color::rgba(255, 255, 255, 255);
        sprites.push(sprite);
        x += 12.0;
    }

    let sprite_pipeline = ImageRenderPipeline::new(
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
        256,
        256,
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
        256,
        256,
    );

    let allocation = Texture::from_file("images/anim/0.png")?
        .group_upload(&mut animation_atlas, renderer.device(), renderer.queue())
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let animation_buffer = InstanceBuffer::new(renderer.device());

    let mut animation = Image::new(allocation);

    animation.pos = Vec3::new(96.0, 96.0, 5.0);
    animation.hw = Vec2::new(64.0, 64.0);
    animation.uv = Vec4::new(0.0, 0.0, 64.0, 64.0);
    animation.color = Color::rgba(255, 255, 255, 255);
    animation.frames = Vec2::new(8.0, 4.0);
    animation.switch_time = 300;
    animation.animate = true;

    let rects_pipeline = RectsRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;

    let rects_buffer = InstanceBuffer::new(renderer.device());

    let mut rects_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &mut layout_storage,
        GroupType::Textures,
        256,
        256,
    );

    let mut rects = Rect {
        position: Vec3::new(150.0, 150.0, 1.0),
        size: Vec2::new(132.0, 32.0),
        border_width: 2,
        radius: Some(5.0),
        changed: true,
        ..Default::default()
    };

    rects
        .set_color(
            renderer.device(),
            renderer.queue(),
            &mut rects_atlas,
            Color::rgba(255, 255, 0, 255),
        )
        .set_border_color(
            renderer.device(),
            renderer.queue(),
            &mut rects_atlas,
            Color::rgba(0, 0, 0, 255),
        )
        .set_container_uv(Vec4::new(0.0, 0.0, 168.0, 32.0));

    let text_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::R8Unorm,
        &mut layout_storage,
        GroupType::Fonts,
        2,
        256,
    );

    let emoji_atlas = AtlasGroup::new(
        renderer.device(),
        2048,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &mut layout_storage,
        GroupType::Textures,
        2,
        256,
    );

    let text_pipeline = TextRenderPipeline::new(
        renderer.device(),
        renderer.surface_format(),
        &mut layout_storage,
    )?;
    let text_buffer = InstanceBuffer::new(renderer.device());
    let mut font_cache: SwashCache<'static> = SwashCache::new(&FONT_SYSTEM);
    let text_render = TextRender::new();
    let scale = renderer.window().current_monitor().unwrap().scale_factor();

    let mut text = Text::new(
        &FONT_SYSTEM,
        Some(Metrics::new(16, 16).scale(scale as i32)),
        Vec3::new(0.0, 32.0, 1.0),
        Vec2::new(256.0, 256.0),
        Some(TextBounds::new(8.0, 32.0, 190.0, 0.0)),
    );

    text.set_buffer_size(size.width as i32, size.height as i32);

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
        rects,
        rects_buffer,
        rects_pipeline,
        rects_atlas,
        text_render,
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
    let mut mouse_pos: [i32; 2] = [0; 2];
    let mut id = 0;

    #[allow(deprecated)]
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

        if state.rects.check_mouse_bounds(mouse_pos) {
            println!("Within the Shape: {id}");
            id += 1;
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

        mouse_pos = {
            let pos = input_handler.mouse_position().unwrap_or((0.0, 0.0));
            [pos.0 as i32, size.height as i32 - pos.1 as i32]
        };

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
                &[],
            );
        }

        let update = text
            .update(
                &mut font_cache,
                &mut state.text_atlas,
                &mut state.emoji_atlas,
                renderer.queue(),
                renderer.device(),
                &state.system,
            )
            .unwrap();

        if update {
            state.text_render.clear();
            state.text_render.push(&text);
            state.text_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.text_render.text_bytes,
                &[],
            );
        }

        let update =
            state.map.update(renderer.queue(), &mut state.map_textures);

        if update {
            state.maplower_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.map.lowerbytes,
                &[],
            );

            state.mapupper_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.map.upperbytes,
                &[],
            );
        }

        let update = state.animation.update();

        if update {
            state.animation_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.animation.bytes,
                &[],
            );
        }

        let update = state.rects.update();

        if update {
            state.rects_buffer.set_from(
                renderer.device(),
                renderer.queue(),
                &state.rects.bytes,
                &[Some(Bounds::new(150.0, 150.0, 132.0, 32.0, 32.0))],
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
            text.set_text(
                &format!("生活,삶,जिंदगी 😀 FPS: {fps} \nhello"),
                cosmic_text::Attrs::new(),
            );
            fps = 0u32;
            time = seconds + 1.0;
        }

        fps += 1;

        views.remove("framebuffer");

        input_handler.end_frame();
        frame_time.update();
        frame.present();

        state.animation_atlas.clean();
        state.rects_atlas.clean();
        state.map_atlas.clean();
        state.sprite_atlas.clean();
        state.text_atlas.clean();
        state.emoji_atlas.clean();
    })
}
