use std::{marker::PhantomData, ops::Range};
use wgpu::util::DeviceExt;

#[derive(Copy, Clone, Debug, Default)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub obj_w: f32,
    pub obj_h: f32,
}

impl Bounds {
    pub fn new(x: f32, y: f32, w: f32, h: f32, obj_w: f32, obj_h: f32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            obj_w,
            obj_h,
        }
    }
}

pub trait InstanceLayout {
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
    pub buffer: wgpu::Buffer,
    pub bounds: Vec<Option<Bounds>>,
    count: usize,
    len: usize,
    max: usize,
    // Ghost Data that doesnt Actually exist. Used to set the Generic to a trait.
    // without needing to set it to a variable that is loaded.
    phantom_data: PhantomData<K>,
}

impl<K: InstanceLayout> InstanceBuffer<K> {
    /// Used to create GpuBuffer from a BufferPass.
    pub fn create_buffer(device: &wgpu::Device, data: &[u8]) -> Self {
        InstanceBuffer {
            buffer: device.create_buffer_init(
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
            phantom_data: PhantomData,
        }
    }

    //private but resizes the buffer on the GPU when needed.
    fn resize(&mut self, device: &wgpu::Device, capacity: usize) {
        let data = K::with_capacity(capacity);

        self.buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &data,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
            });
        self.count = 0;
        self.max = data.len();
    }

    /// creates a new pre initlized InstanceBuffer with a default size.
    /// default size is based on the initial InstanceLayout::default_buffer length.
    pub fn new(device: &wgpu::Device) -> Self {
        Self::create_buffer(device, &K::default_buffer())
    }

    /// Sets the buffer from byte array of instances.
    /// Sets the count to array length / instance_stride.
    /// Sets the bounds 1 Per Counted Object for Scissoring.
    /// Will resize both vertex_buffer and index_buffer if bytes length is larger than vertex_max.
    pub fn set_from(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        bounds: &[Option<Bounds>],
    ) {
        let size = bytes.len();

        if size > self.max {
            self.resize(device, size / K::instance_stride());
        }

        self.count = size / K::instance_stride();
        self.len = size;
        self.bounds = bounds.to_vec();

        queue.write_buffer(&self.buffer, 0, bytes);
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
    pub fn with_capacity(device: &wgpu::Device, capacity: usize) -> Self {
        Self::create_buffer(device, &K::with_capacity(capacity))
    }
}
