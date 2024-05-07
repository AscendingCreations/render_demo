#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use backtrace::Backtrace;
use camera::{
    controls::{Controls, FlatControls, FlatSettings},
    Projection,
};
use graphics::naga::{front::wgsl, valid::Validator};
use graphics::*;
use graphics::{
    cosmic_text::{Attrs, Metrics},
    wgpu::PowerPreference,
};
use hecs::World;
use input::{Bindings, FrameTime, InputHandler, Key};
use log::{error, info, warn, Level, LevelFilter, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::env;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, File},
    io::{prelude::*, Read, Write},
    iter, panic,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::{Backends, Dx12Compiler, InstanceDescriptor, InstanceFlags};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{WindowAttributes, WindowButtons},
};

mod gamestate;
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

// creates a static global logger type for setting the logger
static MY_LOGGER: MyLogger = MyLogger(Level::Debug);

struct MyLogger(pub Level);

impl log::Log for MyLogger {
    // checks if it can log these types of events.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.0
    }

    // This logs to a panic file. This is so we can see
    // Errors and such if a program crashes in full render mode.
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

enum Runner {
    Loading,
    Ready {
        input_handler: InputHandler<Action, Axis>,
        renderer: GpuRenderer,
        state: State<FlatControls>,
        frame_time: FrameTime,
        time: f32,
        fps: u32,
        text: Text,
        size: PhysicalSize<f32>,
    },
}

impl winit::application::ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Loading = self {
            info!("loading initiation");
            let win_attrs = WindowAttributes::default()
                .with_active(false)
                .with_visible(false)
                .with_inner_size(PhysicalSize::new(800, 600))
                .with_title("Demo")
                .with_enabled_buttons({
                    let mut buttons = WindowButtons::all();
                    buttons.remove(WindowButtons::MAXIMIZE);
                    buttons
                });

            // Builds the Windows that will be rendered too.
            let window = Arc::new(
                event_loop.create_window(win_attrs).expect("Create window"),
            );

            info!("after window initiation");
            // Generates an Instance for WGPU. Sets WGPU to be allowed on all possible supported backends
            // These are DX12, DX11, Vulkan, Metal and Gles. if none of these work on a system they cant
            // play the game basically.
            let instance = wgpu::Instance::new(InstanceDescriptor {
                backends: Backends::all(),
                flags: InstanceFlags::empty(),
                dx12_shader_compiler: Dx12Compiler::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            });

            info!("after wgpu instance initiation");

            // This is used to ensure the GPU can load the correct.
            let compatible_surface =
                instance.create_surface(window.clone()).unwrap();

            info!("after compatible initiation");
            print!("{:?}", &compatible_surface);
            // This creates the Window Struct and Device struct that holds all the rendering information
            // we need to render to the screen. Window holds most of the window information including
            // the surface type. device includes the queue and GPU device for rendering.
            // This then adds gpu_window and gpu_device and creates our renderer type. for easy passing of window, device and font system.
            let mut renderer =
                futures::executor::block_on(instance.create_device(
                    window,
                    //used to find adapters
                    AdapterOptions {
                        allowed_backends: Backends::all(),
                        power: AdapterPowerSettings::HighPower,
                        compatible_surface: Some(compatible_surface),
                    },
                    // used to deturmine which adapters support our special limits or features for our backends.
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None,
                    // How we are presenting the screen which causes it to either clip to a FPS limit or be unlimited.
                    wgpu::PresentMode::AutoVsync,
                ))
                .unwrap();

            info!("after renderer initiation");
            // we print the GPU it decided to use here for testing purposes.
            println!("{:?}", renderer.adapter().get_info());

            // We generate Texture atlases to use with out types.
            let mut atlases: Vec<AtlasSet> = iter::from_fn(|| {
                Some(AtlasSet::new(
                    &mut renderer,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    true,
                ))
            })
            .take(4)
            .collect();

            // we generate the Text atlas seperatly since it contains a special texture that only has the red color to it.
            // and another for emojicons.
            let text_atlas = TextAtlas::new(&mut renderer).unwrap();

