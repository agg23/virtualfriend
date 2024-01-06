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
