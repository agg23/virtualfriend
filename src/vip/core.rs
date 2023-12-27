use std::f32::consts::E;

use crate::constants::{
    DISPLAY_HEIGHT, DISPLAY_PIXEL_LENGTH, DISPLAY_WIDTH, DRAWING_BLOCK_COUNT,
    DRAWING_BLOCK_CYCLE_COUNT, FCLK_LOW_CYCLE_OFFSET, FRAME_COMPLETE_CYCLE_OFFSET,
    LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET, LEFT_FRAME_BUFFER_CYCLE_OFFSET,
    RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET, RIGHT_FRAME_BUFFER_CYCLE_OFFSET,
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

impl VIP {
    pub fn get_byte(&self, address: u32) -> u8 {
        let address = address as usize;

        match address {
            0x0..=0x4_0000 => self.vram[address],
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
                    self.render_normal_background(&world, true);
                    self.render_normal_background(&world, false);
                }
                BackgroundType::HBias => todo!(),
                BackgroundType::Affine => todo!(),
                BackgroundType::Obj => todo!(),
            }
        }
    }

    fn render_normal_background(&mut self, world: &World, left_eye: bool) {
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

        // TODO: Loop over y pixels in range of this block
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
}

fn framebuffer_addresses(use_fb_1: bool) -> (usize, usize) {
    let left_framebuffer_address = if use_fb_1 { 0x8000 } else { 0 };
    let right_framebuffer_address = left_framebuffer_address + 0x1_0000;

    (left_framebuffer_address, right_framebuffer_address)
}
