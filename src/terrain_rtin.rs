use image::{ImageBuffer, Luma};
extern crate nalgebra as na;
use bevy_render::{
    pipeline::PrimitiveTopology,
    mesh::{Mesh, VertexAttributeValues, Indices},
};
use na::Scalar;
use std::{cmp::max, collections::HashMap, vec::Vec};
use bevy::prelude::*;
use anyhow::Result;

type ErrorsVec = Vec::<f32>;

use crate::rtin::{BinId, TriangleU32, Vec2u32, get_index_level_start, get_triangle_children_indices, get_triangle_coords, index_to_bin_id, pixel_coords_for_triangle_mid_point};

type HeightMapU16 = ImageBuffer<Luma<u16>, Vec::<u16>>;

pub type Trianglef32 = (Vec3, Vec3, Vec3);

/// https://codegolf.stackexchange.com/questions/44680/showcase-of-languages
pub fn is_power_of_2(x: u32) -> bool {
    ( x & !( x & (x-1) ) ) > 0
}

pub fn assert_valid_rtin_heightmap(heightmap: &HeightMapU16) {
    assert_eq!(heightmap.width(), heightmap.height());
    assert!(is_power_of_2(heightmap.width()));
}

pub fn assert_coordinate_is_within_heightmap(heightmap: &HeightMapU16, coord: Vec2u32) {
    assert!(coord[0] < heightmap.width());
    assert!(coord[1] < heightmap.height());
}

pub fn vecu32_to_vecf32(v: Vec2u32) -> Vec3 {
    Vec3::new(v[0] as f32, 0f32, v[1] as f32)
}

pub fn triangleu32_to_trianglef32(triangle: TriangleU32) -> Trianglef32 {
    (
        vecu32_to_vecf32(triangle.0),
        vecu32_to_vecf32(triangle.1),
        vecu32_to_vecf32(triangle.2)
    )
}

pub fn rtin_select_triangles_for_heightmap_process_triangle(
    heightmap: &HeightMapU16, 
    errors_vec: &Vec::<f32>,
    triangles: &mut Vec::<BinId>, 
    triangle_index: u32, 
    error_threshold: f32)  {

    let triangle_bin_id = index_to_bin_id(triangle_index);

    let (right_child_index, left_child_index) = 
        get_triangle_children_indices(triangle_bin_id);
    let has_children = (left_child_index as usize) < errors_vec.len() 
        && (right_child_index as usize) < errors_vec.len();
    let leaf_triangle = !has_children;

    let this_triangle_error = errors_vec[triangle_index as usize];
    let error_within_threshold = this_triangle_error <= error_threshold;

    if error_within_threshold || leaf_triangle {

        triangles.push(triangle_bin_id);
    } else {
        rtin_select_triangles_for_heightmap_process_triangle(
            heightmap, errors_vec, triangles, left_child_index, error_threshold);
        rtin_select_triangles_for_heightmap_process_triangle(
            heightmap, errors_vec, triangles, right_child_index, error_threshold);
    }
}

pub fn rtin_terrain_example() -> Mesh {
    let error_threshold = 0.25f32;
    let filename = "terrain.png";

    let mesh = rtin_load_terrain_bitmap(
        filename, error_threshold, 10.0, false);

    mesh.unwrap()
}

