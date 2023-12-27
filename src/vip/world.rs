use bitvec::array::BitArray;
use bitvec::prelude::Lsb0;

use crate::util::sign_extend_16;

pub struct World {
    pub display_state: WorldDisplayState,

    /// BGM Background modification
    pub background_type: BackgroundType,

    pub screen_x_size: u8,
    pub screen_y_size: u8,

    pub overplane: bool,

    /// Marks the previous world as the final world
    pub end: bool,

    pub map_base_index: u8,

    pub background_x_destination: i16,
    pub background_parallax_destination: i16,
    pub background_y_destination: i16,
    pub background_x_source: i16,
    pub background_parallax_source: i16,
    pub background_y_source: i16,

    pub window_width: u16,
    pub window_height: u16,

    /// Offset into memory for additional parameters based on `BackgroundType`
    pub param_base: u16,

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
    pub fn parse(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 10);

        let value = bytes[0];
        let array = BitArray::<_, Lsb0>::new([value]);

        let map_base_index = value & 0xF;
        let end = *array.get(6).unwrap();
        let overplane = *array.get(7).unwrap();

        let value = bytes[1];

        let screen_y_size = value & 0x3;
        let screen_x_size = (value >> 2) & 0x3;
        let background_type = (value >> 4) & 0x3;

        let left_display_on = value & 0x40 != 0;
        let right_display_on = value & 0x80 != 0;

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

        let value = ((bytes[3] as u16) << 8) | (bytes[2] as u16);
        // Masking is not required as it is shifted out by the sign extension
        let background_x_destination = sign_extend_16(value, 10);

        let value = ((bytes[5] as u16) << 8) | (bytes[4] as u16);
        let background_parallax_destination = sign_extend_16(value, 10);

        let background_y_destination = (((bytes[7] as u16) << 8) | (bytes[6] as u16)) as i16;

        let value = ((bytes[7] as u16) << 8) | (bytes[6] as u16);
        let background_x_source = sign_extend_16(value, 13);

        let value = ((bytes[9] as u16) << 8) | (bytes[8] as u16);
        let background_parallax_source = sign_extend_16(value, 15);

        let value = ((bytes[11] as u16) << 8) | (bytes[10] as u16);
        let background_y_source = sign_extend_16(value, 13);

        let window_width = (((bytes[13] as u16) << 8) | (bytes[12] as u16)) & 0x1FFF;
        let window_height = ((bytes[15] as u16) << 8) | (bytes[14] as u16);

        let param_base = ((bytes[17] as u16) << 8) | (bytes[16] as u16);
        let overplane_character_index = ((bytes[19] as u16) << 8) | (bytes[18] as u16);

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
