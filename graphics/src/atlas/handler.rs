use crate::{Allocation, Layer};
use lru::LruCache;
use std::time::{Duration, Instant};
use std::{hash::Hash, num::NonZeroU32};

pub struct Atlas<U: Hash + Eq + Clone = String, Data: Copy + Default = i32> {
    /// Texture in GRAM
    pub texture: wgpu::Texture,
    /// Texture View for WGPU
    pub texture_view: wgpu::TextureView,
    /// Layers of texture.
    pub layers: Vec<Layer>,
    /// Holds the Original Texture Size and layer information.
    pub extent: wgpu::Extent3d,
    /// File Paths or names to prevent duplicates.
    pub cache: LruCache<U, Allocation<Data>>,
    /// Format the Texture uses.
    pub format: wgpu::TextureFormat,
    /// When layer at when to start recycling old allocations before making new layers.
    /// if set to zero it will never clear this automatically.
    pub cache_start: usize,
    /// Duration deturmined if an item is unused and can be removed from the cache.
    pub cache_duration: Duration,
}

impl<U: Hash + Eq + Clone, Data: Copy + Default> Atlas<U, Data> {
    fn allocate(
        &mut self,
        width: u32,
        height: u32,
        data: Data,
    ) -> Option<Allocation<Data>> {
        /* Check if the allocation would fit. */
        if width > self.extent.width || height > self.extent.height {
            return None;
        }

        /* Try allocating from an existing layer. */
        for (i, layer) in self.layers.iter_mut().enumerate() {
            if let Some(allocation) = layer.allocator.allocate(width, height) {
                return Some(Allocation {
                    allocation,
                    layer: i,
                    data,
                    used_last: Instant::now(),
                });
            }
        }

        // Lets try to handle caching here if enabled. We will check the oldest allocation
        // By doing so we will see if it really has not been used based on Duration.
        // If it has not been accessed in a while we will unload it and try to see
        // if the new items fits within it. If this fails we will make a new layer.
        // This is mostly used for things that dont hold onto the allocations.
        // If they do hold the allocation cache_start needs to be 0.
        if self.cache_start > 0 && self.layers.len() >= self.cache_start {
            let mut removed = false;
            // we will clean up all old allocations first.
            loop {
                if let Some((key, old_allocation)) = self.cache.pop_lru() {
                    if old_allocation.used_last.elapsed() < self.cache_duration
                    {
                        if old_allocation.size() == (width, height) {
                            return Some(Allocation {
                                allocation: old_allocation.allocation,
                                layer: old_allocation.layer,
                                data,
                                used_last: Instant::now(),
                            });
                        } else {
                            self.layers
                                .get_mut(old_allocation.layer)
                                .unwrap()
                                .allocator
                                .deallocate(old_allocation.allocation);
                            removed = true;
                            continue;
                        }
                    } else {
                        self.cache.push(key.clone(), old_allocation);
                        self.cache.demote(&key);
                    }
                }

                break;
            }

            if removed {
                for (i, layer) in self.layers.iter_mut().enumerate() {
                    if let Some(allocation) =
                        layer.allocator.allocate(width, height)
                    {
                        return Some(Allocation {
                            allocation,
                            layer: i,
                            data,
                            used_last: Instant::now(),
                        });
                    }
                }
            }
        }

        /* Add a new layer, as we found no layer to allocate from and could
        not retrieve any old allocations to use. */
        let mut layer = Layer::new(self.extent.width);

        if let Some(allocation) = layer.allocator.allocate(width, height) {
            self.layers.push(layer);

            return Some(Allocation {
                allocation,
                layer: self.layers.len() - 1,
                data,
                used_last: Instant::now(),
            });
        }

        /* We are out of luck. */
        None
    }

    pub fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.allocator.clear();
        }

        self.cache.clear();
    }

    // Gets the data and updates its cache position and time.
    pub fn get(&mut self, key: &U) -> Option<Allocation<Data>> {
        if let Some(mut allocation) = self.cache.get_mut(key) {
            allocation.used_last = Instant::now();
            return Some(*allocation);
        }

        None
    }

    // Gets the data without updating its cache or time.
    pub fn peek(&mut self, key: &U) -> Option<Allocation<Data>> {
        if let Some(allocation) = self.cache.peek(key) {
            return Some(*allocation);
        }

        None
    }

    // Checks if the Data Exists or not.
    pub fn contains(&mut self, key: &U) -> bool {
        self.cache.contains(key)
    }

    // Changes allocations time and its position in cache.
    pub fn promote(&mut self, key: &U) {
        if let Some(mut allocation) = self.cache.get_mut(key) {
            allocation.used_last = Instant::now();
        }
    }

    fn grow(
        &mut self,
        amount: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if amount == 0 {
            return;
        }

        let extent = wgpu::Extent3d {
            width: self.extent.width,
            height: self.extent.height,
            depth_or_array_layers: self.layers.len() as u32,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: extent,
            mip_level_count: 0,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Bgra8Unorm],
        });

        let amount_to_copy = self.layers.len() - amount;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Texture command encoder"),
            });

        for (i, _) in self.layers.iter_mut().take(amount_to_copy).enumerate() {
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTextureBase {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTextureBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.extent.width,
                    height: self.extent.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.texture = texture;
        self.texture_view =
            self.texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Texture Atlas"),
                format: Some(self.format),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: std::num::NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: NonZeroU32::new(self.layers.len() as u32),
            });
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn new(
        device: &wgpu::Device,
        size: u32,
        format: wgpu::TextureFormat,
        cache_start: usize,
        cache_duration: Duration,
    ) -> Self {
        let extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[format],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Atlas"),
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(1),
        });

        Self {
            texture,
            texture_view,
            layers: vec![Layer::new(size)],
            extent,
            cache: LruCache::unbounded(),
            format,
            cache_start,
            cache_duration,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        hash: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Allocation<Data>> {
        if let Some(allocation) = self.cache.get(&hash) {
            Some(*allocation)
        } else {
            let allocation = {
                let nlayers = self.layers.len();
                let allocation = self.allocate(width, height, data)?;
                self.grow(self.layers.len() - nlayers, device, queue);

                allocation
            };

            self.upload_allocation(bytes, &allocation, queue);
            self.cache.push(hash, allocation);
            Some(allocation)
        }
    }

    fn upload_allocation(
        &mut self,
        buffer: &[u8],
        allocation: &Allocation<Data>,
        queue: &wgpu::Queue,
    ) {
        let (x, y) = allocation.position();
        let (width, height) = allocation.size();
        let layer = allocation.layer;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x,
                    y,
                    z: layer as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(
                    if self.format == wgpu::TextureFormat::Rgba8UnormSrgb {
                        4 * width
                    } else {
                        width
                    },
                ),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
