use bitintr::Lzcnt;

extern crate nalgebra as na;
use na::Vector2;

pub type BinId = u32;

pub type Vec2u32 = Vector2<u32>;

pub type TriangleU32 = (Vec2u32, Vec2u32, Vec2u32);

/// get the corresponding index of the first triangle 
/// of a given level
///
/// ```
/// # use bevy_terrain::rtin::*;
/// assert_eq!(get_index_level_start(0), 0b0);
/// assert_eq!(get_index_level_start(1), 0b10);
/// assert_eq!(get_index_level_start(1), 2);
/// assert_eq!(get_index_level_start(2), 0b110);
/// assert_eq!(get_index_level_start(2), 6);
/// assert_eq!(get_index_level_start(3), 0b1110);
/// assert_eq!(get_index_level_start(3), 14);
/// ```
pub fn get_index_level_start(level: u32) -> u32 {
    ( (2 << level) - 1 ) & (!1u32)
}

/// returns the relative triangle index within its level
///
/// ```
/// # use bevy_terrain::rtin::*;
/// assert_eq!(bin_id_to_index_in_level(0b10), 0);
/// assert_eq!(bin_id_to_index_in_level(0b11), 1);
/// assert_eq!(bin_id_to_index_in_level(0b100), 0);
/// assert_eq!(bin_id_to_index_in_level(0b101), 1);
/// assert_eq!(bin_id_to_index_in_level(0b110), 2);
/// assert_eq!(bin_id_to_index_in_level(0b111), 3);
/// ```
pub fn bin_id_to_index_in_level(bin_id: u32) -> u32 {
    bin_id - (1 << (bin_id.msbscan()-1) )
}

pub trait MSBScan {

    /// returns the position of the set most significant bit.
    /// i.e. reading from left to right, the position of the 
    /// first bit that is set to 1
    /// 
    /// the position index starts from right and increases to left
    /// returns zero when no bit is set

    /// ```
    /// # use bevy_terrain::rtin::*;
    /// assert_eq!(0b0000_0000_u32.msbscan(), 0_u32);
    /// assert_eq!(0b0000_0001_u32.msbscan(), 1_u32);
    /// assert_eq!(0b0001_1001_u32.msbscan(), 5_u32);
    /// ```
    fn msbscan(self) -> Self;

}

impl MSBScan for u32 {
    
    fn msbscan(self) -> u32 {
        32 - self.lzcnt()
    }

}

/// level 0: 
///
///   +----+
///   |\   |
///   | \  |
///   |  \ |
///   |   \|
///   +----+
/// has the following bin_id:
///   10, 11
///
/// level 1:
///   +----+
///   |\  /|
///   | \/ |
///   | /\ |
///   |/  \|
///   +----+
///
/// has the following bin_id:
///   100, 101, 110, 111
///
/// ```
/// # use bevy_terrain::rtin::*;
/// assert_eq!(bin_id_to_index(0b10), 0);
/// assert_eq!(bin_id_to_index(0b11), 1);
/// assert_eq!(bin_id_to_index(0b100), 2);
/// assert_eq!(bin_id_to_index(0b111), 5);
/// assert_eq!(bin_id_to_index(0b1011), 9);
/// ```
///
pub fn bin_id_to_index(bin_id: u32) -> u32 {
    let level = bin_id_to_level(bin_id);
    let index_level_start = get_index_level_start(level);
    let index_in_level = bin_id_to_index_in_level(bin_id);

    index_level_start + index_in_level
}


pub fn get_triangle_children_indices(bin_id: u32) -> (u32, u32) {
    let (right_index, left_index) = 
        get_triangle_children_bin_ids(bin_id);
    (bin_id_to_index(right_index), bin_id_to_index(left_index))
}

/// ```
/// # use bevy_terrain::rtin::*;
/// assert_eq!(get_triangle_children_bin_ids(0b10), (0b100, 0b110));
/// assert_eq!(get_triangle_children_bin_ids(0b1010), (0b10010, 0b11010));
/// ```
pub fn get_triangle_children_bin_ids(bin_id: u32) -> (u32, u32) {
    let level = bin_id_to_level(bin_id);
    let right_bin_id = 
        bin_id + (1 << (level+2) ) - (1 << (level+1) );
    let left_bin_id = 
        bin_id + (1 << (level+2) );
    (right_bin_id, left_bin_id)
}

/// convert the binary id of the triangle to the index
///
/// ```
/// # use bevy_terrain::rtin::*;
/// assert_eq!(index_to_bin_id(0), 0b10);
/// assert_eq!(index_to_bin_id(1), 0b11);
/// assert_eq!(index_to_bin_id(2), 0b100);
/// assert_eq!(index_to_bin_id(5), 0b111);
/// assert_eq!(index_to_bin_id(9), 0b1011);
/// ```
pub fn index_to_bin_id(index: u32) -> u32 {
    let mut level = 0;
    let mut index_level_start = 0;

    for i in 0..32 {
        let new_index_level_start = get_index_level_start(i);
        if index >= new_index_level_start {
            level = i;
            index_level_start = new_index_level_start;
        } else {
            break;
        }
    }

    ( 1 << (level+1) ) + (index - index_level_start)
}

