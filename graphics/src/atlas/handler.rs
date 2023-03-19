use crate::{Allocation, GpuRenderer, Layer};
use std::{collections::HashMap, hash::Hash, num::NonZeroU32};

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
    pub cache: [HashMap<U, Allocation<Data>>; 2],
    /// Format the Texture uses.
    pub format: wgpu::TextureFormat,
    /// When Eviction starts each Cleanup Call. This is the amount of layers.
    /// 0 means it will never evict anything.
    pub pressure_min: usize,
    /// When the System will Error if reached. This is the max allowed Layers
    /// Default is 256 as Most GPU allow a max of 256.
    pub pressure_max: usize,
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
                });
            }
        }

        /* Add a new layer, as we found no layer to allocate from and could
        not retrieve any old allocations to use. */

        if self.layers.len() + 1 == self.pressure_max {
            return None;
        }

        let mut layer = Layer::new(self.extent.width);

        if let Some(allocation) = layer.allocator.allocate(width, height) {
            self.layers.push(layer);

            return Some(Allocation {
                allocation,
                layer: self.layers.len() - 1,
                data,
            });
        }

        /* We are out of luck. */
        None
    }

    pub fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.allocator.clear();
        }

        for cache in &mut self.cache {
            cache.clear();
        }
    }

    pub fn clean(&mut self) {
        if self.layers.len() >= self.pressure_min {
            self.cache.swap(0, 1);

            for (_key, allocation) in self.cache[1].drain() {
                self.layers
                    .get_mut(allocation.layer)
                    .unwrap()
                    .allocator
                    .deallocate(allocation.allocation);
            }

            return;
        } else if self.cache[0].len() < self.cache[1].len() {
            self.cache.swap(0, 1);
        }

        let mut data = Vec::with_capacity(self.cache[1].len());

        for (key, allocation) in self.cache[1].drain() {
            data.push((key, allocation));
        }

        for (key, allocation) in data {
            self.cache[0].insert(key, allocation);
        }
    }

    // Gets the data and updates its cache position and time.
    pub fn get(&mut self, key: &U) -> Option<Allocation<Data>> {
        let alloc = self.cache[0].remove(key);

        if let Some(allocation) = alloc {
            self.cache[1].insert(key.clone(), allocation);
            Some(allocation)
        } else {
            self.cache[1].get(key).copied()
        }
    }

    fn grow(&mut self, amount: usize, renderer: &GpuRenderer) {
        if amount == 0 {
            return;
        }

        let extent = wgpu::Extent3d {
            width: self.extent.width,
            height: self.extent.height,
            depth_or_array_layers: self.layers.len() as u32,
        };

        let texture =
            renderer.device().create_texture(&wgpu::TextureDescriptor {
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

        let mut encoder = renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Texture command encoder"),
            },
        );

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
        renderer.queue().submit(std::iter::once(encoder.finish()));
    }

    pub fn new(
        renderer: &GpuRenderer,
        size: u32,
        format: wgpu::TextureFormat,
        pressure_min: usize,
        pressure_max: usize,
    ) -> Self {
        let extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };

        let texture =
            renderer.device().create_texture(&wgpu::TextureDescriptor {
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
            cache: [HashMap::new(), HashMap::new()],
            format,
            pressure_min,
            pressure_max,
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
        renderer: &GpuRenderer,
    ) -> Option<Allocation<Data>> {
        if self.cache[0].contains_key(&hash) {
            if let Some(allocation) = self.cache[0].remove(&hash) {
                self.cache[1].insert(hash.clone(), allocation);
                return Some(allocation);
            }
        } else if self.cache[1].contains_key(&hash) {
            if let Some(allocation) = self.cache[1].get(&hash) {
                return Some(*allocation);
            }
        }

        let allocation = {
            let nlayers = self.layers.len();
            let allocation = self.allocate(width, height, data)?;
            self.grow(self.layers.len() - nlayers, renderer);

            allocation
        };

        self.upload_allocation(bytes, &allocation, renderer);
        self.cache[1].insert(hash, allocation);
        Some(allocation)
    }

    fn upload_allocation(
        &mut self,
        buffer: &[u8],
        allocation: &Allocation<Data>,
        renderer: &GpuRenderer,
    ) {
        let (x, y) = allocation.position();
        let (width, height) = allocation.size();
        let layer = allocation.layer;

        renderer.queue().write_texture(
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
