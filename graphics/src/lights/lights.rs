use crate::{
    Color, DrawOrder, GpuRenderer, Index, LightsVertex, OrderedIndex, Vec2,
    Vec3,
};
use slab::Slab;

pub const MAX_LIGHTS: usize = 2000;

pub struct AreaLight {
    pub pos: Vec2,
    pub color: Color,
    pub max_distance: f32,
    pub animate: u32,
}

impl AreaLight {}

pub struct DirectionalLight {
    pub pos: Vec2,
    pub color: Color,
    pub max_distance: f32,
    pub max_radius: f32,
    pub smoothness: f32,
    pub angle: f32,
    pub animate: u32,
}

/// rendering data for world Light and all Lights.
pub struct Lights {
    pub world_color: Color,
    pub enable_lights: bool,
    pub store_id: Index,
    pub order: DrawOrder,
    pub render_layer: u32,
    pub area_lights: Slab<AreaLight>,
    pub directional_lights: Slab<DirectionalLight>,
    pub area_count: u32,
    pub dir_count: u32,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
    pub directionals_changed: bool,
    pub areas_changed: bool,
}

impl Lights {
    pub fn new(renderer: &mut GpuRenderer, render_layer: u32) -> Self {
        Self {
            world_color: Color::rgba(255, 255, 255, 0),
            enable_lights: false,
            store_id: renderer.new_buffer(),
            order: DrawOrder::default(),
            render_layer,
            area_lights: Slab::with_capacity(MAX_LIGHTS),
            directional_lights: Slab::with_capacity(MAX_LIGHTS),
            area_count: 0,
            dir_count: 0,
            changed: true,
            directionals_changed: true,
            areas_changed: true,
        }
    }

    pub fn create_quad(&mut self, renderer: &mut GpuRenderer) {
        let instance = LightsVertex {
            world_color: self.world_color.0,
            enable_lights: u32::from(self.enable_lights),
            dir_count: self.directional_lights.len() as u32,
            area_count: self.area_lights.len() as u32,
        };

        if let Some(store) = renderer.get_buffer_mut(&self.store_id) {
            store.store = bytemuck::bytes_of(&instance).to_vec();
            store.changed = true;
        }

        self.order = DrawOrder::new(false, &Vec3::default(), self.render_layer);
        self.changed = false;
    }

    pub fn insert_area_light(&mut self, light: AreaLight) -> Option<usize> {
        if self.area_lights.len() + 1 >= MAX_LIGHTS {
            return None;
        }

        self.areas_changed = true;
        self.changed = true;
        Some(self.area_lights.insert(light))
    }

    pub fn remove_area_light(&mut self, key: usize) {
        self.areas_changed = true;
        self.changed = true;
        self.area_lights.remove(key);
    }

    pub fn get_mut_area_light(&mut self, key: usize) -> Option<&mut AreaLight> {
        self.areas_changed = true;
        self.area_lights.get_mut(key)
    }

    pub fn insert_directional_light(
        &mut self,
        light: DirectionalLight,
    ) -> Option<usize> {
        if self.directional_lights.len() + 1 >= MAX_LIGHTS {
            return None;
        }

        self.directionals_changed = true;
        self.changed = true;
        Some(self.directional_lights.insert(light))
    }

    pub fn remove_directional_light(&mut self, key: usize) {
        self.directionals_changed = true;
        self.changed = true;
        self.directional_lights.remove(key);
    }

    pub fn get_mut_directional_light(
        &mut self,
        key: usize,
    ) -> Option<&mut DirectionalLight> {
        self.directionals_changed = true;
        self.directional_lights.get_mut(key)
    }

    /// used to check and update the vertex array.
    pub fn update(
        &mut self,
        renderer: &mut GpuRenderer,
        areas: &mut wgpu::Buffer,
        dirs: &mut wgpu::Buffer,
    ) -> OrderedIndex {
        // if pos or tex_pos or color changed.
        if self.changed {
            self.create_quad(renderer);
        }

        if self.areas_changed {}

        OrderedIndex::new(self.order, self.store_id, 0)
    }
}
