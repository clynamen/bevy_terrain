use bevy::prelude::*;
pub struct Terrain {}

#[derive(Default)]
pub struct TerrainImageLoadOptions {
    pub max_image_height : f32,
    pub pixel_side_length : f32
}

#[derive(Default)]
pub struct TerrainMeshResource {
    pub shaded: Handle<Mesh>,
    pub wireframe: Handle<Mesh>,
}
