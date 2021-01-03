use crate::terrain_material::TerrainMaterial;
// use Srgb::into_raw;
use image::{ImageBuffer, Luma};
extern crate nalgebra as na;
use bevy_render::{
    pipeline::PrimitiveTopology,
    mesh::{Mesh, VertexAttributeValues, Indices},
};
use na::Scalar;
use std::{collections::HashMap, vec::Vec};
use bevy::prelude::*;
use anyhow::Result;
use palette::{FromColor, Gradient, Hsv, LinSrgb, Srgb};

type ErrorsVec = Vec::<f32>;

use crate::rtin::{BinId, TriangleU32, Vec2u32, get_index_level_start, get_triangle_children_bin_ids, get_triangle_children_indices, get_triangle_coords, index_to_bin_id, pixel_coords_for_triangle_mid_point};

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

pub fn triangle_errors_vec_index(bin_id: BinId, grid_size: u32) -> usize {
    let triangle_midpoint = pixel_coords_for_triangle_mid_point(
        bin_id, grid_size);
    let midpoint_error_vec_index = 
        triangle_midpoint[1] * grid_size +
        triangle_midpoint[0];

    midpoint_error_vec_index as usize
}

pub fn rtin_select_triangles_for_heightmap_process_triangle(
    heightmap: &HeightMapU16, 
    errors_vec: &Vec::<f32>,
    triangles: &mut Vec::<BinId>, 
    triangle_index: u32, 
    error_threshold: f32)  {
    
    let grid_size = heightmap.width() + 1;

    let triangle_bin_id = index_to_bin_id(triangle_index);

    let (right_child_index, left_child_index) = 
        get_triangle_children_indices(triangle_bin_id);

    let side = heightmap.width();
    let number_of_last_level_triangles = side*side*2;
    let number_of_triangles = side * side * 2 - 2 + number_of_last_level_triangles;

    let has_children = right_child_index < number_of_triangles;

    let leaf_triangle = !has_children;

    let this_triangle_errors_vec_index = triangle_errors_vec_index(
        triangle_bin_id, grid_size);
    let this_triangle_error = errors_vec[this_triangle_errors_vec_index];
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
    let mut indices : Vec::<u32> = Vec::new();
    let mut colors  : Vec::<[f32; 3]> = Vec::new();
    let indices_len = if enable_wireframe {
        terrain_mesh_data.indices.len() * 2
    } else {
        terrain_mesh_data.indices.len()
    };

    vertices.reserve(terrain_mesh_data.vertices.len());
    colors.reserve(vertices.len());
    indices.reserve(indices_len);

    // let grad = Gradient::new(vec![
    //     LinSrgb::new(1.0, 0.1, 0.1),
    //     LinSrgb::new(0.1, 1.0, 1.0)
    // ]);
    let grad = Gradient::new(vec![
        Hsv::from(LinSrgb::new(1.0, 0.1, 0.1)),
        Hsv::from(LinSrgb::new(0.1, 1.0, 1.0))
    ]);

    for vertex in &terrain_mesh_data.vertices {
        vertices.push(
            [vertex.x, 
            vertex.y * y_scale, 
            vertex.z]);

        let color = grad.get(vertex.y);
        let raw_float : Srgb::<f32> = 
            Srgb::<f32>::from_linear(color.into());
        colors.push([raw_float.red, raw_float.green, raw_float.blue]);
    }


    let triangle_number = terrain_mesh_data.indices.len() / 3;

    if enable_wireframe {
        for i in 0..triangle_number {
            for j in &[0, 1, 1, 2, 2, 0] {
                indices.push(terrain_mesh_data.indices[i*3+j]);
            }
        }
    } else {
        for i in 0..triangle_number {
            for j in 0..3 {
                indices.push(terrain_mesh_data.indices[i*3+j]);
            }
        }
    }


    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertices));
    mesh.set_attribute(
        TerrainMaterial::ATTRIBUTE_COLOR, 
        VertexAttributeValues::Float3(colors)
    );
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

