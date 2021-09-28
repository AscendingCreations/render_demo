use crate::graphics::{
    atlas::{Allocation, Layer},
    textures::Texture,
};
use std::collections::HashMap;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

pub struct Atlas {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub layers: Vec<Layer>,
    pub extent: wgpu::Extent3d,
    pub names: HashMap<String, Allocation>,
    //Used to deturmine if we need to redo the Bindgroup
    //associated with this texture array
    pub dirty: bool,
}

impl Atlas {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Atlas"),
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
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
            names: HashMap::new(),
            dirty: true,
        }
    }

    pub fn upload(
        &mut self,
        texture: &Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Allocation> {
        if let Some(allocation) = self.names.get(texture.name()) {
            Some(allocation.clone())
        } else {
            let (width, height) = texture.size();

            let allocation = {
                let nlayers = self.layers.len();
                let allocation = self.allocate(width, height)?;
                self.grow(self.layers.len() - nlayers, device, queue);

                allocation
            };

            self.upload_allocation(texture.bytes(), width, height, 0, &allocation, queue);
            self.names
                .insert(texture.name().to_string(), allocation.clone());
            Some(allocation)
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<Allocation> {
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
                });
            }
        }

        /* Add a new layer, as we found no layer to allocate from. */
        let mut layer = Layer::new(self.extent.width);

        if let Some(allocation) = layer.allocator.allocate(width, height) {
            self.layers.push(layer);

            return Some(Allocation {
                allocation,
                layer: self.layers.len() - 1,
            });
        }

        /* We are out of luck. */
        None
    }

    fn upload_allocation(
        &mut self,
        buffer: &[u8],
        image_width: u32,
        image_height: u32,
        offset: usize,
        allocation: &Allocation,
        queue: &wgpu::Queue,
    ) {
        let (x, y) = allocation.position();
        let (width, height) = allocation.size();
        let layer = allocation.layer;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: layer as u32,
            },
        );
    }

    pub fn get(&mut self, name: String) -> Option<Allocation> {
        self.names.get(&name).cloned()
    }

    pub fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.allocator.clear();
        }

        self.names.clear();
    }

    fn grow(&mut self, amount: usize, device: &wgpu::Device, queue: &wgpu::Queue) {
        if amount == 0 {
            return;
        }

        let extent = wgpu::Extent3d {
            width: self.extent.width,
            height: self.extent.height,
            depth_or_array_layers: self.layers.len() as u32,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        let amount_to_copy = self.layers.len() - amount;

        let mut encoder = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("command encoder"),
        });

        for (i, _) in self.layers.iter_mut().take(amount_to_copy).enumerate() {
           

            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTextureBase {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTextureBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.extent.width,
                    height: self.extent.height,
                    depth_or_array_layers: i as u32,
                },
            );
        }

        self.texture = texture;
        self.texture_view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Texture Atlas"),
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(self.layers.len() as u32),
        });
        queue.submit(std::iter::once(encoder.finish()));
        self.dirty = true;
    }
}
