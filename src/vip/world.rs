use bitvec::array::BitArray;
use bitvec::prelude::Lsb0;

use crate::util::sign_extend_16;

pub struct World {
    /// Encapsulates LON and RON (left/right display on)
    pub display_state: WorldDisplayState,

    /// BGM Background modification
    pub background_type: BackgroundType,

    /// SCX Screen X size
    ///
    /// Raise 2 to this power to get the width of the world's background in the background maps.
    pub screen_x_size: u8,
    /// SCY Screen Y size
    ///
    /// Raise 2 to this power to get the height of the world's background in the background maps.
    pub screen_y_size: u8,

    /// If set, characters outside of the background bounds will use the `overplane_character_index`.
    /// If unset, the background will repeat infinitely.
    pub overplane: bool,

    /// Marks the previous world as the final world.
    pub end: bool,

    /// BG The index of the first background map used in the background.
    pub map_base_index: u8,

    /// GX The signed horizontal coordinate of the left edge of the world relative to the left of the image
    pub background_x_destination: i16,
    /// GP The signed horizontal parallax offset applied to the world's horizontal coordinate
    pub background_parallax_destination: i16,
    /// GY The signed vertical coordinate of the top edge of the world relative to the top of the image
    pub background_y_destination: i16,
    /// MX The signed horizontal source coordinate of the pixel in the world's background, to be displayed in the
    /// top left corner of the world, relative to the top left corner of the background.
    pub background_x_source: i16,
    /// MP The signed horizontal parallax offset applied to the background's source coordinate.
    pub background_parallax_source: i16,
    /// MY The signed vertical source coordinate of the pixel in the world's background, to be displayed in the
    /// top left corner of the world, relative to the top left corner of the background.
    pub background_y_source: i16,

    /// W The width (in pixels) of the world, minus 1.
    ///
    /// The `background_type` changes the interpretation of this value.
    pub window_width: u16,
    /// H The height (in pixels) of the world, minus 1.
    pub window_height: u16,

    /// Offset into memory for additional parameters based on `BackgroundType`
    pub param_base: usize,

    /// When `overplane` is set, this character is used for overflow characters.
    ///
    /// The address will be 0x2_0000 + `overplane_character_index` * 2
    pub overplane_character_index: u16,
}

#[derive(PartialEq)]
pub enum WorldDisplayState {
    Left,
    Right,
    Both,
    Dummy,
}

pub enum BackgroundType {
    Normal,
    HBias,
    Affine,
    Obj,
}

impl World {
    pub fn parse(halfwords: &[u16]) -> Self {
        assert!(halfwords.len() == 11);

        let value = halfwords[0];
        let array = BitArray::<_, Lsb0>::new([value]);

        let map_base_index = (value & 0xF) as u8;
        let end = *array.get(6).unwrap();
        let overplane = *array.get(7).unwrap();

        let screen_y_size = ((value >> 8) & 0x3) as u8;
        let screen_x_size = ((value >> 10) & 0x3) as u8;
        let background_type = (value >> 12) & 0x3;

        let left_display_on = value & 0x4000 != 0;
        let right_display_on = value & 0x8000 != 0;

        let display_state = match (left_display_on, right_display_on) {
            (true, true) => WorldDisplayState::Both,
            (true, false) => WorldDisplayState::Left,
            (false, true) => WorldDisplayState::Right,
            (false, false) => WorldDisplayState::Dummy,
        };

        let background_type = match background_type {
            0 => BackgroundType::Normal,
            1 => BackgroundType::HBias,
            2 => BackgroundType::Affine,
            3 => BackgroundType::Obj,
            _ => unreachable!(),
        };

        // Masking is not required as it is shifted out by the sign extension
        let background_x_destination = sign_extend_16(halfwords[1], 10);
        let background_parallax_destination = sign_extend_16(halfwords[2], 10);
        let background_y_destination = halfwords[3] as i16;

        let background_x_source = sign_extend_16(halfwords[4], 13);
        let background_parallax_source = sign_extend_16(halfwords[5], 15);
        let background_y_source = sign_extend_16(halfwords[6], 13);

        let window_width = halfwords[7] & 0x1FFF;
        let window_height = halfwords[8];

        let param_base = halfwords[9] as usize;
        let overplane_character_index = halfwords[10];

        World {
            display_state,
            background_type,
            screen_x_size,
            screen_y_size,
            overplane,
            end,
            map_base_index,
            background_x_destination,
            background_parallax_destination,
            background_y_destination,
            background_x_source,
            background_parallax_source,
            background_y_source,
            window_width,
            window_height,
            param_base,
            overplane_character_index,
        }
    }
}