pub fn sample_heightmap_height_corner_mean(
    heightmap: &HeightMapU16, corner_u32: Vec2u32) -> f32 {        

    let mut new_corner = corner_u32;

    if new_corner[0] >= heightmap.width() {
        new_corner[0] = heightmap.width() - 1;
    }

    if new_corner[1] >= heightmap.height() {
        new_corner[1] = heightmap.height() - 1;
    }

    heightmap.get_pixel(
        new_corner[0], new_corner[1]).0[0] as f32
        / std::u16::MAX as f32


}

pub fn rtin_build_terrain_from_heightmap(
    heightmap: &HeightMapU16, error_threshold: f32) -> TerrainMeshData {
    let errors_vec = build_triangle_errors_vec(heightmap);

    let mut vertices = Vec::<Vec3>::new();
    let mut indices = Vec::<u32>::new();
    let mut vertices_array_position = HashMap::<u32, usize>::new(); 

    let triangle_bin_ids = rtin_select_triangles_for_heightmap(
        heightmap, &errors_vec, error_threshold);

    for triangle_bin_id in triangle_bin_ids {
        let grid_size = heightmap.width() + 1;
        let triangle_coords = get_triangle_coords(triangle_bin_id, grid_size);
        let new_vertices = &[triangle_coords.0, triangle_coords.1, triangle_coords.2];

        for new_vertex in new_vertices {
            let vertex_id = new_vertex[1] * grid_size + new_vertex[0];


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


pub fn build_triangle_errors_vec(heightmap: &HeightMapU16) -> Vec::<f32> {
    assert_valid_rtin_heightmap(heightmap);


    let side = heightmap.width();
    let grid_size = side+1;
    let number_of_triangles = side * side * 2 - 2;
    let number_of_levels = log_2(side)*2;
    let last_level = number_of_levels - 1;

    let last_level_index_start = get_index_level_start(last_level);
    
    let mut errors_vec = Vec::new();
    errors_vec.resize( (grid_size*grid_size) as usize, 0.0f32);

    for triangle_index in (0..number_of_triangles).rev() {

        let triangle_bin_id = index_to_bin_id(triangle_index);

        let midpoint =
            pixel_coords_for_triangle_mid_point(triangle_bin_id, grid_size);

        let triangle_coords = get_triangle_coords(triangle_bin_id, grid_size);
        let h0 = sample_heightmap_height_corner_mean(heightmap, triangle_coords.0);
        let h1 = sample_heightmap_height_corner_mean(heightmap, triangle_coords.1);
        let midpoint_interpolated = (h1+h0)/2.0;
        let midpoint_height = sample_heightmap_height_corner_mean(heightmap, midpoint);

        let this_triangle_error = (midpoint_interpolated - midpoint_height).abs();

        let this_triangle_mid_point_error_vec_index = triangle_errors_vec_index(
            triangle_bin_id, grid_size);

        // println!("Processing triangle {:b} of coords {:?} with error index {}",
        //      triangle_bin_id, triangle_coords, this_triangle_mid_point_error_vec_index);

        if triangle_index >= last_level_index_start {
            errors_vec[this_triangle_mid_point_error_vec_index] = this_triangle_error;
        } else {
            let (right_child_bin_id, left_child_bin_id) = 
                get_triangle_children_bin_ids(triangle_bin_id);

            let right_errors_vec_index = triangle_errors_vec_index(
                right_child_bin_id, grid_size);
            let left_errors_vec_index = triangle_errors_vec_index(
                left_child_bin_id, grid_size);
            

            let prev_error = errors_vec[this_triangle_mid_point_error_vec_index];
            let right_error = errors_vec[right_errors_vec_index];
            let left_error = errors_vec[left_errors_vec_index];

            errors_vec[this_triangle_mid_point_error_vec_index] = 
                prev_error.max(left_error).max(right_error).max(this_triangle_error);
        }
       
    }

    errors_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_triangle_error_vec() {
        let heightmap_data = vec![
            0u16,     256u16,
            256u16,  1024u16 
        ];

        let heightmap  = 
            HeightMapU16::from_vec(2, 2, heightmap_data).unwrap();

        let error_vec = build_triangle_errors_vec(&heightmap);

        assert_eq!(error_vec, 
         vec![0.0, 0.1, 0.3, 0.4, 0.5, 0.6]);
    }

}