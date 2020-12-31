use bitintr::Lzcnt;

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
///
pub fn bin_id_to_index(bin_id: u32) -> u32 {
    let level = bin_id.msbscan() - 2;
    let index_level_start = get_index_level_start(level);
    let index_in_level = bin_id_to_index_in_level(bin_id);

    index_level_start + index_in_level
}
