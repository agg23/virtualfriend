use crate::{
    constants::DISPLAY_WIDTH,
    vip::{object::Object, util::RenderState, vram::VRAM},
};

use super::shared::draw_character_pixel;

pub fn render_obj_world(
    vram: &mut VRAM,
    state: &RenderState,
    left_eye: bool,
    block_start_y: u32,
    object: &Object,
) {
    if !(object.render_to_left_display && left_eye)
        && !(object.render_to_right_display && !left_eye)
    {
        // Nothing to render for this eye
        return;
    }

    let palette = match object.palette {
        0 => state.object_palette_control0,
        1 => state.object_palette_control1,
        2 => state.object_palette_control2,
        _ => state.object_palette_control3,
    };

    for offset_y in 0..8 {
        let pixel_y = (object.display_pointer_y as u32).wrapping_add(offset_y);

        if pixel_y < block_start_y || pixel_y >= block_start_y + 8 {
            // This pixel is not currently being rendered, skip
            continue;
        }

        for offset_x in 0..8 {
            let pixel_x = (object.display_pointer_x as u32).wrapping_add(offset_x);
            let pixel_x = if left_eye {
                pixel_x.wrapping_sub(object.parallax as u32)
            } else {
                pixel_x.wrapping_add(object.parallax as u32)
            };

            if pixel_x >= DISPLAY_WIDTH as u32 {
                // Out of bounds (positive or negative)
                continue;
            }

            draw_character_pixel(
                vram,
                state,
                left_eye,
                pixel_x,
                pixel_y,
                offset_x,
                offset_y,
                object.character_index,
                palette,
                object.horizontal_flip,
                object.vertical_flip,
            );
        }
    }
}
