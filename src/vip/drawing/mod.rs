mod affine;
mod normal_hbias;
mod obj;
mod shared;

use crate::{
    constants::{DISPLAY_WIDTH, FRAMEBUFFER_HEIGHT},
    vip::{drawing::obj::render_obj_world, object::Object},
};

use self::{affine::render_affine_background, normal_hbias::render_normal_or_hbias_background};

use super::{
    util::{framebuffer_addresses, RenderState},
    vram::VRAM,
    world::{BackgroundType, World, WorldDisplayState},
};

///
/// Draws a block (1x8), but for an entire row. This represents 1 value of `SBCOUNT`.
///
/// The actual hardware does not draw entire rows at a time, but due to lack of "racing the beam", there's no
/// real reason to break up the drawing.
pub fn draw_block_row(vram: &mut VRAM, state: &RenderState) {
    // Get drawing framebuffer
    let (left_framebuffer_address, right_framebuffer_address) =
        framebuffer_addresses(state.drawing_framebuffer_1);

    // Initialize all values to BKCOL
    let clear_pixel = ((state.last_bkcol << 6)
        | (state.last_bkcol << 4)
        | (state.last_bkcol << 2)
        | state.last_bkcol) as u16;
    let clear_pixel = (clear_pixel << 8) | clear_pixel;

    let y = (state.sbcount as u32) * 8;

    for x in 0..DISPLAY_WIDTH {
        // Overwrite every pixel in the 384x8 segment
        // Pixels are stored in the framebuffer in columns, rather than in rows
        let x_index = x * FRAMEBUFFER_HEIGHT;
        // This contains the bottom three bytes as the bit offset
        let pixel_byte_index = (x_index + y as usize) >> 2;
        vram.set_u16(left_framebuffer_address + pixel_byte_index, clear_pixel);
        vram.set_u16(right_framebuffer_address + pixel_byte_index, clear_pixel);
    }

    // Counter for total object groups
    let mut object_group_counter = 3;

    for i in (0..=31).rev() {
        // Process world
        let world_attribute_address = 0x3_D800 + 16 * 2 * i;
        // Convert byte address into halfword addresses so we can grab a slice of memory
        let world_attribute_halfword_address = world_attribute_address >> 1;
        let halfwords = vram.slice_mut(
            world_attribute_halfword_address,
            world_attribute_halfword_address + 11,
        );

        let world = World::parse(halfwords);

        if world.end {
            // We're done processing worlds
            break;
        } else if world.display_state == WorldDisplayState::Dummy {
            // Dummy world, skip
            continue;
        }

        match world.background_type {
            BackgroundType::Normal | BackgroundType::HBias => {
                let hbias = world.background_type == BackgroundType::HBias;
                render_normal_or_hbias_background(vram, state, &world, true, hbias, y);
                render_normal_or_hbias_background(vram, state, &world, false, hbias, y);
            }
            BackgroundType::Affine => {
                render_affine_background(vram, state, &world, true, y);
                render_affine_background(vram, state, &world, false, y);
            }
            BackgroundType::Obj => {
                let (mut start_obj_index, end_obj_index) = match object_group_counter {
                    // First group always starts at 0
                    0 => (0, state.object_group_end0),
                    // All other groups start at the end of the last group + 1
                    1 => (state.object_group_end0 + 1, state.object_group_end1),
                    2 => (state.object_group_end1 + 1, state.object_group_end2),
                    _ => (state.object_group_end2 + 1, state.object_group_end3),
                };

                // TODO: Unsure if we should skip this or continue
                if start_obj_index > end_obj_index {
                    start_obj_index = 0;
                }

                assert!(
                    start_obj_index <= end_obj_index,
                    "{start_obj_index} {end_obj_index}, {object_group_counter} {} {} {} {}",
                    state.object_group_end0,
                    state.object_group_end1,
                    state.object_group_end2,
                    state.object_group_end3
                );

                // Process objects in group in reverse order
                for i in (start_obj_index..end_obj_index + 1).rev() {
                    // 8 bytes per object attribute
                    let obj_group_address = 0x3_E000 + (i as usize) * 8;

                    let obj_group_halfword_address = obj_group_address >> 1;
                    let halfwords =
                        vram.slice_mut(obj_group_halfword_address, obj_group_halfword_address + 4);

                    let object = Object::parse(halfwords);

                    render_obj_world(vram, state, true, y, &object);
                    render_obj_world(vram, state, false, y, &object);
                }

                if object_group_counter == 0 {
                    // Object groups loop around
                    object_group_counter = 3;
                } else {
                    object_group_counter -= 1;
                }
            }
        }
    }
}
