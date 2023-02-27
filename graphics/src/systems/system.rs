use crate::{Bounds, GpuDevice, Layout, LayoutStorage};
use bytemuck::{Pod, Zeroable};
use camera::Projection;
use crevice::std140::AsStd140;
use input::FrameTime;
use ultraviolet::{Mat4, Vec2, Vec4};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct SystemLayout;

impl Layout for SystemLayout {
    fn create_layout(&self, gpu_device: &GpuDevice) -> wgpu::BindGroupLayout {
        gpu_device.device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("system_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        )
    }
}

#[derive(AsStd140)]
pub struct CameraUniform {
    view: mint::ColumnMatrix4<f32>,
    proj: mint::ColumnMatrix4<f32>,
    eye: mint::Vector3<f32>,
    scale: f32,
}

#[derive(AsStd140)]
pub struct ScreenUniform {
    size: mint::Vector2<f32>,
}

#[derive(AsStd140)]
pub struct TimeUniform {
    //seconds since the start of the program. given by the FrameTime
    seconds: f32,
}

pub struct System<Controls: camera::controls::Controls> {
    camera: camera::Camera<Controls>,
    pub screen_size: [f32; 2],
    camera_buffer: wgpu::Buffer,
    time_buffer: wgpu::Buffer,
    screen_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl<Controls> System<Controls>
where
    Controls: camera::controls::Controls,
{
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn controls(&self) -> &Controls {
        self.camera.controls()
    }

    pub fn controls_mut(&mut self) -> &mut Controls {
        self.camera.controls_mut()
    }

    pub fn eye(&self) -> [f32; 3] {
        self.camera.eye()
    }

    pub fn new(
        gpu_device: &GpuDevice,
        layout_storage: &mut LayoutStorage,
        projection: Projection,
        controls: Controls,
        screen_size: [f32; 2],
    ) -> Self {
        let mut camera = camera::Camera::new(projection, controls);

        // FIXME: think more about the initial state of the camera.
        // Update the camera.
        camera.update(0.0);

        // Create the camera uniform.
        let proj = camera.projection();
        let view = camera.view();
        let eye: mint::Vector3<f32> = camera.eye().into();
        let scale = camera.scale();

        let camera_info = CameraUniform {
            view,
            proj,
            eye,
            scale,
        };
        let time_info = TimeUniform { seconds: 0.0 };
        let screen_info = ScreenUniform {
            size: screen_size.into(),
        };

        // Create the uniform buffers.
        let camera_buffer = gpu_device.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: camera_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let time_buffer = gpu_device.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("time buffer"),
                contents: time_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let screen_buffer = gpu_device.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Screen buffer"),
                contents: screen_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout = layout_storage.create_layout(gpu_device, SystemLayout);

        // Create the bind group.
        let bind_group =
            gpu_device
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: camera_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: time_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: screen_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("system_bind_group"),
                });

        Self {
            camera,
            screen_size,
            camera_buffer,
            time_buffer,
            screen_buffer,
            bind_group,
        }
    }

    pub fn projection(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.projection()
    }

    pub fn set_controls(&mut self, controls: Controls) -> Controls {
        self.camera.set_controls(controls)
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.camera.set_projection(projection);
    }

    pub fn update(&mut self, gpu_device: &GpuDevice, frame_time: &FrameTime) {
        if self.camera.update(frame_time.delta_seconds()) {
            let proj = self.camera.projection();
            let view = self.camera.view();
            let eye: mint::Vector3<f32> = self.camera.eye().into();
            let scale = self.camera.scale();

            let camera_info = CameraUniform {
                view,
                proj,
                eye,
                scale,
            };

            gpu_device.queue().write_buffer(
                &self.camera_buffer,
                0,
                camera_info.as_std140().as_bytes(),
            );
        }

        let time_info = TimeUniform {
            seconds: frame_time.seconds(),
        };

        gpu_device.queue().write_buffer(
            &self.time_buffer,
            0,
            time_info.as_std140().as_bytes(),
        );
    }

    pub fn update_screen(
        &mut self,
        gpu_device: &GpuDevice,
        screen_size: [f32; 2],
    ) {
        if self.screen_size != screen_size {
            self.screen_size = screen_size;
            let screen_info = ScreenUniform {
                size: screen_size.into(),
            };

            gpu_device.queue().write_buffer(
                &self.screen_buffer,
                0,
                screen_info.as_std140().as_bytes(),
            );
        }
    }

    pub fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.view()
    }

    ///Y does not calculate correctly here...
    pub fn projected_world_to_screen(
        &self,
        scale: bool,
        bounds: &Bounds,
    ) -> Bounds {
        let projection = Mat4::from(self.camera.projection());
        let model = Mat4::identity();
        let view = if scale {
            ultraviolet::Mat4::from(self.camera.view())
        } else {
            Mat4::identity()
        };
        let clip_coords = projection
            * view
            * model
            * Vec4::new(bounds.0.x, bounds.0.y, 1.0, 1.0);
        let coords = clip_coords.xyz() / clip_coords.w;

        let xy = Vec2::new(
            (coords.x + 1.0) * 0.5 * self.screen_size[0],
            (1.0 - coords.y) * 0.5 * self.screen_size[1],
        );

        let (bw, bh, objh) = if scale {
            (
                bounds.0.z * self.camera.scale(),
                bounds.0.w * self.camera.scale(),
                bounds.1 * self.camera.scale(),
            )
        } else {
            (bounds.0.z, bounds.0.w, bounds.1)
        };

        Bounds::new(Vec4::new(xy.x, xy.y - objh, bw, bh), objh)
    }

    pub fn world_to_screen(&self, scale: bool, bounds: &Bounds) -> Bounds {
        let projection = Mat4::from(self.camera.projection());
        let model = Mat4::identity();
        let clip_coords =
            projection * model * Vec4::new(bounds.0.x, bounds.0.y, 1.0, 1.0);
        let coords = clip_coords.xyz() / clip_coords.w;

        let xy = Vec2::new(
            (coords.x + 1.0) * 0.5 * self.screen_size[0],
            (1.0 - coords.y) * 0.5 * self.screen_size[1],
        );

        let (bw, bh, objh) = if scale {
            (
                bounds.0.z * self.camera.scale(),
                bounds.0.w * self.camera.scale(),
                bounds.1 * self.camera.scale(),
            )
        } else {
            (bounds.0.z, bounds.0.w, bounds.1)
        };

        Bounds::new(Vec4::new(xy.x, xy.y - objh, bw, bh), objh)
    }
}
