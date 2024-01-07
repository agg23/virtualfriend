use crate::{
    constants::DISPLAY_WIDTH,
    vip::{util::RenderState, vram::VRAM, world::World},
};

use super::shared::draw_background_pixel;

struct AffineElement {
    /// 13.3 fixed point signed horizontal source coordinate
    background_x_source: i16,
    /// Parallax offset
    background_parallax_source: i16,
    /// 13.3 fixed point signed vertical source coordinate
    background_y_source: i16,

    /// 7.9 fixed point signed horizontal source offset from the previous pixel in this row
    background_x_direction: i16,
    /// 7.9 fixed point signed vertical source offset from the previous pixel in this row
    background_y_direction: i16,
}

impl AffineElement {
    pub fn parse(halfwords: &[u16]) -> Self {
        assert!(halfwords.len() == 5);

        let background_x_source = halfwords[0] as i16;
        let background_parallax_source = halfwords[1] as i16;
        let background_y_source = halfwords[2] as i16;

        let background_x_direction = halfwords[3] as i16;
        let background_y_direction = halfwords[4] as i16;

        AffineElement {
            background_x_source,
            background_parallax_source,
            background_y_source,
            background_x_direction,
            background_y_direction,
        }
    }
}

pub fn render_affine_background(
    vram: &mut VRAM,
    state: &RenderState,
    world: &World,
    left_eye: bool,
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

    let window_width = world.window_width as u32 + 1;
    let window_height = world.window_height as u32 + 1;

    for window_y in 0..window_height {
        // Iterate over all possible rows. Find eligible rows for this drawing group
        let pixel_y = window_y.wrapping_add(world.background_y_destination as u32);

        if pixel_y < block_start_y || pixel_y >= block_start_y + 8 {
            continue;
        }

        // Affine element for each row. Each affine element is 16 bytes
        let param_address = 0x2_0000 + world.param_base * 2 + window_y as usize * 16;
        let param_halfword_address = param_address >> 1;
        let halfwords = vram.slice_mut(param_halfword_address, param_halfword_address + 5);

        let affine_element = AffineElement::parse(halfwords);

        let affine_parallax = if left_eye {
            if affine_element.background_parallax_source < 0 {
                // Shift is on the left eye
                // Subtract negative value and convert value to u32
                0u32.wrapping_sub(affine_element.background_parallax_source as u32)
            } else {
                0
            }
        } else {
            if affine_element.background_parallax_source >= 0 {
                // Shift is on the right eye
                affine_element.background_parallax_source as u32
            } else {
                0
            }
        };

        for window_x in 0..window_width {
            // Iterate over all possible columns. Make sure to stay within display bounds
            let pixel_x = window_x.wrapping_add(parallax_x as u32);

            if pixel_x as usize >= DISPLAY_WIDTH {
                continue;
            }

            let parallaxed_window_x = window_x.wrapping_add(affine_parallax);

            // Convert 13.3 floating point to 23.9
            let (background_x_source, background_y_source) = (
                (affine_element.background_x_source as i32) << 6,
                (affine_element.background_y_source as i32) << 6,
            );

            // Perform offset with like 23.9 and 32.0 (integer) values
            let background_x = background_x_source
                + (affine_element.background_x_direction as i32) * (parallaxed_window_x as i32);
            // For some reason the parallax multiplier is applied to y as well
            let background_y = background_y_source
                + (affine_element.background_y_direction as i32) * (parallaxed_window_x as i32);

            // Scale from 23.9 back to 32.0
            let background_x = (background_x >> 9) as u32;
            let background_y = (background_y >> 9) as u32;

            draw_background_pixel(
                vram,
                state,
                world,
                left_eye,
                pixel_x,
                pixel_y,
                background_x,
                background_y,
            );
        }
    }
}
