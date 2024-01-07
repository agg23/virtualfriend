use crate::{
    constants::DISPLAY_WIDTH,
    util::sign_extend_16,
    vip::{util::RenderState, vram::VRAM, world::World},
};

use super::shared::draw_background_pixel;

pub fn render_normal_or_hbias_background(
    vram: &mut VRAM,
    state: &RenderState,
    world: &World,
    left_eye: bool,
    is_hbias: bool,
    block_start_y: u32,
) {
    // Calculate start coordinate offset using world parallax
    // Depth of the world in the image
    let parallax_x = if left_eye {
        world
            .background_x_destination
            .wrapping_sub(world.background_parallax_destination)
    } else {
        world
            .background_x_destination
            .wrapping_add(world.background_parallax_destination)
    };

    let world_height = world.window_height as u32 + 1;
    let world_width = world.window_width as u32 + 1;

    for window_y in 0..world_height {
        // For each row in the block
        // Get the final pixel height corresponding to that row, after background shift
        let pixel_y = window_y.wrapping_add(world.background_y_destination as u32);

        // TODO: I don't like this. I would rather loop over exactly the elements we need
        if pixel_y < block_start_y || pixel_y >= block_start_y + 8 {
            // This pixel is not currently being rendered, skip
            continue;
        }

        let line_offset = if is_hbias {
            // HBias has two additional parameters of 16 bits per row
            let base_address = 0x2_0000 + world.param_base * 2 + window_y as usize * 4;

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

        for window_x in 0..world_width {
            // Loop over all columns in the row
            // Get the final pixel column corresponding to that world column, after background shift
            let pixel_x = window_x.wrapping_add(parallax_x as u32);

            // TODO: I don't like this. I would rather loop over exactly the elements we need
            if pixel_x >= DISPLAY_WIDTH as u32 {
                continue;
            }

            let background_x = window_x.wrapping_add(world.background_x_source as u32);
            // Offset line from h-bias
            let background_x = background_x.wrapping_add(line_offset as u32);

            // Add depth of the background within the world
            let background_x = if left_eye {
                background_x.wrapping_sub(world.background_parallax_source as u32)
            } else {
                background_x.wrapping_add(world.background_parallax_source as u32)
            };

            let background_y = window_y.wrapping_add(world.background_y_source as u32);

            draw_background_pixel(
                vram,
                state,
                world,
                left_eye,
                pixel_x,
                pixel_y,
                background_x as u32,
                background_y as u32,
            );
        }
    }
}
