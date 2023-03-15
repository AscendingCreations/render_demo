use crate::{
    AscendingError, BufferStore, DrawOrder, GpuDevice, GpuWindow, Layout,
    LayoutStorage, OtherError, PipeLineLayout, PipelineStorage,
    StaticBufferObject,
};
use generational_array::{
    GenerationalArray, GenerationalArrayResult, GenerationalArrayResultMut,
    GenerationalIndex,
};
use std::cmp::Ordering;
use std::rc::Rc;
use winit::{dpi::PhysicalSize, event::Event, window::Window};

pub type Index = GenerationalIndex;

pub struct OrderedIndex {
    pub(crate) order: DrawOrder,
    pub(crate) index: Index,
}

impl PartialOrd for OrderedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderedIndex {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Eq for OrderedIndex {}

impl Ord for OrderedIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl OrderedIndex {
    pub fn new(order: DrawOrder, index: Index) -> Self {
        Self { order, index }
    }
}

///Handles the Window, Device and buffer stores.
pub struct GpuRenderer {
    pub(crate) window: GpuWindow,
    pub(crate) device: GpuDevice,
    pub(crate) buffer_stores: GenerationalArray<BufferStore>,
    pub(crate) layout_storage: LayoutStorage,
    pub(crate) pipeline_storage: PipelineStorage,
    pub(crate) depthbuffer: wgpu::TextureView,
    pub(crate) framebuffer: Option<wgpu::TextureView>,
    pub(crate) frame: Option<wgpu::SurfaceTexture>,
    pub buffer_object: StaticBufferObject,
}

impl GpuRenderer {
    pub fn new(window: GpuWindow, device: GpuDevice) -> Self {
        let buffer_object = StaticBufferObject::create_buffer(&device);
        let depth_buffer = window.create_depth_texture(&device);

        Self {
            window,
            device,
            buffer_stores: GenerationalArray::new(),
            layout_storage: LayoutStorage::new(),
            pipeline_storage: PipelineStorage::new(),
            depthbuffer: depth_buffer,
            framebuffer: None,
            frame: None,
            buffer_object,
        }
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        self.window.adapter()
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<(), AscendingError> {
        self.window.resize(&self.device, size)
    }

    pub fn frame_buffer(&self) -> &Option<wgpu::TextureView> {
        &self.framebuffer
    }

    pub fn depth_buffer(&self) -> &wgpu::TextureView {
        &self.depthbuffer
    }

    pub fn size(&self) -> PhysicalSize<f32> {
        self.window.size
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.window.surface
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.window.surface_format
    }

    pub fn update(
        &mut self,
        event: &Event<()>,
    ) -> Result<bool, AscendingError> {
        let frame = match self.window.update(&self.device, event)? {
            Some(frame) => frame,
            _ => return Ok(false),
        };

        self.framebuffer = Some(
            frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        self.frame = Some(frame);

        Ok(true)
    }

    pub fn window(&self) -> &Window {
        &self.window.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window.window
    }

    pub fn update_depth_texture(&mut self) {
        self.depthbuffer = self.window.create_depth_texture(&self.device);
    }

    pub fn present(&mut self) -> Result<(), AscendingError> {
        self.framebuffer = None;

        match self.frame.take() {
            Some(frame) => {
                frame.present();
                Ok(())
            }
            None => Err(AscendingError::Other(OtherError::new(
                "Frame does not Exist. Did you forget to update the renderer?",
            ))),
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device.device
    }

    pub fn gpu_device(&self) -> &GpuDevice {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.device.queue
    }

    pub fn new_buffer(&mut self) -> Index {
        self.buffer_stores.insert(BufferStore::default())
    }

    pub fn remove_buffer(&mut self, index: Index) {
        let _ = self.buffer_stores.remove(index);
    }

    pub fn get_buffer(&self, index: &Index) -> Option<&BufferStore> {
        match self.buffer_stores.get(index) {
            GenerationalArrayResult::None => None,
            GenerationalArrayResult::OutDated => None,
            GenerationalArrayResult::OutOfBounds => None,
            GenerationalArrayResult::Some(v) => Some(v),
        }
    }

    pub fn get_buffer_mut(
        &mut self,
        index: &Index,
    ) -> Option<&mut BufferStore> {
        match self.buffer_stores.get_mut(index) {
            GenerationalArrayResultMut::None => None,
            GenerationalArrayResultMut::OutDated => None,
            GenerationalArrayResultMut::OutOfBounds => None,
            GenerationalArrayResultMut::Some(v) => Some(v),
        }
    }

    pub fn create_layout<K: Layout>(
        &mut self,
        layout: K,
    ) -> Rc<wgpu::BindGroupLayout> {
        self.layout_storage.create_layout(&mut self.device, layout)
    }

    pub fn create_pipelines(&mut self, surface_format: wgpu::TextureFormat) {
        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::ImageRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::MapRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::TextRenderPipeline,
        );

        self.pipeline_storage.create_pipeline(
            &mut self.device,
            &mut self.layout_storage,
            surface_format,
            crate::RectsRenderPipeline,
        );
    }

    pub fn get_pipelines<K: PipeLineLayout>(
        &self,
        pipeline: K,
    ) -> Option<&wgpu::RenderPipeline> {
        self.pipeline_storage.get_pipeline(pipeline)
    }
}