pub fn bin_id_to_level(bin_id: u32) -> u32 {
    bin_id.msbscan() - 2
}

/// Get the rect triangle basis middle point coordinate
// /
// /      C  +        
// /        / \       
// /       /   \      
// /      /     \     
// /     /       \    
// /  A +----o----+ B 
///
/// ```
/// # use bevy_terrain::rtin::*;
/// let n_tiles = 4;
/// assert_eq!(
///    pixel_coords_for_triangle_mid_point(0b10_1110, n_tiles),  
///    Vec2u32::new(1, 2) );
/// ```
///
pub fn pixel_coords_for_triangle_mid_point(bin_id: u32, grid_size: u32) -> Vec2u32 {
    let triangle_coords = get_triangle_coords(bin_id, grid_size);
    let mid_point = (triangle_coords.0 + triangle_coords.1) / 2;

    Vec2u32::new(mid_point[0], mid_point[1])
}

/// 
/// vertex C always on the right-angle corner
/// a, b, c ordering always clockwise
/// 
// / A +----+ C
// /    \   |
// /     \  |
// /      \ |
// /       \|
// /        + B
// /
// / B +
// /   |\   
// /   | \  
// /   |  \ 
// /   |   \
// / C +----+ A
// /
// /        + A
// /       /|
// /      / |
// /     /  |
// /    /   |
// / B +----+ C
// /
// / C +----+ B
// /   |   /
// /   |  / 
// /   | /  
// /   |/   
// / A + 
// /
// /   parent triangle is split into left and right triangle 
// /
// /      C  +                   + A     B +
// /        /.\                 /|         |\ 
// /       / . \               / |         | \
// /      /  .  \      =>     /  |         |  \
// /     /   .   \           /   |         |   \
// /  A +____.____+ B     B +----+ C     C +____+ A
///
/// ```
/// # use bevy_terrain::rtin::*;
/// let n_tiles = 4;
/// assert_eq!(get_triangle_coords(0b11, n_tiles),  
///    (Vec2u32::new(0, 0), Vec2u32::new(4, 4), Vec2u32::new(4, 0)) );
/// assert_eq!(get_triangle_coords(0b110, n_tiles),  
///    (Vec2u32::new(0, 4), Vec2u32::new(4, 4), Vec2u32::new(2, 2)) );
/// assert_eq!(get_triangle_coords(0b1_1110, n_tiles),  
///    (Vec2u32::new(2, 4), Vec2u32::new(2, 2), Vec2u32::new(1, 3)) );
/// ```
///
pub fn get_triangle_coords(bin_id: u32, grid_size: u32) -> TriangleU32 {
    let mut a = Vec2u32::new(0, 0);
    let mut b = Vec2u32::new(0, 0);
    let mut c = Vec2u32::new(0, 0);


    for step in bin_id_to_partition_steps(bin_id) {
        match step {
            PartitionStep::TopRight => {
                // north east right-angle corner
                a[0] = 0; 
                a[1] = 0; 
                b[0] = grid_size-1; 
                b[1] = grid_size-1; 
                c[0] = grid_size-1; 
                c[1] = 0; 
            }
            PartitionStep::BottomLeft => {
                // north east right-angle corner
                a[0] = grid_size-1; 
                a[1] = grid_size-1; 
                b[0] = 0; 
                b[1] = 0; 
                c[0] = 0; 
                c[1] = grid_size-1; 

            }
            PartitionStep::Left => {
                let (new_a, new_b, new_c) = (
                    c, 
                    a, 
                    (a+b) / 2
                );
                a = new_a;
                b = new_b;
                c = new_c;
            }
            PartitionStep::Right => {
                let (new_a, new_b, new_c) = (
                    b, 
                    c, 
                    (a+b) / 2
                );
                a = new_a;
                b = new_b;
                c = new_c;
            }
        }
    }

    (a, b, c)
}

#[derive(Eq, PartialEq, Debug)]
pub enum PartitionStep {
    TopRight,
    BottomLeft,
    Left, 
    Right
}  

///
///
/// ```
/// # use bevy_terrain::rtin::*;
/// assert_eq!(bin_id_to_partition_steps(0b10), [PartitionStep::BottomLeft]);
/// assert_eq!(bin_id_to_partition_steps(0b11), [PartitionStep::TopRight]);
/// assert_eq!(bin_id_to_partition_steps(0b110), 
///   [PartitionStep::BottomLeft, PartitionStep::Left]);
/// assert_eq!(bin_id_to_partition_steps(0b10110), 
///   [PartitionStep::BottomLeft, PartitionStep::Left, PartitionStep::Left,
///    PartitionStep::Right]);
/// ```
///
pub fn bin_id_to_partition_steps(bin_id: u32) -> Vec::<PartitionStep> {
    let mut steps = Vec::new();
    let triangle_level = bin_id_to_level(bin_id);

    if bin_id & 1 > 0 {
        steps.push(PartitionStep::TopRight);
    } else {
        steps.push(PartitionStep::BottomLeft);
    }

    for i in 1..(triangle_level+1) {
       if bin_id & (1 << i) > 0 {
        steps.push(PartitionStep::Left);
       } else {
        steps.push(PartitionStep::Right);
       }
    }

    steps
}