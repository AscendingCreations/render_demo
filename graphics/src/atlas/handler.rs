use crate::{Allocation, GpuRenderer, Layer};
use lru::LruCache;
use std::{collections::HashSet, hash::Hash};

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
    pub last_used: HashSet<U>,
    /// Format the Texture uses.
    pub format: wgpu::TextureFormat,
    /// When the System will Error if reached. This is the max allowed Layers
    /// Default is 256 as Most GPU allow a max of 256.
    pub max_layers: u32,
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

        /* Try to see if we can clear out unused allocations first. */
        loop {
            let (key, _) = self.cache.peek_lru()?;

            //Check if ID has been used yet?
            if self.last_used.contains(key) {
                //Failed to find any unused allocations so lets try to add a layer.
                break;
            }

            let (_, allocation) = self.cache.pop_lru()?;
            let layer_id = allocation.layer;
            let layer = self.layers.get_mut(layer_id).unwrap();

            layer.allocator.deallocate(allocation.allocation);

            if let Some(allocation) = layer.allocator.allocate(width, height) {
                return Some(Allocation {
                    allocation,
                    layer: layer_id,
                    data,
                });
            }
        }

        /* Add a new layer, as we found no layer to allocate from and could
        not retrieve any old allocations to use. */

        if self.layers.len() + 1 == self.max_layers as usize {
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

        self.cache.clear();
        self.last_used.clear();
    }

    pub fn trim(&mut self) {
        self.last_used.clear();
    }

    pub fn promote(&mut self, key: U) {
        self.cache.promote(&key);
        self.last_used.insert(key);
    }

    pub fn peek(&mut self, key: &U) -> Option<&Allocation<Data>> {
        self.cache.peek(key)
    }

    pub fn contains(&mut self, key: &U) -> bool {
        self.cache.contains(key)
    }

    pub fn get(&mut self, key: &U) -> Option<Allocation<Data>> {
        if let Some(allocation) = self.cache.get_mut(key) {
            self.last_used.insert(key.clone());
            return Some(*allocation);
        }

        None
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
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(self.layers.len() as u32),
            });
        renderer.queue().submit(std::iter::once(encoder.finish()));
    }

    pub fn new(renderer: &GpuRenderer, format: wgpu::TextureFormat) -> Self {
        let limits = renderer.device().limits();
        let extent = wgpu::Extent3d {
            width: limits.max_texture_dimension_3d,
            height: limits.max_texture_dimension_3d,
            depth_or_array_layers: 2,
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
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        Self {
            texture,
            texture_view,
            layers: vec![
                Layer::new(limits.max_texture_dimension_3d),
                Layer::new(limits.max_texture_dimension_3d),
            ],
            extent,
            cache: LruCache::unbounded(),
            last_used: HashSet::default(),
            format,
            max_layers: limits.max_texture_array_layers,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        key: U,
        bytes: &[u8],
        width: u32,
        height: u32,
        data: Data,
        renderer: &GpuRenderer,
    ) -> Option<Allocation<Data>> {
        if let Some(allocation) = self.get(&key) {
            Some(allocation)
        } else {
            let allocation = {
                let nlayers = self.layers.len();
                let allocation = self.allocate(width, height, data)?;
                self.grow(self.layers.len() - nlayers, renderer);

                allocation
            };

            self.upload_allocation(bytes, &allocation, renderer);
            self.cache.push(key.clone(), allocation);
            Some(allocation)
        }
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
                bytes_per_row: Some(
                    if self.format == wgpu::TextureFormat::Rgba8UnormSrgb {
                        4 * width
                    } else {
                        width
                    },
                ),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
