use std::f32::consts::E;

use crate::{
    constants::{
        DISPLAY_HEIGHT, DISPLAY_PIXEL_LENGTH, DISPLAY_WIDTH, DRAWING_BLOCK_COUNT,
        DRAWING_BLOCK_CYCLE_COUNT, FCLK_LOW_CYCLE_OFFSET, FRAME_COMPLETE_CYCLE_OFFSET,
        LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET, LEFT_FRAME_BUFFER_CYCLE_OFFSET,
        RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET, RIGHT_FRAME_BUFFER_CYCLE_OFFSET,
    },
    util::sign_extend_16,
};

use super::world::{BackgroundType, World, WorldDisplayState};

pub struct VIP {
    /// Four sets of 512 characters (16 bytes each)
    // character_tables: [[u32; 512 * 4]; 4],

    // /// 1024 objects of 8 bytes each
    // oam: [u32; 1024 * 2],

    // background_map_and_params: [u32; 0x7600],
    current_display_clock_cycle: u32,

    // We map the entirety of VRAM due to overlapping sections
    // (upper background maps overlap with OAM and properties).
    vram: [u8; 0x40000],

    left_rendered_framebuffer: [u8; DISPLAY_PIXEL_LENGTH],
    right_rendered_framebuffer: [u8; DISPLAY_PIXEL_LENGTH],

    interrupt: VIPInterrupt,

    // Registers
    fclk: bool,

    /// XPEN
    drawing_enabled: bool,
    display_enabled: bool,
    /// Sync signals are being sent to the displays, preventing images from being displayed.
    sync_enabled: bool,

    /// Game frame control register
    ///
    /// This value +1 is the number of display frames per rendered frame
    frmcyc: u8,

    /// BG color palette control register
    ///
    /// When set, new background color is not applied until the first 8 rows of the next frame are rendered.
    bkcol: u8,
    last_bkcol: u8,

    /// The current group of 8 rows of pixels, relative to the top of the image, currently being drawn.
    sbcount: u8,

    brightness_control_reg_a: u8,
    brightness_control_reg_b: u8,
    brightness_control_reg_c: u8,

    background_palette_control0: PaletteRegister,
    background_palette_control1: PaletteRegister,
    background_palette_control2: PaletteRegister,
    background_palette_control3: PaletteRegister,

    object_palette_control0: PaletteRegister,
    object_palette_control1: PaletteRegister,
    object_palette_control2: PaletteRegister,
    object_palette_control3: PaletteRegister,

    // Internal
    /// High if drawing from framebuffer 1. Otherwise drawing from framebuffer 0.
    drawing_framebuffer_1: bool,

    in_drawing: bool,
    drawing_cycle_count: u32,

    frame_count: u8,
}

pub struct VIPInterrupt {
    /// Mirrors are not stable.
    scanerr: bool,
    /// The display procedure has completed for the left eye.
    lfbend: bool,
    /// The display procedure has completed for the right eye.
    rfbend: bool,
    /// The drawing procedure has begun.
    gamestart: bool,
    /// The display procedure has begun.
    framestart: bool,
    /// Drawing has begun on the group of 8 rows of pixels specified in the `SBCMP` field of `XPCTRL`.
    sbhit: bool,
    /// The drawing procedure has finished.
    xpend: bool,
    /// Drawing is in progress on a frame buffer that will next be displayed.
    timeerr: bool,
}

pub struct PaletteRegister {
    character1: u8,
    character2: u8,
    character3: u8,
}

