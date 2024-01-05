use crate::{
    constants::DISPLAY_WIDTH,
    util::sign_extend_16,
    vip::{util::RenderState, vram::VRAM, world::World},
};

use super::shared::draw_character_pixel;

pub fn render_normal_or_hbias_background(
    vram: &mut VRAM,
    state: &RenderState,
    world: &World,
    left_eye: bool,
    is_hbias: bool,
    block_start_y: u32,
) {
    // Calculate start coordinate offset using world parallax
    let parallax_x = if left_eye {
        world
            .background_x_destination
            .wrapping_sub(world.background_parallax_destination)
    } else {
        world
            .background_x_destination
            .wrapping_add(world.background_parallax_destination)
    };

    let world_height = world.window_height + 1;

    // Loop over y pixels in range of this block
    let lower_bound = block_start_y as i16 + world.background_y_destination;

    for y in lower_bound..lower_bound + 8 {
        // For each row in the block
        // Get window start Y position
        let window_y = y.wrapping_sub(world.background_y_destination);

        let line_offset = if is_hbias {
            // HBias has two additional parameters of 16 bits per row
            let base_address = 0x20000 + world.param_base * 2 + window_y as usize * 4;

            let address = if left_eye {
                base_address
            } else {
                // "The VIP appears to determine the address of HOFSTR by OR'ing the address of HOFSTL with 2.
                // If the Param Base attribute in the world is not divisibe by 2, this will result in HOFSTL being
                // used for both the left and right images, and HOFSTR will not be accessed."
                base_address | 2
            };

            sign_extend_16(vram.get_u16(address), 13)
        } else {
            0
        };

        let lower_bound = 0 + parallax_x;

        for x in lower_bound..lower_bound + world.window_width as i16 + 1 {
            // Loop over all columns in the row
            if x >= DISPLAY_WIDTH as i16 {
                continue;
            }
            let window_x = x - parallax_x;

            let background_x = window_x.wrapping_add(world.background_x_destination);
            // Offset line from h-bias
            let background_x = background_x.wrapping_add(line_offset);

            let background_x = if left_eye {
                background_x.wrapping_sub(world.background_parallax_source)
            } else {
                background_x.wrapping_add(world.background_parallax_source)
            };

            let background_y = window_y.wrapping_add(world.background_y_source);

            draw_background_pixel(
                vram,
                state,
                world,
                left_eye,
                x as u32,
                y as u32,
                background_x as u32,
                background_y as u32,
            );
        }
    }
}

fn draw_background_pixel(
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
        let overplane_character_address = 0x2_0000 + (world.overplane_character_index as usize) * 2;
        let overplane_character_halfword = vram.get_u16(overplane_character_address);

        extract_and_draw_character_entry_pixel(
            vram,
            state,
            overplane_character_halfword,
            left_eye,
            x,
            y,
            background_x,
            background_y,
        )
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

        extract_and_draw_character_entry_pixel(
            vram,
            state,
            character_halfword,
            left_eye,
            x,
            y,
            background_pixel_offset_x,
            background_pixel_offset_y,
        );
    }
}

fn extract_and_draw_character_entry_pixel(
    vram: &mut VRAM,
    state: &RenderState,
    character_halfword: u16,
    left_eye: bool,
    x: u32,
    y: u32,
    character_offset_x: u32,
    character_offset_y: u32,
) {
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
        character_offset_x,
        character_offset_y,
        character_index,
        palette,
        horizontal_flip,
        vertical_flip,
    );
}
