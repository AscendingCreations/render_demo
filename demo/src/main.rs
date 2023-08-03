#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use backtrace::Backtrace;
use camera::{
    controls::{Controls, FlatControls, FlatSettings},
    Projection,
};
use cosmic_text::{
    Action as TextAction, Attrs, Buffer, FontSystem, Metrics, Style, SwashCache,
};
use graphics::*;
use hecs::World;
use input::{Bindings, FrameTime, InputHandler};
use log::{error, info, warn, Level, LevelFilter, Metadata, Record};
use naga::{front::wgsl, valid::Validator};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{prelude::*, Read, Write},
    iter, panic,
    path::PathBuf,
    rc::Rc,
    time::Duration,
};
use wgpu::InstanceDescriptor;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use graphics::iced_wgpu::{Backend, Renderer, Settings};
use graphics::iced_winit::{
    conversion,
    core::{mouse, renderer, Color as iced_color, Size},
    futures,
    runtime::{program, Debug},
    style::Theme,
    winit, Clipboard,
};

mod gamestate;
mod ui;

use gamestate::*;

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
        .with_visible(false)
        .build(&event_loop)
        .unwrap();
    let instance = wgpu::Instance::default();
    let font_system = FontSystem::new();

    let (gpu_window, gpu_device) = instance
        .create_device(
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

    let mut renderer = GpuRenderer::new(gpu_window, gpu_device, font_system);
    renderer.create_pipelines(renderer.surface_format());

    println!("{:?}", renderer.adapter().get_info());

    let mut atlases: Vec<AtlasGroup> = iter::from_fn(|| {
        Some(AtlasGroup::new(
            &mut renderer,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ))
    })
    .take(4)
    .collect();

    let text_atlas = TextAtlas::new(&mut renderer).unwrap();
    let allocation = Texture::from_file("images/Female_1.png")?
        .group_upload(&mut atlases[0], &renderer)
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let mut sprites = Vec::with_capacity(2001);

    let mut x = 0.0;
    let y = 0.0;

    for _i in 0..2 {
        let mut sprite = Image::new(Some(allocation), &mut renderer, 1);
        sprite.pos = Vec3::new(x, y, 5.1);
        sprite.hw = Vec2::new(48.0, 48.0);
        sprite.uv = Vec4::new(48.0, 96.0, 48.0, 48.0);
        sprite.color = Color::rgba(255, 255, 255, 255);
        sprites.push(sprite);
        x += 12.0;
    }

    sprites[0].pos.z = 5.0;
    sprites[0].color = Color::rgba(255, 255, 255, 120);

    let text_renderer = TextRenderer::new(&renderer).unwrap();
    let sprite_renderer = ImageRenderer::new(&renderer).unwrap();
    let mut map_renderer = MapRenderer::new(&mut renderer, 81).unwrap();
    let mesh_renderer = Mesh2DRenderer::new(&renderer).unwrap();

    let mut size = renderer.size();

    let system = System::new(
        &mut renderer,
        Projection::Orthographic {
            left: 0.0,
            right: size.width,
            bottom: 0.0,
            top: size.height,
            near: 1.0,
            far: -100.0,
        },
        FlatControls::new(FlatSettings::default()),
        [size.width, size.height],
    );

    let mut map = Map::new(&mut renderer);

    (0..32).for_each(|x| {
        (0..32).for_each(|y| {
            map.set_tile((x, y, 0), 1, 0, 255);
        });
    });

    map.set_tile((1, 31, 1), 2, 0, 255);
    map.set_tile((1, 30, 6), 2, 0, 180);
    map.set_tile((0, 0, 1), 2, 0, 255);
    map.pos = Vec2::new(0.0, 0.0);

    for i in 0..3 {
        let _ = Texture::from_file(format!("images/tiles/{i}.png"))?
            .group_upload(&mut atlases[1], &renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
    }

    map.layer = map_renderer
        .get_unused_id()
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let allocation = Texture::from_file("images/anim/0.png")?
        .group_upload(&mut atlases[0], &renderer)
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let mut animation = Image::new(Some(allocation), &mut renderer, 2);

    animation.pos = Vec3::new(96.0, 96.0, 5.0);
    animation.hw = Vec2::new(64.0, 64.0);
    animation.uv = Vec4::new(0.0, 0.0, 64.0, 64.0);
    animation.color = Color::rgba(255, 255, 255, 255);
    animation.frames = Vec2::new(8.0, 4.0);
    animation.switch_time = 300;
    animation.animate = true;

    let scale = renderer.window().current_monitor().unwrap().scale_factor();

    let mut text = Text::new(
        &mut renderer,
        Some(Metrics::new(16.0, 16.0).scale(scale as f32)),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::new(190.0, 32.0),
    );

    text.set_buffer_size(&mut renderer, size.width as i32, size.height as i32)
        .set_bounds(Some(WorldBounds::new(0.0, 0.0, 190.0, 32.0, 1.0)));

    let mut builder = Mesh2DBuilder::new();

    builder
        .circle(
            DrawMode::Fill(FillOptions::DEFAULT),
            Vec2::new(100.0, 100.0),
            60.0,
            0.5,
            1.0,
            Color::rgba(0, 0, 255, 255),
        )
        .unwrap();
    builder
        .circle(
            DrawMode::Stroke(StrokeOptions::DEFAULT),
            Vec2::new(100.0, 100.0),
            60.0,
            0.5,
            1.0,
            Color::rgba(255, 255, 255, 255),
        )
        .unwrap();

    let mut builder2 = Mesh2DBuilder::new();

    builder2
        .circle(
            DrawMode::Fill(FillOptions::DEFAULT),
            Vec2::new(200.0, 200.0),
            60.0,
            0.5,
            1.0,
            Color::rgba(0, 0, 255, 255),
        )
        .unwrap();
    builder2
        .circle(
            DrawMode::Stroke(StrokeOptions::DEFAULT),
            Vec2::new(200.0, 200.0),
            60.0,
            0.5,
            1.0,
            Color::rgba(255, 255, 255, 255),
        )
        .unwrap();
    builder2
        .polyline(
            DrawMode::Stroke(StrokeOptions::DEFAULT),
            &[Vec2::new(200.0, 200.0), Vec2::new(400.0, 400.0)],
            1.0,
            Color::rgba(255, 255, 255, 255),
        )
        .unwrap();
    let mut mesh = [Mesh2D::new(&mut renderer), Mesh2D::new(&mut renderer)];
    mesh[0].from_builder(builder.finalize());
    mesh[1].from_builder(builder2.finalize());

    let mut debug = Debug::new();
    let mut iced_renderer = Renderer::new(Backend::new(
        renderer.device(),
        renderer.queue(),
        Settings::default(),
        renderer.surface_format(),
    ));

    let iced_controls = ui::Controls::new();
    let mut iced_state = program::State::new(
        iced_controls,
        system.iced_view().logical_size(),
        &mut iced_renderer,
        &mut debug,
    );
    renderer.window().set_visible(true);

    let mut state = State {
        system,
        sprites,
        animation,
        image_atlas: atlases.remove(0),
        map,
        map_renderer,
        map_atlas: atlases.remove(0),
        sprite_renderer,
        text_atlas,
        text_renderer,
        mesh,
        mesh_atlas: atlases.remove(0),
        mesh_renderer,
    };

    let mut bindings = Bindings::<Action, Axis>::new();
    bindings.insert_action(
        Action::Quit,
        vec![winit::event::VirtualKeyCode::Q.into()],
    );
    let mut input_handler = InputHandler::new(bindings);

    let mut frame_time = FrameTime::new();
    let mut time = 0.0f32;
    let mut fps = 0u32;

    //let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(renderer.window());
    //let mut mouse_pos = Vec2::default();

    let mut debug = Debug::new();

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
            Event::MainEventsCleared => {
                if !iced_state.is_queue_empty() {
                    // We update iced
                    let _ = iced_state.update(
                        state.system.iced_view().logical_size(),
                        input_handler
                            .physical_mouse_position()
                            .map(|p| {
                                conversion::cursor_position(
                                    p,
                                    state.system.iced_view().scale_factor(),
                                )
                            })
                            .map(mouse::Cursor::Available)
                            .unwrap_or(mouse::Cursor::Unavailable),
                        &mut iced_renderer,
                        &Theme::Dark,
                        &renderer::Style {
                            text_color: iced_color::WHITE,
                        },
                        &mut clipboard,
                        &mut debug,
                    );

                    // and request a redraw
                    renderer.window().request_redraw();
                    return;
                }
            }
            _ => {}
        }

        let new_size = renderer.size();
        let inner_size = renderer.window().inner_size();

        if new_size.width == 0.0
            || new_size.height == 0.0
            || inner_size.width == 0
            || inner_size.height == 0
        {
            return;
        }

        input_handler.update(renderer.window(), &event, 1.0);

        if let Event::WindowEvent { ref event, .. } = &event {
            if let Some(event) = graphics::iced_winit::conversion::window_event(
                event,
                renderer.window().scale_factor(),
                input_handler.modifiers(),
            ) {
                iced_state.queue_event(event);
            }
        }

        if !renderer.update(&event).unwrap() {
            return;
        }

        if size != new_size {
            size = new_size;

            state.system.set_projection(Projection::Orthographic {
                left: 0.0,
                right: new_size.width,
                bottom: 0.0,
                top: new_size.height,
                near: 1.0,
                far: -100.0,
            });

            renderer.update_depth_texture();
        }

        if input_handler.is_action_down(&Action::Quit) {
            *control_flow = ControlFlow::Exit;
        }

        let seconds = frame_time.seconds();
        state.system.update(&renderer, &frame_time);
        state
            .system
            .update_screen(&renderer, [new_size.width, new_size.height]);

        state.sprites.iter_mut().for_each(|sprite| {
            state.sprite_renderer.image_update(sprite, &mut renderer);
        });
        state
            .sprite_renderer
            .image_update(&mut state.animation, &mut renderer);
        state.sprite_renderer.finalize(&mut renderer);
        state
            .text_renderer
            .text_update(&mut text, &mut state.text_atlas, &mut renderer)
            .unwrap();
        state.text_renderer.finalize(&mut renderer);
        state.map_renderer.map_update(&mut state.map, &mut renderer);
        state.map_renderer.finalize(&mut renderer);
        state.mesh.iter_mut().for_each(|mesh| {
            state.mesh_renderer.mesh_update(mesh, &mut renderer);
        });

        state.mesh_renderer.finalize(&mut renderer);

        // Start encoding commands.
        let mut encoder = renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            },
        );

        // Run the render pass. for the games renderer
        state.render(&renderer, &mut encoder);

        // Run the render pass for iced GUI renderer.
        iced_renderer.with_primitives(|backend, primitive| {
            backend.present(
                renderer.device(),
                renderer.queue(),
                &mut encoder,
                None,
                renderer.frame_buffer().as_ref().expect("no frame view?"),
                primitive,
                state.system.iced_view(),
                &debug.overlay(),
            );
        });

        // Submit our command queue.
        renderer.queue().submit(std::iter::once(encoder.finish()));

        if time < seconds {
            text.set_text(
                &mut renderer,
                &format!("生活,삶,जिंदगी 😀 FPS: {fps} \nhello"),
                cosmic_text::Attrs::new(),
            );
            fps = 0u32;
            time = seconds + 1.0;
        }

        fps += 1;

        input_handler.end_frame();
        frame_time.update();
        renderer.present().unwrap();

        renderer.window_mut().set_cursor_icon(
            iced_winit::conversion::mouse_interaction(
                iced_state.mouse_interaction(),
            ),
        );

        state.image_atlas.trim();
        state.map_atlas.trim();
        state.text_atlas.trim();
    })
}