impl VIP {
    pub fn get_byte(&self, address: u32) -> u8 {
        let address = address as usize;

        match address {
            0x0..=0x4_0000 => self.vram[address],
            0x5_F860 => self.background_palette_control0.get(),
            0x5_F862 => self.background_palette_control1.get(),
            0x5_F864 => self.background_palette_control2.get(),
            0x5_F866 => self.background_palette_control3.get(),
            0x5_F868 => self.object_palette_control0.get(),
            0x5_F86A => self.object_palette_control0.get(),
            0x5_F86C => self.object_palette_control0.get(),
            0x5_F86E => self.object_palette_control0.get(),
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.vram[(address & 0x1FFF) + 0x6000]
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.vram[(address & 0x1FFF) + 0xE000]
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.vram[(address & 0x1FFF) + 0x1_6000]
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.vram[(address & 0x1FFF) + 0x1_E000]
            }
        }
    }

    pub fn set_byte(&self, address: u32, value: u8) {
        let address = address as usize;

        match address {
            0x0..=0x4_0000 => self.vram[address] = value,
            0x5_F860 => self.background_palette_control0.set(value),
            0x5_F862 => self.background_palette_control1.set(value),
            0x5_F864 => self.background_palette_control2.set(value),
            0x5_F866 => self.background_palette_control3.set(value),
            0x5_F868 => self.object_palette_control0.set(value),
            0x5_F86A => self.object_palette_control0.set(value),
            0x5_F86C => self.object_palette_control0.set(value),
            0x5_F86E => self.object_palette_control0.set(value),
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.vram[(address & 0x1FFF) + 0x6000] = value;
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.vram[(address & 0x1FFF) + 0xE000] = value;
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.vram[(address & 0x1FFF) + 0x1_6000] = value;
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.vram[(address & 0x1FFF) + 0x1_E000] = value;
            }
        }
    }

    pub fn run_for_cycles(&mut self, cycles_to_run: usize) {
        for _ in 0..cycles_to_run {
            // Display process
            match self.current_display_clock_cycle {
                0 => {
                    // Raise FCLK
                    self.init_display_frame();
                }
                LEFT_FRAME_BUFFER_CYCLE_OFFSET => {
                    // Render left frame buffer
                    // Column table stuff is ignored as it's not relevant to software emulation.
                    // TODO: Do we need to update CTA?
                    self.display_framebuffer();
                }
                LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End left frame buffer
                    self.interrupt.lfbend = true;
                }
                FCLK_LOW_CYCLE_OFFSET => {
                    // Lower FCLK
                    self.fclk = false;
                }
                RIGHT_FRAME_BUFFER_CYCLE_OFFSET => {
                    // Render right frame buffer
                }
                RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End right frame buffer
                    self.interrupt.rfbend = true;
                }
                FRAME_COMPLETE_CYCLE_OFFSET => {
                    // End frame
                }
                _ => {}
            }

            // Drawing process
            if self.in_drawing {
                // Display and drawing segments run in parallel
                if self.drawing_cycle_count == DRAWING_BLOCK_CYCLE_COUNT {
                    self.drawing_cycle_count = 0;

                    if self.sbcount < DRAWING_BLOCK_COUNT as u8 {
                        // Within the frame still
                        self.draw_block_row();

                        if self.sbcount == 0 {
                            // First set of rows just rendered, copy over BKCOL
                            self.last_bkcol = self.bkcol;
                        }

                        // TODO: Set and clear SBOUT
                        self.sbcount += 1;
                    } else {
                        self.sbcount = 0;
                        self.interrupt.xpend = true;
                    }
                } else {
                    self.drawing_cycle_count += 1;
                }
            }

            if self.current_display_clock_cycle == FRAME_COMPLETE_CYCLE_OFFSET {
                self.current_display_clock_cycle = 0;
            } else {
                self.current_display_clock_cycle += 1;
            }
        }
    }

    fn init_display_frame(&mut self) {
        self.fclk = true;

        self.interrupt.framestart = true;

        self.frame_count += 1;

        if self.frame_count == self.frmcyc + 1 {
            self.frame_count = 0;

            self.init_drawing_frame();
        }
    }

    fn init_drawing_frame(&mut self) {
        self.interrupt.gamestart = true;

        if self.drawing_enabled {
            // Flip framebuffers to start writing to the currently displayed ones
            self.drawing_framebuffer_1 = !self.drawing_framebuffer_1;

            // Enter render mode
            self.in_drawing = true;
        } else {
            // Immediately mark drawing as ended, as we're not drawing at all
            // TODO: This should actually be after 2.8ms
            self.interrupt.xpend = true;
        }
    }

    ///
    /// Draws a block (1x8), but for an entire row. This represents 1 value of `SBCOUNT`.
    ///
    /// The actual hardware does not draw entire rows at a time, but due to lack of "racing the beam", there's no
    /// real reason to break up the drawing.
    fn draw_block_row(&mut self) {
        // Get drawing framebuffer
        let (left_framebuffer_address, right_framebuffer_address) =
            framebuffer_addresses(self.drawing_framebuffer_1);

        // Initialize all values to BKCOL
        let clear_pixel = (self.last_bkcol << 6)
            | (self.last_bkcol << 4)
            | (self.last_bkcol << 2)
            | self.last_bkcol;

        let y = self.sbcount as usize * 8;

        for x in 0..DISPLAY_WIDTH {
            // Overwrite every pixel in the 384x8 segment
            // Pixels are stored in the framebuffer in columns, rather than in rows
            let x_index = x * DISPLAY_HEIGHT;
            // This contains the bottom three bytes as the bit offset
            let pixel_byte_index = (x_index + y) >> 2;
            self.set_byte(
                (left_framebuffer_address + pixel_byte_index) as u32,
                clear_pixel,
            );
            self.set_byte(
                (left_framebuffer_address + pixel_byte_index + 1) as u32,
                clear_pixel,
            );
            self.set_byte(
                (right_framebuffer_address + pixel_byte_index) as u32,
                clear_pixel,
            );
            self.set_byte(
                (right_framebuffer_address + pixel_byte_index + 1) as u32,
                clear_pixel,
            );
        }

        // Counter for total object groups
        let mut object_group_counter = 3;

        for i in 31..=0 {
            // Process world
            let world_attribute_address = 0x3_D800 + 16 * i;
            let bytes = &self.vram[world_attribute_address..world_attribute_address + 10 * 2];

            let world = World::parse(bytes);

            if world.end {
                // We're done processing worlds
                break;
            } else if world.display_state == WorldDisplayState::Dummy {
                // Dummy world, skip
                continue;
            }

            match world.background_type {
                BackgroundType::Normal => {
                    self.render_normal_or_hbias_background(&world, true, false, y);
                    self.render_normal_or_hbias_background(&world, false, false, y);
                }
                BackgroundType::HBias => todo!(),
                BackgroundType::Affine => todo!(),
                BackgroundType::Obj => todo!(),
            }
        }
    }

    fn render_normal_or_hbias_background(
        &mut self,
        world: &World,
        left_eye: bool,
        is_hbias: bool,
        block_start_y: usize,
    ) {
        let (left_framebuffer_address, right_framebuffer_address) =
            framebuffer_addresses(self.drawing_framebuffer_1);

        let framebuffer_address = if left_eye {
            left_framebuffer_address
        } else {
            right_framebuffer_address
        };

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

            // TODO: Implement HBias
            let line_offset = if is_hbias {
                // HBias has two additional parameters
                let base_address = world.param_base + 10 * 2;

                if left_eye {
                    let slice = &self.vram[base_address..base_address + 2];

                    sign_extend_16(((slice[1] as u16) << 8) | (slice[0] as u16), 13)
                } else {
                    // "The VIP appears to determine the address of HOFSTR by OR'ing the address of HOFSTL with 2.
                    // If the Param Base attribute in the world is not divisibe by 2, this will result in HOFSTL being
                    // used for both the left and right images, and HOFSTR will not be accessed."
                    let base_address = base_address | 0x2;
                    let slice = &self.vram[base_address..base_address + 2];
                    sign_extend_16(((slice[1] as u16) << 8) | (slice[1] as u16), 13)
                }
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

                let background_x = if left_eye {
                    background_x.wrapping_sub(world.background_parallax_source)
                } else {
                    background_x.wrapping_add(world.background_parallax_source)
                };

                let background_y = window_y.wrapping_add(world.background_y_source);

                self.draw_background_pixel(
                    world,
                    left_eye,
                    x,
                    y,
                    background_x as usize,
                    background_y as usize,
                );
            }
        }
    }

    fn display_framebuffer(&mut self) {
        if !self.display_enabled || !self.sync_enabled {
            // Do not render
            return;
        }

        // Rendering is done with the opposite of the drawing framebuffer
        let (left_framebuffer_address, right_framebuffer_address) =
            framebuffer_addresses(!self.drawing_framebuffer_1);

        let brightness_c = self
            .brightness_control_reg_a
            .wrapping_add(self.brightness_control_reg_b)
            .wrapping_add(self.brightness_control_reg_c);

        for x in 0..DISPLAY_WIDTH {
            for y in 0..DISPLAY_HEIGHT {
                // Pixels are stored in the framebuffer in columns, rather than in rows
                let x_index = x * DISPLAY_HEIGHT;
                // This contains the bottom three bytes as the bit offset
                let pixel_byte_index = (x_index + y) >> 2;

                // Remove bottom bit, as we're getting every 2 bits
                let bit_index = y & 0x6;

                // TODO: Cache these pixels instead of refetching
                // Could mask this with 0x3, but the match below makes that unnecessary
                let left_pixel = self
                    .get_byte((left_framebuffer_address + pixel_byte_index) as u32)
                    >> bit_index;

                let right_pixel = self
                    .get_byte((right_framebuffer_address + pixel_byte_index) as u32)
                    >> bit_index;

                let left_pixel = match left_pixel {
                    0 => 0,
                    1 => self.brightness_control_reg_a,
                    2 => self.brightness_control_reg_b,
                    _ => brightness_c,
                };

                let right_pixel = match right_pixel {
                    0 => 0,
                    1 => self.brightness_control_reg_a,
                    2 => self.brightness_control_reg_b,
                    _ => brightness_c,
                };

                let output_framebuffer_index = y * DISPLAY_WIDTH + x;
                self.left_rendered_framebuffer[output_framebuffer_index] = left_pixel;
                self.right_rendered_framebuffer[output_framebuffer_index] = right_pixel;
            }
        }
    }

    fn draw_background_pixel(
        &mut self,
        world: &World,
        left_eye: bool,
        x: i16,
        y: i16,
        background_x: usize,
        background_y: usize,
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
            let active_background_map_x = (background_x / 512) & (total_background_width - 1);
            let active_background_map_y = (background_y / 512) & (total_background_height - 1);

            // Each background is 0x2000 bytes
            let background_base_offset_address =
                0x2_0000 + (world.map_base_index as usize) * 0x2000;
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
            let character_address =
                background_offset_address + (character_y * 64 + character_x) * 2;

            // Get character block info
            let character_halfword = ((self.vram[character_address + 1] as u16) << 8)
                | (self.vram[character_address] as u16);

            let character_index = character_halfword & 0x7FF;
            let horizontal_flip = character_halfword & 0x1000 != 0;
            let vertical_flip = character_halfword & 0x2000 != 0;
            let palette = character_halfword >> 14;

            // TODO: Handle OBJ palettes
            let palette = match palette {
                0 => &self.background_palette_control0,
                1 => &self.background_palette_control1,
                2 => &self.background_palette_control2,
                _ => &self.background_palette_control3,
            };

            // Flip pixel position, if necessary
            let x = (if horizontal_flip { 7 - x } else { x }) as usize;
            let y = (if vertical_flip { 7 - y } else { y }) as usize;

            // Index into character blocks using the virtual addresses for ease of access
            // 8 rows per block. 2 bytes per row = 16 per character
            let character_address = 0x7_8000 + (character_index as usize) * 16;

            // Index to the correct row
            let character_address = character_address + background_pixel_offset_y * 2;

            // TODO: This can be optimized
            let row_halfword = ((self.vram[character_address + 1] as u16) << 8)
                | (self.vram[character_address] as u16);

            // Extract pixel
            let pixel_palette_index = (row_halfword >> (background_pixel_offset_x * 2)) & 0x3;

            if pixel_palette_index == 0 {
                return;
            }

            let pixel = match pixel_palette_index {
                // No need to draw. Background "blank" pixel has already been written to FB
                0 => return,
                1 => palette.character1,
                2 => palette.character2,
                _ => palette.character3,
            };

            // Write to framebuffer
            let framebuffer_offset = y * DISPLAY_WIDTH + x;
            // Each pixel is 2 bits, so find the right byte for this pixel
            let framebuffer_byte_offset = framebuffer_offset / 4;
            let pixel_shift = (y & 0x3) * 2;

            let framebuffer_address =
                framebuffer_address_at_side(left_eye, self.drawing_framebuffer_1)
                    + framebuffer_byte_offset;

            let removal_mask = !(0x3 << pixel_shift);

            let existing_byte = self.vram[framebuffer_address];
            let byte = (existing_byte & removal_mask) | (pixel << pixel_shift);
            self.vram[framebuffer_address] = byte;
        }
    }
}

impl PaletteRegister {
    fn get(&self) -> u8 {
        (self.character3 << 6) | (self.character2 << 4) | (self.character1 << 2)
    }

    fn set(&mut self, value: u8) {
        self.character1 = (value >> 2) & 0x3;
        self.character2 = (value >> 4) & 0x3;
        self.character3 = (value >> 6) & 0x3;
    }
}

fn framebuffer_addresses(use_fb_1: bool) -> (usize, usize) {
    let left_framebuffer_address = if use_fb_1 { 0x8000 } else { 0 };
    let right_framebuffer_address = left_framebuffer_address + 0x1_0000;

    (left_framebuffer_address, right_framebuffer_address)
}

fn framebuffer_address_at_side(left_eye: bool, use_fb_1: bool) -> usize {
    let (left_framebuffer_address, right_framebuffer_address) = framebuffer_addresses(use_fb_1);

    match left_eye {
        true => left_framebuffer_address,
        false => right_framebuffer_address,
    }
}