pub fn rtin_load_terrain_bitmap(
        filename: &str, error_threshold: f32, y_scale: f32,
            enable_wireframe: bool) -> Result<Mesh> {
    let terrain_bitmap = image::open(filename)?;

    let mut mesh = if enable_wireframe {
        Mesh::new(PrimitiveTopology::LineList)
    } else {
        Mesh::new(PrimitiveTopology::TriangleList)
    };

    let heightmap = terrain_bitmap.as_luma16().unwrap();

    let terrain_mesh_data = rtin_build_terrain_from_heightmap(heightmap, error_threshold);


    let mut vertices : Vec::<[f32; 3]> = Vec::new();
    let mut normals : Vec::<[f32; 3]> = Vec::new();
    let mut indices : Vec::<u32> = Vec::new();

    vertices.reserve(terrain_mesh_data.vertices.len());
    for vertex in terrain_mesh_data.vertices {
        vertices.push([vertex.x, vertex.y * y_scale, vertex.z]);
    }

    let triangle_number = terrain_mesh_data.indices.len() / 3;

    if enable_wireframe {

        for i in 0..triangle_number {
            indices.push(terrain_mesh_data.indices[i*3+0]);
            indices.push(terrain_mesh_data.indices[i*3+1]);
            indices.push(terrain_mesh_data.indices[i*3+1]);
            indices.push(terrain_mesh_data.indices[i*3+2]);
            indices.push(terrain_mesh_data.indices[i*3+2]);
            indices.push(terrain_mesh_data.indices[i*3+0]);
        }

    } else {

        for i in 0..triangle_number {
            indices.push(terrain_mesh_data.indices[i*3+0]);
            indices.push(terrain_mesh_data.indices[i*3+1]);
            indices.push(terrain_mesh_data.indices[i*3+2]);
        }
        
    }

    normals.resize(vertices.len(), [0.0f32, 1.0f32, 0.0f32]);

    let uvs = vec![[0.0, 0.0, 0.0]; vertices.len()];


    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertices));
    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL, 
        VertexAttributeValues::Float3(normals));
    mesh.set_attribute(
        Mesh::ATTRIBUTE_UV_0,
         VertexAttributeValues::Float3(uvs));
    mesh.set_indices(Some(Indices::U32(indices)));

    Ok(mesh)
}

pub struct TerrainMeshData {
   pub vertices: Vec::<Vec3>,
   pub indices: Vec::<u32>
}

trait VecClamp {
    fn clamp(&self, left: &Self, right: &Self) -> Self;
}

impl<T> VecClamp for na::Vector2<T> where T: Scalar + Ord + Copy {

    fn clamp(&self, left: &Self, right: &Self) -> Self {
        na::Vector2::<T>::new(
            self[0].max(left[0]).min(right[0]),
            self[1].max(left[1]).min(right[1]),
        )
    }
}

/// sample the height of a triangle corner
/// by averaging the heightmap value of the four pixel
/// around the corner. Since the vertices of the triangles
/// are on a (N+1, N+1) grid, we will use the 
/// (0, 0), (0, +1), (+1, 0), (1, 1) offsets
///
///   -------------
///   |     |     |
///   |     |     |
///   |-----X-----|
///   |     |     |
///   |     |     |
///   -------------
///
pub fn sample_heightmap_height_corner_mean(
    heightmap: &HeightMapU16, corner: Vec2u32) -> f32 {        

    let min_heightmap_vec2 = Vec2u32::new(0, 0);
    let max_heightmap_vec2 = Vec2u32::new(heightmap.width()-1, heightmap.height()-1);
        
    let offsets = &[
        Vec2u32::new(0, 0),
        // Vec2u32::new(0, 1),
        // Vec2u32::new(1, 0),
        // Vec2u32::new(1, 1),
    ];
    let mean = offsets.iter()
        .map(|offset| (corner+offset).clamp(&min_heightmap_vec2, &max_heightmap_vec2) )
        .map(|pixel_coord| 
            heightmap.get_pixel(pixel_coord[0], pixel_coord[1]).0[0] as f32 / std::u16::MAX as f32)
        .sum::<f32>() / offsets.len() as f32;

    mean
}

pub fn rtin_build_terrain_from_heightmap(
    heightmap: &HeightMapU16, error_threshold: f32) -> TerrainMeshData {
    let errors_vec = build_triangle_error_vec(heightmap);

    let mut vertices = Vec::<Vec3>::new();
    let mut indices = Vec::<u32>::new();
    let mut vertices_array_position = HashMap::<u32, usize>::new(); 

    let triangle_bin_ids = rtin_select_triangles_for_heightmap(
        heightmap, &errors_vec, error_threshold);

    for triangle_bin_id in triangle_bin_ids {
        let n_tiles = heightmap.width();
        let triangle_coords = get_triangle_coords(triangle_bin_id, n_tiles);
        let new_vertices = &[triangle_coords.0, triangle_coords.1, triangle_coords.2];

        for new_vertex in new_vertices {
            let vertex_id = new_vertex[1] * heightmap.width() + new_vertex[0];


            let vertex_index = if vertices_array_position.contains_key(&vertex_id) {
                *vertices_array_position.get(&vertex_id).unwrap()
            } else {
                let new_vertex_index = vertices.len();
                vertices_array_position.insert(vertex_id, new_vertex_index);
                let vertex_height = sample_heightmap_height_corner_mean(
                    heightmap, *new_vertex);
                let new_vertex_3d = Vec3::new(
                    new_vertex[0] as f32,
                    vertex_height,
                    new_vertex[1] as f32,
                );
                vertices.push(new_vertex_3d);
                new_vertex_index
            };
            indices.push(vertex_index as u32);
        }

    }

    TerrainMeshData {
        vertices, 
        indices
    }
}

