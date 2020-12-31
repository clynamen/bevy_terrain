use bevy::{asset::filesystem_watcher_system, math::{Vec2, vec2}};
use bitintr::Lzcnt;

extern crate nalgebra as na;
use na::Vector2;

type Vec2u32 = Vector2<u32>;

type TriangleU32 = (Vec2u32, Vec2u32, Vec2u32);

/// get the corresponding index of the first triangle 
/// of a given level
///
/// ```
/// # use bevy_terrain::RTIN::*;
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
/// # use bevy_terrain::RTIN::*;
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
    /// # use bevy_terrain::RTIN::*;
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
/// # use bevy_terrain::RTIN::*;
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

pub fn bin_id_to_level(bin_id: u32) -> u32 {
    bin_id.msbscan() - 2
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
// /
// /  A +____.____+ B     B +----+ C     C +____+ A
///
pub fn get_triangle_coords(bin_id: u32, n_tiles: u32) -> TriangleU32 {
    let mut a = Vec2u32::new(0, 0);
    let mut b = Vec2u32::new(0, 0);
    let mut c = Vec2u32::new(0, 0);

    // north east right-angle corner
    a[0] = 0; 
    a[1] = 0; 
    b[0] = n_tiles; 
    b[1] = n_tiles; 
    c[0] = n_tiles; 
    c[1] = 0; 

    // while()

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
/// # use bevy_terrain::RTIN::*;
/// use PartitionStep;
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