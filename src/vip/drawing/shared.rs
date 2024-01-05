use crate::{
    constants::FRAMEBUFFER_HEIGHT,
    vip::{
        util::{framebuffer_address_at_side, PaletteRegister, RenderState},
        vram::VRAM,
        world::World,
    },
};

pub fn draw_background_pixel(
    vram: &mut VRAM,
    state: &RenderState,
    world: &World,
    left_eye: bool,
    x: u32,
    y: u32,
    background_x: u32,
    background_y: u32,
) {
    // TODO: Move this out of pixel draw method. Unnecessary duplication of work
    let screen_x_size = 1 << world.screen_x_size;
    let screen_y_size = 1 << world.screen_y_size;

    // Size of all of the background tiles together (they're 512x512 pixels)
    let total_background_width = screen_x_size * 512;
    let total_background_height = screen_y_size * 512;

    if world.overplane
        && (background_x >= total_background_height || background_y >= total_background_width)
    {
        // Overplane is enabled and our background tile is outside of the total background bounds
        // Draw overplane character
        todo!();
    } else {
        // Draw normal pixel
        // Get active background map (AND to limit to the available range)
        let active_background_map_x = (background_x / 512) & (screen_x_size - 1);
        let active_background_map_y = (background_y / 512) & (screen_y_size - 1);

        // Each background is 0x2000 bytes
        let background_base_offset_address = 0x2_0000 + (world.map_base_index as u32) * 0x2000;
        let background_offset_address = background_base_offset_address
            + (active_background_map_y * screen_x_size + active_background_map_x) * 0x2000;

        // Limit our pixel positions to be within the background
        let background_x = background_x & 0x1FF;
        let background_y = background_y & 0x1FF;

        // Get X/Y position of the character block
        let character_x = background_x / 8;
        let character_y = background_y / 8;

        // Get pixel offset in the given block
        let background_pixel_offset_x = background_x & 0x7;
        let background_pixel_offset_y = background_y & 0x7;

        // There are 512/8 = 64 blocks in a row in a background
        let character_address = background_offset_address + (character_y * 64 + character_x) * 2;

        // Get background map character block info
        let character_halfword = vram.get_u16(character_address as usize);

        let character_index = character_halfword & 0x7FF;
        let vertical_flip = character_halfword & 0x1000 != 0;
        let horizontal_flip = character_halfword & 0x2000 != 0;
        let palette = character_halfword >> 14;

        // TODO: Handle OBJ palettes
        let palette = match palette {
            0 => state.background_palette_control0,
            1 => state.background_palette_control1,
            2 => state.background_palette_control2,
            _ => state.background_palette_control3,
        };

        draw_character_pixel(
            vram,
            state,
            left_eye,
            x,
            y,
            background_pixel_offset_x,
            background_pixel_offset_y,
            character_index,
            palette,
            horizontal_flip,
            vertical_flip,
        );
    }
}

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