pub fn rtin_select_triangles_for_heightmap(
    heightmap: &HeightMapU16, 
    errors_vec: &ErrorsVec, error_threshold: f32) -> Vec::<BinId> {

    let mut triangles = Vec::<BinId>::new();

    rtin_select_triangles_for_heightmap_process_triangle(
        heightmap, &errors_vec, &mut triangles, 
        0, error_threshold);
    rtin_select_triangles_for_heightmap_process_triangle(
        heightmap, &errors_vec, &mut triangles, 
        1, error_threshold);

    triangles
}


const fn num_bits<T>() -> usize { std::mem::size_of::<T>() * 8 }
fn log_2(x: u32) -> u32 {
    num_bits::<u32>() as u32 - x.leading_zeros() - 1
}


pub fn build_triangle_error_vec(heightmap: &HeightMapU16) -> Vec::<f32> {
    assert_valid_rtin_heightmap(heightmap);


    let side = heightmap.width();
    let number_of_triangles = side * side * 2 - 2;
    let number_of_levels = log_2(side)*2;
    let last_level = number_of_levels - 1;

    let last_level_index_start = get_index_level_start(last_level);
    
    // println!("number of levels: {} last_level: {}", number_of_levels, last_level);

    let mut error_vec = Vec::new();
    error_vec.resize(number_of_triangles as usize, 0.0f32);

    for triangle_index in (0..number_of_triangles).rev() {

        let triangle_bin_id = index_to_bin_id(triangle_index);
        let triangle_pixel_coordinates =
            pixel_coords_for_triangle_mid_point(triangle_bin_id, side);

        // println!("\n\ndoing index={} bin_id={:b} triangle_pixel_coordinate={} last_level_start={}", triangle_index, triangle_bin_id,
        //     triangle_pixel_coordinates, last_level_index_start);

        // println!("triangle coordinates={:?}", get_triangle_coords(triangle_bin_id, side));

        assert_coordinate_is_within_heightmap(heightmap, triangle_pixel_coordinates);

        let this_triangle_error = heightmap.get_pixel(
                triangle_pixel_coordinates[0], 
                triangle_pixel_coordinates[1]).0[0] as f32 / std::u16::MAX as f32;

        // println!("this_error={}",
        //     this_triangle_error);

        if triangle_index >= last_level_index_start {
            error_vec[triangle_index as usize] = this_triangle_error;
        } else {
            let (right_child_index, left_child_index) = 
                get_triangle_children_indices(triangle_bin_id);

            // println!("right_child_index: {}, left_child_index: {}", right_child_index, left_child_index);

            let right_error = error_vec[right_child_index as usize];
            let left_error = error_vec[left_child_index as usize];

            error_vec[triangle_index as usize] = left_error.max(right_error).max(this_triangle_error);
        }
        
    }

    error_vec
}

#[cfg(test)]
mod tests {
    use bevy::ui::widget::Image;

    use super::*;

    #[test]
    fn test_build_triangle_error_vec() {
        let heightmap_data = vec![
            0u16,     256u16,
            256u16,  1024u16 
        ];

        let heightmap  = 
            HeightMapU16::from_vec(2, 2, heightmap_data).unwrap();

        let error_vec = build_triangle_error_vec(&heightmap);

        assert_eq!(error_vec, 
         vec![0.0, 0.1, 0.3, 0.4, 0.5, 0.6]);
    }

}