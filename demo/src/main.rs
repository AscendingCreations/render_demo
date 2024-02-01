#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use backtrace::Backtrace;
use camera::{
    controls::{Controls, FlatControls, FlatSettings},
    Projection,
};
use cosmic_text::{Attrs, Metrics};
use glam::vec4;
use graphics::*;
use hecs::World;
use input::{Bindings, FrameTime, InputHandler, Key};
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
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::{Backends, Dx12Compiler, InstanceDescriptor, InstanceFlags};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowButtons},
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

#[tokio::main]
async fn main() -> Result<(), AscendingError> {
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

    // Starts an event gathering type for the window.
    let event_loop = EventLoop::new()?;

    // Builds the Windows that will be rendered too.
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Demo")
            .with_inner_size(PhysicalSize::new(800, 600))
            .with_visible(false)
            .with_enabled_buttons({
                let mut buttons = WindowButtons::all();
                buttons.remove(WindowButtons::MAXIMIZE);
                buttons
            })
            .build(&event_loop)
            .unwrap(),
    );

    // Generates an Instance for WGPU. Sets WGPU to be allowed on all possible supported backends
    // These are DX12, DX11, Vulkan, Metal and Gles. if none of these work on a system they cant
    // play the game basically.
    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        flags: InstanceFlags::default(),
        dx12_shader_compiler: Dx12Compiler::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    // This is used to ensure the GPU can load the correct.
    let compatible_surface = instance.create_surface(window.clone()).unwrap();

    print!("{:?}", &compatible_surface);
    // This creates the Window Struct and Device struct that holds all the rendering information
    // we need to render to the screen. Window holds most of the window information including
    // the surface type. device includes the queue and GPU device for rendering.
    // This then adds gpu_window and gpu_device and creates our renderer type. for easy passing of window, device and font system.
    let mut renderer = instance
        .create_device(
            window,
            &wgpu::RequestAdapterOptions {
                // High performance mode says to use Dedicated Graphics devices first.
                // Low power is APU graphic devices First.
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&compatible_surface),
                // we will never use this as this forces us to use an alternative renderer.
                force_fallback_adapter: false,
            },
            // used to deturmine if we need special limits or features for our backends.
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
            // How we are presenting the screen which causes it to either clip to a FPS limit or be unlimited.
            wgpu::PresentMode::AutoVsync,
        )
        .await
        .unwrap();

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
    let allocation = Texture::from_file("images/Female_1.png")?
        .upload(&mut atlases[0], &renderer)
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

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
    let ui_renderer = RectRenderer::new(&mut renderer).unwrap();

    // get the screen size.
    let mut size = renderer.size();

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

    let _tilesheet = Texture::from_file(format!("images/tiles/1.png"))?
        .new_tilesheet(&mut atlases[1], &renderer, 20)
        .ok_or_else(|| OtherError::new("failed to upload tiles"))?;

    //println!("tilesheet: {:?}", tilesheet);

    let allocation = Texture::from_file("images/anim/0.png")?
        .upload(&mut atlases[0], &renderer)
        .ok_or_else(|| OtherError::new("failed to upload image"))?;

    let mut animation = Image::new(Some(allocation), &mut renderer, 2);

    animation.pos = Vec3::new(96.0, 96.0, 5.0);
    animation.hw = Vec2::new(64.0, 64.0);
    animation.uv = Vec4::new(0.0, 0.0, 64.0, 64.0);
    animation.color = Color::rgba(255, 255, 255, 255);
    animation.frames = Vec2::new(8.0, 4.0);
    animation.switch_time = 300;
    animation.animate = true;

    // get the Scale factor the pc currently is using for upscaling or downscaling the rendering.
    let scale = renderer.window().current_monitor().unwrap().scale_factor();

    // create a Text rendering object.
    let mut text = Text::new(
        &mut renderer,
        Some(Metrics::new(16.0, 16.0).scale(scale as f32)),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::new(190.0, 32.0),
        1.0,
    );

    text.set_buffer_size(&mut renderer, size.width as i32, size.height as i32)
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
    let mut mesh = [Mesh2D::new(&mut renderer), Mesh2D::new(&mut renderer)];
    mesh[0].from_builder(builder.finalize());
    mesh[1].from_builder(builder2.finalize());

    let mut lights = Lights::new(&mut renderer, 0);

    lights.world_color = Vec4::new(0.0, 0.0, 0.0, 0.995);
    lights.enable_lights = true;

    lights.insert_area_light(AreaLight {
        pos: Vec2::new(24.0, 24.0),
        color: Color::rgba(255, 255, 0, 20),
        max_distance: 20.0,
        animate: false,
        anim_speed: 5.0,
        dither: 0.5,
    });

    lights.insert_area_light(AreaLight {
        pos: Vec2::new(100.0, 100.0),
        color: Color::rgba(255, 255, 0, 20),
        max_distance: 20.0,
        animate: true,
        anim_speed: 5.0,
        dither: 0.8,
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
        .set_use_camera(true);

    // add everything into our convience type for quicker access and passing.
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
        lights,
        light_renderer,
        ui_atlas: atlases.remove(0),
        ui_renderer,
        rect,
    };

    // Create the mouse/keyboard bindings for our stuff.
    let mut bindings = Bindings::<Action, Axis>::new();
    bindings.insert_action(Action::Quit, vec![Key::Character('q').into()]);

    // set bindings and create our own input handler.
    let mut input_handler = InputHandler::new(bindings);

    let mut frame_time = FrameTime::new();
    let mut time = 0.0f32;
    let mut fps = 0u32;

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        // we check for the first batch of events to ensure we dont need to stop rendering here first.
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } if window_id == renderer.window().id() => {
                if let WindowEvent::CloseRequested = *event {
                    println!("The close button was pressed; stopping");
                    elwt.exit();
                }
            }
            Event::AboutToWait => {
                renderer.window().request_redraw();
            }
            _ => {}
        }

        // get the current window size so we can see if we need to resize the renderer.
        let new_size = renderer.size();
        let inner_size = renderer.window().inner_size();

        // if our rendering size is zero stop rendering to avoid errors.
        if new_size.width == 0.0
            || new_size.height == 0.0
            || inner_size.width == 0
            || inner_size.height == 0
        {
            return;
        }

        // update our inputs.
        input_handler.update(renderer.window(), &event, 1.0);

        // update our renderer based on events here
        if !renderer.update(&event).unwrap() {
            return;
        }

        if size != new_size {
            size = new_size;

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
            elwt.exit();
        }

        let seconds = frame_time.seconds();
        // update our systems data to the gpu. this is the Camera in the shaders.
        state.system.update(&renderer, &frame_time);

        // update our systems data to the gpu. this is the Screen in the shaders.
        state
            .system
            .update_screen(&renderer, [new_size.width, new_size.height]);

        // This adds the Image data to the Buffer for rendering.
        state.sprites.iter_mut().for_each(|sprite| {
            state.sprite_renderer.image_update(
                sprite,
                &mut renderer,
                &mut state.image_atlas,
                0,
            );
        });

        state.sprite_renderer.image_update(
            &mut state.animation,
            &mut renderer,
            &mut state.image_atlas,
            0,
        );

        // this cycles all the Image's in the Image buffer by first putting them in rendering order
        // and then uploading them to the GPU if they have moved or changed in any way. clears the
        // Image buffer for the next render pass. Image buffer only holds the ID's and Sortign info
        // of the finalized Indicies of each Image.
        state.sprite_renderer.finalize(&mut renderer);

        state
            .text_renderer
            .text_update(&mut text, &mut state.text_atlas, &mut renderer, 0)
            .unwrap();
        state.text_renderer.finalize(&mut renderer);

        state.map_renderer.map_update(
            &mut state.map,
            &mut renderer,
            &mut state.map_atlas,
            [0, 1],
        );

        state.map_renderer.finalize(&mut renderer);

        state
            .light_renderer
            .lights_update(&mut state.lights, &mut renderer, 0);
        state.light_renderer.finalize(&mut renderer);
        state.mesh.iter_mut().for_each(|mesh| {
            state.mesh_renderer.mesh_update(mesh, &mut renderer, 0);
        });

        state.mesh_renderer.finalize(&mut renderer);

        state.ui_renderer.rect_update(
            &mut state.rect,
            &mut renderer,
            &mut state.ui_atlas,
            0,
        );
        state.ui_renderer.finalize(&mut renderer);
        // Start encoding commands. this stores all the rendering calls for execution when
        // finish is called.
        let mut encoder = renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            },
        );

        // Run the render pass. for the games renderer
        state.render(&renderer, &mut encoder);

        // Submit our command queue. for it to upload all the changes that were made.
        // Also tells the system to begin running the commands on the GPU.
        renderer.queue().submit(std::iter::once(encoder.finish()));

        if time < seconds {
            text.set_text(
                &mut renderer,
                &format!("生活,삶,जिंदगी 😀 FPS: {fps} \nyhelloy"),
                Attrs::new(),
                Shaping::Advanced,
            );
            fps = 0u32;
            time = seconds + 1.0;
        }

        fps += 1;

        input_handler.end_frame();
        frame_time.update();
        renderer.present().unwrap();

        /*renderer.window_mut().set_cursor_icon(
            iced_winit::conversion::mouse_interaction(
                iced_state.mouse_interaction(),
            ),
        );*/

        // These clear the Last used image tags.
        //Can be used later to auto unload things not used anymore if ram/gpu ram becomes a issue.
        state.image_atlas.trim();
        state.map_atlas.trim();
        state.text_atlas.trim();
    })?;

    Ok(())
}
