use crate::{GpuDevice, GpuRenderer, Index, Vec4};
use std::{cell::RefCell, marker::PhantomData, ops::Range, rc::Rc};
use wgpu::util::DeviceExt;

#[derive(Copy, Clone, Debug)]
pub struct Bounds(pub Vec4, pub f32);

impl Bounds {
    pub fn new(bounds: Vec4, obj_h: f32) -> Self {
        Self(bounds, obj_h)
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self(Vec4::new(0.0, 0.0, 2_147_483_600.0, 2_147_483_600.0), 0.0)
    }
}

pub type BufferStoreRef = Rc<RefCell<BufferStore>>;

#[derive(Default)]
pub struct BufferStore {
    pub store: Vec<u8>,
    pub bounds: Option<Bounds>,
    pub changed: bool,
    pub pos: Range<usize>,
}

pub trait InstanceLayout {
    fn is_bounded() -> bool;
    ///WGPU's Shader Attributes
    fn attributes() -> Vec<wgpu::VertexAttribute>;

    ///Default Buffer set to a large size.
    fn default_buffer() -> Vec<u8>;

    ///The size in bytes the instance is
    fn instance_stride() -> usize;

    /// Creates a Buffer at a capacity
    /// Capacity is a count of objects.
    fn with_capacity(capacity: usize) -> Vec<u8>;
}

//This Holds onto all the instances Compressed into a byte array.
pub struct InstanceBuffer<K: InstanceLayout> {
    pub buffers: Vec<Index>,
    pub buffer: wgpu::Buffer,
    pub bounds: Vec<Option<Bounds>>,
    count: usize,
    len: usize,
    max: usize,
    // this is a calculation of the buffers size when being marked as ready to add into the buffer.
    needed_size: usize,
    // Ghost Data that doesnt Actually exist. Used to set the Generic to a trait.
    // without needing to set it to a variable that is loaded.
    phantom_data: PhantomData<K>,
}

impl<K: InstanceLayout> InstanceBuffer<K> {
    /// Used to create GpuBuffer from a BufferPass.
    /// Only use this for creating a reusable buffer.
    pub fn create_buffer(gpu_device: &GpuDevice, data: &[u8]) -> Self {
        InstanceBuffer {
            buffers: Vec::with_capacity(256),
            buffer: gpu_device.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: data,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST,
                },
            ),
            bounds: Vec::new(),
            count: 0,
            len: 0,
            max: data.len(),
            needed_size: 0,
            phantom_data: PhantomData,
        }
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &mut GpuRenderer,
        index: Index,
    ) {
        if let Some(store) = renderer.get_buffer(&index) {
            let size = store.store.len();

            self.buffers.push(index);
            self.needed_size += size;
        }
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        let mut changed = false;
        let mut pos = 0;

        if self.needed_size > self.max {
            self.resize(
                renderer.gpu_device(),
                self.needed_size / K::instance_stride(),
            );
            changed = true;
        }

        self.count = self.needed_size / K::instance_stride();
        self.len = self.needed_size;

        for buf in &self.buffers {
            let mut write_buffer = false;
            let old_pos = pos as u64;

            if let Some(store) = renderer.get_buffer_mut(buf) {
                let range = pos..pos + store.store.len();

                if store.pos != range || changed || store.changed {
                    if K::is_bounded() {
                        self.bounds.push(store.bounds);
                    }

                    store.pos = range;
                    store.changed = false;
                    write_buffer = true
                }

                pos += store.store.len();
            }

            if write_buffer {
                if let Some(store) = renderer.get_buffer(buf) {
                    renderer.device.queue.write_buffer(
                        &self.buffer,
                        old_pos,
                        &store.store,
                    );
                }
            }
        }

        self.needed_size = 0;
        self.buffers.clear();
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(&mut self, gpu_device: &GpuDevice, capacity: usize) {
        let data = K::with_capacity(capacity);

        self.buffer = gpu_device.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &data,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
            },
        );
        self.count = 0;
        self.max = data.len();
    }

    /// creates a new pre initlized InstanceBuffer with a default size.
    /// default size is based on the initial InstanceLayout::default_buffer length.
    pub fn new(gpu_device: &GpuDevice) -> Self {
        Self::create_buffer(gpu_device, &K::default_buffer())
    }

    /// Sets the buffer from byte array of instances.
    /// Sets the count to array length / instance_stride.
    /// Sets the bounds 1 Per Counted Object for Scissoring.
    /// Will resize both vertex_buffer and index_buffer if bytes length is larger than vertex_max.
    /// This will bypass the buffer optimizations. Avoid usage unless you need it.
    pub fn set_from(
        &mut self,
        gpu_device: &GpuDevice,
        bytes: &[u8],
        bounds: &[Option<Bounds>],
    ) {
        let size = bytes.len();

        if size > self.max {
            self.resize(gpu_device, size / K::instance_stride());
        }

        self.count = size / K::instance_stride();
        self.len = size;
        self.bounds = bounds.to_vec();

        gpu_device.queue().write_buffer(&self.buffer, 0, bytes);
    }

    /// Returns the elements count.
    pub fn count(&self) -> u32 {
        self.count as u32
    }

    /// Returns the elements byte count.
    pub fn len(&self) -> u64 {
        self.len as u64
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns vertex_buffer's max size in bytes.
    pub fn max(&self) -> usize {
        self.max
    }

    /// Returns buffer's stride.
    pub fn instance_stride(&self) -> usize {
        K::instance_stride()
    }

    /// Returns wgpu::BufferSlice of vertices.
    /// bounds is used to set a specific Range if needed.
    /// If bounds is None then range is 0..vertex_count.
    pub fn instances(&self, bounds: Option<Range<u64>>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds
        } else {
            0..self.len()
        };

        self.buffer.slice(range)
    }

    /// Creates a Buffer based on capacity.
    /// Capacity is the amount of objects to initialize for.
    pub fn with_capacity(gpu_device: &GpuDevice, capacity: usize) -> Self {
        Self::create_buffer(gpu_device, &K::with_capacity(capacity))
    }
}
