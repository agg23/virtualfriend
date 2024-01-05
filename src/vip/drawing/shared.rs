use crate::{
    constants::FRAMEBUFFER_HEIGHT,
    vip::{
        util::{framebuffer_address_at_side, PaletteRegister, RenderState},
        vram::VRAM,
    },
};

pub fn draw_character_pixel(
    vram: &mut VRAM,
    state: &RenderState,
    left_eye: bool,
    x: u32,
    y: u32,
    character_offset_x: u32,
    character_offset_y: u32,
    character_index: u16,
    palette: PaletteRegister,
    horizontal_flip: bool,
    vertical_flip: bool,
) {
    // Flip pixel position, if necessary
    let character_offset_x = if horizontal_flip {
        7 - character_offset_x
    } else {
        character_offset_x
    };
    let character_offset_y = if vertical_flip {
        7 - character_offset_y
    } else {
        character_offset_y
    };

    // Index into character blocks. We don't use virtual addresses here so we can
    // directly access VRAM.
    // 8 rows per block. 2 bytes per row = 16 per character
    let character_index = character_index as u32;
    let local_index = character_index & 0x1FF;

    let character_address = match character_index {
        0..=0x1FF => 0x6000 + character_index * 16,
        0x200..=0x3FF => 0xE000 + local_index * 16,
        0x400..=0x5FF => 0x1_6000 + local_index * 16,
        _ => 0x1_E000 + local_index * 16,
    };

    // Index to the correct row
    let character_address = character_address + character_offset_y * 2;

    // TODO: This can be optimized
    let row_halfword = vram.get_u16(character_address as usize);

    // Extract pixel
    let pixel_palette_index = (row_halfword >> (character_offset_x * 2)) & 0x3;

    let pixel = match pixel_palette_index {
        // No need to draw. Background "blank" pixel has already been written to FB
        0 => return,
        1 => palette.character1,
        2 => palette.character2,
        _ => palette.character3,
    };

    // Write to framebuffer
    // Pixels are stored in the framebuffer in columns, rather than in rows
    let framebuffer_offset = x * FRAMEBUFFER_HEIGHT as u32 + y;
    // Each pixel is 2 bits, so find the right byte for this pixel
    let framebuffer_byte_offset = framebuffer_offset / 4;
    let pixel_shift = (y & 0x3) * 2;

    let framebuffer_address = framebuffer_address_at_side(left_eye, state.drawing_framebuffer_1)
        + framebuffer_byte_offset as usize;

    let removal_mask = !(0x3 << pixel_shift);

    let existing_byte = vram.get_u8(framebuffer_address);
    let byte = (existing_byte & removal_mask) | (pixel << pixel_shift);
    vram.set_u8(framebuffer_address, byte);
}
