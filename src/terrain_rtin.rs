use image::{ImageBuffer, Luma};
use bevy_render::{
    pipeline::PrimitiveTopology,
    mesh::{Mesh, VertexAttributeValues, Indices},
};
use std::{cmp::max, vec::Vec};
use bevy::prelude::*;
use anyhow::Result;

use crate::rtin::{TriangleU32, Vec2u32, get_index_level_start, get_triangle_children_indices, get_triangle_coords, index_to_bin_id, pixel_coords_for_triangle_mid_point};

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

pub fn rtin_build_terrain_from_heightmap_process_triangle(
    heightmap: &HeightMapU16, 
    errors_vec: &Vec::<f32>,
    triangles: &mut Vec::<Trianglef32>, 
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
        let n_tiles = heightmap.width();
        let triangle_coords = get_triangle_coords(triangle_bin_id, n_tiles);

        let triangle  = triangleu32_to_trianglef32(triangle_coords);
        triangles.push(triangle);
    } else {
        rtin_build_terrain_from_heightmap_process_triangle(
            heightmap, errors_vec, triangles, left_child_index, error_threshold);
        rtin_build_terrain_from_heightmap_process_triangle(
            heightmap, errors_vec, triangles, right_child_index, error_threshold);
    }
}

pub fn rtin_terrain_example() -> Mesh {
    let error_threshold = 0f32;
    let filename = "terrain.png";

    let mesh = rtin_load_terrain_bitmap(filename, error_threshold);

    mesh.unwrap()
}

pub fn rtin_load_terrain_bitmap(filename: &str, error_threshold: f32) -> Result<Mesh> {
    let terrain_bitmap = image::open(filename)?;
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    let heightmap = terrain_bitmap.as_luma16().unwrap();

    let triangles = rtin_build_terrain_from_heightmap(heightmap, error_threshold);

    println!("TRIANGLE LIST START");
    for (triangle_index, &triangle) in triangles.iter().enumerate() {
        println!("{}: {:?}", triangle_index, triangle);
    }
    println!("TRIANGLE LIST END");

    let mut vertices : Vec::<[f32; 3]> = Vec::new();
    let mut normals : Vec::<[f32; 3]> = Vec::new();
    let mut indices : Vec::<u32> = Vec::new();

    vertices.resize(triangles.len() * 3, [0f32, 0f32, 0f32]);

    for (triangle_index, &triangle) in triangles.iter().enumerate() {
       vertices[triangle_index*3+0] = [
           triangle.0[0], triangle.0[1], triangle.0[2]]; 
       vertices[triangle_index*3+1] = [
           triangle.1[0], triangle.1[1], triangle.1[2]]; 
       vertices[triangle_index*3+2] = [
           triangle.2[0], triangle.2[1], triangle.2[2]]; 

       indices.push( (triangle_index*3+0) as u32 );
       indices.push( (triangle_index*3+1) as u32 );
       indices.push( (triangle_index*3+1) as u32 );
       indices.push( (triangle_index*3+2) as u32 );
       indices.push( (triangle_index*3+2) as u32 );
       indices.push( (triangle_index*3+0) as u32 );
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

pub fn rtin_build_terrain_from_heightmap(
    heightmap: &HeightMapU16, error_threshold: f32) -> Vec::<Trianglef32> {

    let mut triangles = Vec::<Trianglef32>::new();
    let errors_vec = build_triangle_error_vec(heightmap);

    rtin_build_terrain_from_heightmap_process_triangle(
        heightmap, &errors_vec, &mut triangles, 
        0, error_threshold);
    rtin_build_terrain_from_heightmap_process_triangle(
        heightmap, &errors_vec, &mut triangles, 
        1, error_threshold);

    triangles
}

pub fn build_triangle_error_vec(heightmap: &HeightMapU16) -> Vec::<f32> {
    assert_valid_rtin_heightmap(heightmap);


    let side = heightmap.width();
    let number_of_triangles = side * side * 2 - 2;
    let number_of_levels = side;
    let last_level = number_of_levels - 1;
    let last_level_index_start = get_index_level_start(last_level);

    let mut error_vec = Vec::new();
    error_vec.resize(number_of_triangles as usize, 0.0f32);

    for triangle_index in (0..number_of_triangles).rev() {

        let triangle_bin_id = index_to_bin_id(triangle_index);
        let triangle_pixel_coordinates =
            pixel_coords_for_triangle_mid_point(triangle_bin_id, side);

        // println!("\n\ndoing index={} bin_id={:b} triangle_pixel_coordinate={}", triangle_index, triangle_bin_id,
        //     triangle_pixel_coordinates);

        // println!("triangle coordinates={:?}", get_triangle_coords(triangle_bin_id, side));

        assert_coordinate_is_within_heightmap(heightmap, triangle_pixel_coordinates);

        let this_triangle_error = heightmap.get_pixel(
                triangle_pixel_coordinates[0], triangle_pixel_coordinates[1]).0[0] as f32 / std::u16::MAX as f32;

        // println!("this_error={}",
        //     this_triangle_error);

        if triangle_index >= last_level_index_start {
            error_vec[triangle_index as usize] = this_triangle_error;
        } else {
            let (right_child_index, left_child_index) = 
                get_triangle_children_indices(triangle_bin_id);
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