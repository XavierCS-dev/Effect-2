use anyhow::Result;
use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::engine::{
    entity::entity::{Entity2D, Entity2DRaw},
    primitives::vertex::Vertex,
    texture::{
        texture2d::{Texture2D, TextureID},
        texture_atlas2d::TextureAtlas2D,
    },
    traits::layer::Layer,
};

#[derive(std::cmp::PartialEq, std::cmp::Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct LayerID(pub u32);
pub struct Initialised;
pub struct Uninitialised;

pub struct Layer2D {
    id: LayerID,
    textures: HashMap<TextureID, Texture2D>,
    atlas: Option<TextureAtlas2D>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: wgpu::Buffer,
    entity_count: u32,
    entity_buffer: Option<wgpu::Buffer>,
}

impl Layer2D {
    pub fn new(id: LayerID, texture: Texture2D, device: &wgpu::Device) -> Result<Self> {
        let mut textures = HashMap::new();
        let index_buffer = Layer2DSystem::create_index_buffer(device);
        let entity_count = 0;
        Ok(Self {
            id,
            textures,
            atlas: None,
            vertex_buffer: None,
            index_buffer,
            entity_count,
            entity_buffer: None,
        })
    }

    pub fn id(&self) -> LayerID {
        self.id
    }

    pub fn contains_texture(&self, texture_id: &TextureID) -> bool {
        self.textures.contains_key(texture_id)
    }

    /// Add a texture to a layer for entities to use
    // set offset of all textures when added
    pub fn add_texture(
        &mut self,
        texture: Texture2D,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        todo!()
    }

    pub fn get_texture(&self, texture_id: &TextureID) -> Option<&Texture2D> {
        self.textures.get(texture_id)
    }

    pub fn get_texture_mut(&mut self, texture_id: &TextureID) -> Option<&mut Texture2D> {
        self.textures.get_mut(texture_id)
    }
}

impl Layer for Layer2D {
    fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.atlas?.bind_group())
    }

    fn texture_ids(&self) -> &HashMap<TextureID, Texture2D> {
        &self.textures
    }

    fn vertex_buffer(&self) -> Option<&wgpu::Buffer> {
        self.vertex_buffer.as_ref()
    }

    fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    fn entity_buffer(&self) -> Option<&wgpu::Buffer> {
        self.entity_buffer.as_ref()
    }

    fn index_count(&self) -> usize {
        (self.entity_count * 6) as usize
    }

    fn id(&self) -> LayerID {
        self.id
    }

    fn entity_count(&self) -> u32 {
        self.entity_count
    }
}

pub struct Layer2DSystem;

impl Layer2DSystem {
    fn create_entity_buffer(entities: &Vec<&Entity2D>, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(
                entities
                    .iter()
                    .map(|e| e.to_raw())
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&([0, 1, 2, 0, 2, 3] as [u16; 6])),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_vertex_buffer(entities: &Vec<&Entity2D>, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(
                entities
                    .iter()
                    .flat_map(|e| e.vertices())
                    .copied()
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    /// Update transformation data (not the vertices).
    // Panics of uninitialised
    pub fn update_entities(layer: &mut Layer2D, entities: Vec<&Entity2D>, queue: &wgpu::Queue) {
        if entities.len() as u32 > layer.entity_count() {
            panic!("Entities would not fit buffer")
        }
        let data: Vec<Entity2DRaw> = entities.iter().map(|e| e.to_raw()).collect();
        queue.write_buffer(
            &layer.entity_buffer.unwrap(),
            0,
            bytemuck::cast_slice(&data),
        );
    }

    /// Set the vertices and entity data. Use this when adding or removing entities
    pub fn set_entities(layer: &mut Layer2D, entities: Vec<&Entity2D>, device: &wgpu::Device) {
        layer.entity_count = entities.len() as u32;
        let data: Vec<Entity2DRaw> = entities.iter().map(|e| e.to_raw()).collect();
        // possibly extra copying going on here...look into it
        let vertices: Vec<Vertex> = entities.iter().flat_map(|e| *e.vertices()).collect();
        layer.entity_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Entity Buffer"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );
        layer.vertex_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        );
    }

    // Same as set entities, but reuse the buffers, for when the number of entities hasn't grown
    // Panics if unintialised
    pub fn set_entities_fast(layer: &mut Layer2D, entities: Vec<&Entity2D>, queue: &wgpu::Queue) {
        if entities.len() as u32 > layer.entity_count() {
            panic!("Entities would not fit buffer")
        }
        let data: Vec<Entity2DRaw> = entities.iter().map(|e| e.to_raw()).collect();
        // possibly extra copying going on here...look into it
        let vertices: Vec<Vertex> = entities.iter().flat_map(|e| *e.vertices()).collect();
        queue.write_buffer(
            &layer.entity_buffer.unwrap(),
            0,
            bytemuck::cast_slice(&data),
        );
        queue.write_buffer(
            &layer.vertex_buffer.unwrap(),
            0,
            bytemuck::cast_slice(&vertices),
        );
    }
}