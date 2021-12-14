use bytemuck::{Pod, Zeroable};
use camera::Projection;
use crevice::std140::{AsStd140, Std140};
use ultraviolet::Mat4;
use wgpu::util::DeviceExt;

pub(crate) use super::{Layout, LayoutStorage, Renderer};

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct CameraLayout;

impl Layout for CameraLayout {
    fn create_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

#[derive(AsStd140)]
pub struct CameraUniform {
    view_proj: mint::ColumnMatrix4<f32>,
    eye: mint::Vector3<f32>,
}

pub struct Camera<Controls: camera::controls::Controls> {
    camera: camera::Camera<Controls>,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl<Controls> Camera<Controls>
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
        renderer: &Renderer,
        layout_storage: &mut LayoutStorage,
        projection: Projection,
        controls: Controls,
    ) -> Self {
        let mut camera = camera::Camera::new(projection, controls);

        // FIXME: think more about the initial state of the camera.
        // Update the camera.
        camera.update(0.0);

        // Create the camera uniform.
        let proj: Mat4 = camera.projection().into();
        let view: Mat4 = camera.view().into();
        let view_proj: mint::ColumnMatrix4<f32> = (proj * view).into();
        let eye: mint::Vector3<f32> = camera.eye().into();

        let camera_info = CameraUniform { view_proj, eye };

        // Create the uniform buffer.
        let buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: camera_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout =
            layout_storage.create_layout(renderer.device(), CameraLayout);

        // Create the bind group.
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: Some("camera_bind_group"),
                });

        Self {
            camera,
            buffer,
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

    pub fn update(&mut self, renderer: &Renderer, delta: f32) {
        if !self.camera.update(delta) {
            return;
        }

        // Create the camera uniform.
        let proj: Mat4 = self.camera.projection().into();
        let view: Mat4 = self.camera.view().into();
        let view_proj: mint::ColumnMatrix4<f32> = (proj * view).into();
        let eye: mint::Vector3<f32> = self.camera.eye().into();

        let camera_info = CameraUniform { view_proj, eye };

        renderer.queue().write_buffer(
            &self.buffer,
            0,
            camera_info.as_std140().as_bytes(),
        );
    }

    pub fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.view()
    }
}