            // This is how we load a image into a atlas/Texture. It returns the location of the image
            // within the texture. its x, y, w, h.  Texture loads the file. group_uploads sends it to the Texture
            // renderer is used to upload it to the GPU when done.
            let allocation = Texture::from_file("images/Female_1.png")
                .unwrap()
                .upload(&mut atlases[0], &renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))
                .unwrap();

            let mut sprites = Vec::with_capacity(2001);

            let mut x = 0.0;
            let y = 0.0;

            for _i in 0..2 {
                // I named this image simply because it can do a lot of different animations etc, but technically
                // Image is sprite and I am thinking of renaming this to make it easier for you and others.
                // Image is mostly the backend render type used to render it to the screen. Im unsure though how
                // To name this atm to keep it seperated from Sprite that would contain most of the actual not rendering
                // data needed.
                let mut sprite = Image::new(Some(allocation), &mut renderer, 1);
                sprite.pos = Vec3::new(x, y, 7.0);
                sprite.hw = Vec2::new(48.0, 48.0);
                sprite.uv = Vec4::new(48.0, 96.0, 48.0, 48.0);
                sprite.color = Color::rgba(255, 255, 255, 255);
                sprites.push(sprite);
                x += 12.0;
            }

            sprites[0].pos.z = 7.0;
            sprites[0].color = Color::rgba(255, 255, 255, 120);

            // We establish the different renderers here to load their data up to use them.
            let text_renderer = TextRenderer::new(&renderer).unwrap();
            let sprite_renderer = ImageRenderer::new(&renderer).unwrap();
            let map_renderer = MapRenderer::new(&mut renderer, 81).unwrap();
            let mesh_renderer = Mesh2DRenderer::new(&renderer).unwrap();
            let light_renderer = LightRenderer::new(&mut renderer).unwrap();
            let ui_renderer = RectRenderer::new(&renderer).unwrap();

            // get the screen size.
            let size = renderer.size();
            let mat = Mat4::from_translation(Vec3 {
                x: 40.0,
                y: 0.0,
                z: 0.0,
            });

            // setup our system which includes Camera and projection as well as our controls.
            // for the camera.
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
                FlatControls::new(FlatSettings { zoom: 1.5 }),
                [size.width, size.height],
                mat,
                1.5,
            );

            // We make a new Map to render here.
            let mut map = Map::new(&mut renderer, 20);

            (0..32).for_each(|x| {
                (0..32).for_each(|y| {
                    map.set_tile(
                        (x, y, 0),
                        TileData {
                            id: 1,
                            color: Color::rgba(255, 255, 255, 255),
                        },
                    )
                });
            });

            map.set_tile(
                (2, 1, 1),
                TileData {
                    id: 2,
                    color: Color::rgba(255, 255, 255, 255),
                },
            );
            map.set_tile(
                (1, 1, 6),
                TileData {
                    id: 2,
                    color: Color::rgba(255, 255, 255, 230),
                },
            );
            map.set_tile(
                (0, 0, 1),
                TileData {
                    id: 2,
                    color: Color::rgba(255, 255, 255, 255),
                },
            );
            map.pos = Vec2::new(0.0, 0.0);
            map.can_render = true;

            let _tilesheet = Texture::from_file("images/tiles/1.png")
                .unwrap()
                .new_tilesheet(&mut atlases[1], &renderer, 20)
                .ok_or_else(|| OtherError::new("failed to upload tiles"))
                .unwrap();

            //println!("tilesheet: {:?}", tilesheet);

            let allocation = Texture::from_file("images/anim/0.png")
                .unwrap()
                .upload(&mut atlases[0], &renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))
                .unwrap();

            let mut animation = Image::new(Some(allocation), &mut renderer, 2);

            animation.pos = Vec3::new(96.0, 96.0, 5.0);
            animation.hw = Vec2::new(64.0, 64.0);
            animation.uv = Vec4::new(0.0, 0.0, 64.0, 64.0);
            animation.color = Color::rgba(255, 255, 255, 255);
            animation.frames = Vec2::new(8.0, 4.0);
            animation.switch_time = 300;
            animation.animate = true;

            // get the Scale factor the pc currently is using for upscaling or downscaling the rendering.
            let scale = 1.0; //renderer.window().current_monitor().unwrap().scale_factor();

            // create a Text rendering object.
            let mut text = Text::new(
                &mut renderer,
                Some(Metrics::new(16.0, 16.0).scale(scale as f32)),
                Vec3::new(0.0, 0.0, 1.0),
                Vec2::new(190.0, 32.0),
                1.0,
            );

            text.set_buffer_size(
                &mut renderer,
                size.width as i32,
                size.height as i32,
            )
            .set_bounds(Some(Bounds::new(0.0, 0.0, 250.0, 600.0)))
            .set_default_color(Color::rgba(255, 255, 255, 255));

            // Start the process of building a shape.
            let mut builder = Mesh2DBuilder::default();

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

            let mut builder2 = Mesh2DBuilder::default();

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
            let mut mesh =
                [Mesh2D::new(&mut renderer), Mesh2D::new(&mut renderer)];
            mesh[0].from_builder(builder.finalize());
            mesh[1].from_builder(builder2.finalize());

            let mut lights = Lights::new(&mut renderer, 0, 1.0);

            lights.world_color = Vec4::new(0.0, 0.0, 0.0, 0.995);
            lights.enable_lights = true;

            lights.insert_area_light(AreaLight {
                pos: Vec2::new(24.0, 24.0),
                color: Color::rgba(255, 255, 0, 20),
                max_distance: 20.0,
                animate: false,
                anim_speed: 5.0,
                dither: 0.5,
                camera_type: CameraType::ManualViewWithScale,
            });

            lights.insert_area_light(AreaLight {
                pos: Vec2::new(100.0, 100.0),
                color: Color::rgba(255, 255, 0, 20),
                max_distance: 20.0,
                animate: true,
                anim_speed: 5.0,
                dither: 0.8,
                camera_type: CameraType::None,
            });

            lights.insert_directional_light(DirectionalLight {
                pos: Vec2::new(24.0, 24.0),
                color: Color::rgba(255, 255, 0, 20),
                max_distance: 90.0,
                max_width: 15.0,
                anim_speed: 2.0,
                angle: 90.0,
                dither: 6.0,
                fade_distance: 5.0,
                edge_fade_distance: 0.5,
                animate: false,
                camera_type: CameraType::ManualViewWithScale,
            });

            lights.insert_directional_light(DirectionalLight {
                pos: Vec2::new(150.0, 150.0),
                color: Color::rgba(255, 255, 0, 20),
                max_distance: 90.0,
                max_width: 10.0,
                anim_speed: 2.0,
                angle: 90.0,
                dither: 6.0,
                fade_distance: 4.0,
                edge_fade_distance: 0.6,
                animate: true,
                camera_type: CameraType::None,
            });
            // Allow the window to be seen. hiding it then making visible speeds up
            // load times.
            renderer.window().set_visible(true);

            let mut rect = Rect::new(&mut renderer, 0);
            rect.set_size(Vec2::new(32.0, 32.0))
                .set_position(Vec3::new(40.0, 40.0, 1.0))
                .set_radius(8.0)
                .set_border_color(Color::rgba(0, 0, 0, 255))
                .set_border_width(2.0)
                .set_use_camera(CameraType::ManualViewWithScale);

            // add everything into our convience type for quicker access and passing.
            let state = State {
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
                lights,
                light_renderer,
                ui_atlas: atlases.remove(0),
                ui_renderer,
                rect,
            };

            // Create the mouse/keyboard bindings for our stuff.
            let mut bindings = Bindings::<Action, Axis>::new();
            bindings
                .insert_action(Action::Quit, vec![Key::Character('q').into()]);

            // You should change this if you want to render continuously
            //event_loop.set_control_flow(ControlFlow::Wait);

            *self = Self::Ready {
                text,
                renderer,
                state,
                input_handler: InputHandler::new(
                    bindings,
                    Duration::from_millis(150),
                ),
                frame_time: FrameTime::new(),
                time: 0.0f32,
                fps: 0u32,
                size,
            };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Self::Ready {
            text,
            renderer,
            state,
            input_handler,
            frame_time,
            time,
            fps,
            size,
        } = self
        {
            if window_id == renderer.window().id() {
                if let WindowEvent::CloseRequested = event {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                    return;
                }
            }

            // update our inputs.
            input_handler.window_updates(renderer.window(), &event, 1.0);

            for input in input_handler.events() {
                if let input::InputEvent::MouseButtonAction(action) = input {
                    match action {
                        input::MouseButtonAction::Single(_) => {
                            info!("Single Click")
                        }
                        input::MouseButtonAction::Double(_) => {
                            info!("Double Click")
                        }
                        input::MouseButtonAction::Triple(_) => {
                            info!("Triple Click")
                        }
                        _ => panic!("No clicks?"),
                    }
                }
            }

            // update our renderer based on events here
            if !renderer.update(&event).unwrap() {
                return;
            }

            // get the current window size so we can see if we need to resize the renderer.
            let new_size = renderer.size();

            if *size != new_size {
                *size = new_size;

                // Reset screen size for the Surface here.
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

            // check if out close action was hit for esc
            if input_handler.is_action_down(&Action::Quit) {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            frame_time.update();
            let seconds = frame_time.seconds();
            // update our systems data to the gpu. this is the Camera in the shaders.
            state.system.update(renderer, frame_time);

            // update our systems data to the gpu. this is the Screen in the shaders.
            state
                .system
                .update_screen(renderer, [new_size.width, new_size.height]);

            // This adds the Image data to the Buffer for rendering.
            state.sprites.iter_mut().for_each(|sprite| {
                state.sprite_renderer.image_update(
                    sprite,
                    renderer,
                    &mut state.image_atlas,
                    0,
                );
            });

            state.sprite_renderer.image_update(
                &mut state.animation,
                renderer,
                &mut state.image_atlas,
                0,
            );

            // this cycles all the Image's in the Image buffer by first putting them in rendering order
            // and then uploading them to the GPU if they have moved or changed in any way. clears the
            // Image buffer for the next render pass. Image buffer only holds the ID's and Sortign info
            // of the finalized Indicies of each Image.
            state.sprite_renderer.finalize(renderer);

            state
                .text_renderer
                .text_update(text, &mut state.text_atlas, renderer, 0)
                .unwrap();
            state.text_renderer.finalize(renderer);

            state.map_renderer.map_update(
                &mut state.map,
                renderer,
                &mut state.map_atlas,
                [0, 1],
            );

            state.map_renderer.finalize(renderer);

            state
                .light_renderer
                .lights_update(&mut state.lights, renderer, 0);
            state.light_renderer.finalize(renderer);
            state.mesh.iter_mut().for_each(|mesh| {
                state.mesh_renderer.mesh_update(mesh, renderer, 0);
            });

            state.mesh_renderer.finalize(renderer);

            state.ui_renderer.rect_update(
                &mut state.rect,
                renderer,
                &mut state.ui_atlas,
                0,
            );
            state.ui_renderer.finalize(renderer);
            // Start encoding commands. this stores all the rendering calls for execution when
            // finish is called.
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("command encoder"),
                },
            );

            // Run the render pass. for the games renderer
            state.render(renderer, &mut encoder);

            // Submit our command queue. for it to upload all the changes that were made.
            // Also tells the system to begin running the commands on the GPU.
            renderer.queue().submit(std::iter::once(encoder.finish()));

            if *time < seconds {
                text.set_text(
                    renderer,
                    &format!("生活,삶,जिंदगी 😀 FPS: {fps} \nyhelloy"),
                    Attrs::new(),
                    Shaping::Advanced,
                );
                *fps = 0u32;
                *time = seconds + 1.0;
            }

            *fps += 1;

            renderer.window().pre_present_notify();
            renderer.present().unwrap();

            // These clear the Last used image tags.
            //Can be used later to auto unload things not used anymore if ram/gpu ram becomes a issue.
            if *fps == 1 {
                state.image_atlas.trim();
                state.map_atlas.trim();
                state.text_atlas.trim();
                renderer.font_sys.shape_run_cache.trim(1024);
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Self::Ready {
            text: _,
            renderer,
            state: _,
            input_handler,
            frame_time: _,
            time: _,
            fps: _,
            size: _,
        } = self
        {
            input_handler.device_updates(renderer.window(), &event);
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Self::Ready {
            text: _,
            renderer,
            state: _,
            input_handler: _,
            frame_time: _,
            time: _,
            fps: _,
            size: _,
        } = self
        {
            renderer.window().request_redraw();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), GraphicsError> {
    // Create logger to output to a File
    log::set_logger(&MY_LOGGER).unwrap();
    // Set the Max level we accept logging to the file for.
    log::set_max_level(LevelFilter::Info);

    info!("starting up");

    // This allows us to take control of panic!() so we can send it to a file via the logger.
    panic::set_hook(Box::new(|panic_info| {
        let bt = Backtrace::new();

        error!("PANIC: {}, BACKTRACE: {:?}", panic_info, bt);
    }));

    env::set_var("WGPU_VALIDATION", "0");
    env::set_var("WGPU_DEBUG", "0");
    // Starts an event gathering type for the window.
    let event_loop = EventLoop::new()?;

    let mut runner = Runner::Loading;
    Ok(event_loop.run_app(&mut runner)?)
}
