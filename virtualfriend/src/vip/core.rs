// This design is heavily influenced by Rustual Boy, and some of the rendering logic is nearly identical
// (though rewritten and commented differently). Logic has been implemented independantly with docs and
// Rustual Boy as a reference without intending to copy. The process thus producing many bugs that have been
// fixed by examining the Rustual Boy codebase.

use bitvec::field::BitField;
use bitvec::prelude::Lsb0;
use bitvec::{array::BitArray, bitarr};
use tartan_bitfield::bitfield;

use crate::constants::{
    DISPLAY_HEIGHT, DISPLAY_PIXEL_LENGTH, DISPLAY_WIDTH, DRAWING_BLOCK_COUNT,
    DRAWING_BLOCK_CYCLE_COUNT, FCLK_LOW_CYCLE_OFFSET, FRAME_COMPLETE_CYCLE_OFFSET,
    LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET, LEFT_FRAME_BUFFER_CYCLE_OFFSET,
    RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET, RIGHT_FRAME_BUFFER_CYCLE_OFFSET,
};
use crate::constants::{FRAMEBUFFER_HEIGHT, SBOUT_HIGH_CYCLE_COUNT};

use super::drawing::draw_block_row;
use super::util::{framebuffer_addresses, RenderState};
use super::vram::VRAM;

pub struct VIP {
    pub current_display_clock_cycle: usize,

    vram: VRAM,

    pub left_rendered_framebuffer: Vec<u8>,
    pub right_rendered_framebuffer: Vec<u8>,

    interrupt_pending: VIPInterrupt,
    interrupt_enabled: VIPInterrupt,

    render_state: RenderState,

    // Registers
    // DPSTTS/DPCTRL
    pub display_enabled: bool,
    /// Simple R/W register in emulator.
    ///
    /// Would normally allow you to disable sending memory refreshes to VRAM
    refresh_ram: bool,

    fclk: bool,
    /// Sync signals are being sent to the displays, preventing images from being displayed.
    sync_enabled: bool,
    /// When set, column table read start address will not change
    lock_column_table: bool,

    /// XPEN
    pub drawing_enabled: bool,

    /// Game frame control register
    ///
    /// This value +1 is the number of display frames per rendered frame
    frmcyc: u8,

    /// The most recent (or locked value, if `lock_column_table`) index of the start of the column table entry
    // TODO: Unimplemented
    last_left_column_table_index: u8,
    last_right_column_table_index: u8,

    /// Tracks the number of cycles `SBOUT` is high. Brings it low after 56us
    sbout_cycle_high_count: usize,

    // Internal
    current_displaying: DisplayState,

    pub in_drawing: bool,
    drawing_cycle_count: usize,

    frame_count: u8,
}

#[derive(PartialEq)]
pub enum DisplayState {
    Left,
    Right,
    None,
}

bitfield! {
    pub struct VIPInterrupt(u16) {
        /// Mirrors are not stable.
        [0] scanerr,
        /// The display procedure has completed for the left eye.
        [1] lfbend,
        /// The display procedure has completed for the right eye.
        [2] rfbend,
        /// The drawing procedure has begun.
        [3] gamestart,
        /// The display procedure has begun.
        [4] framestart,
        /// Drawing has begun on the group of 8 rows of pixels specified in the `SBCMP` field of `XPCTRL`.
        [13] sbhit,
        /// The drawing procedure has finished.
        [14] xpend,
        /// Drawing is in progress on a frame buffer that will next be displayed.
        [15] timeerr,
    }
}

impl VIP {
    pub fn new() -> Self {
        let mut left_rendered_framebuffer = Vec::with_capacity(DISPLAY_PIXEL_LENGTH);
        let mut right_rendered_framebuffer = Vec::with_capacity(DISPLAY_PIXEL_LENGTH);

        for _ in 0..DISPLAY_PIXEL_LENGTH {
            left_rendered_framebuffer.push(0);
            right_rendered_framebuffer.push(0);
        }

        VIP {
            current_display_clock_cycle: 0,
            vram: VRAM::new(),
            left_rendered_framebuffer,
            right_rendered_framebuffer,
            interrupt_pending: VIPInterrupt(0),
            interrupt_enabled: VIPInterrupt(0),
            render_state: RenderState::new(),
            // Mednafen starts with display enabled
            display_enabled: true,
            refresh_ram: true,
            fclk: false,
            sync_enabled: false,
            lock_column_table: false,
            drawing_enabled: false,
            frmcyc: 0,
            last_left_column_table_index: 0,
            last_right_column_table_index: 0,
            sbout_cycle_high_count: 0,
            current_displaying: DisplayState::None,
            in_drawing: false,
            drawing_cycle_count: 0,
            frame_count: 0,
        }
    }

    pub fn get_bus(&self, address: usize) -> u16 {
        match address {
            0x0..=0x3_FFFF => self.vram.get_u16(address),
            0x5_F800..=0x5_F801 => {
                // INTPND Interrupt pending
                self.interrupt_pending.0
            }
            0x5_F802..=0x5_F803 => {
                // INTENB Interrupt enable
                self.interrupt_enabled.0
            }
            0x5_F804..=0x5F805 => {
                // INTCLEAR Interrupt clear
                // Reading is undefined
                0
            }
            0x5_F820..=0x5_F821 => {
                // DPSTTS Display control read register
                let mut value = bitarr![u16, Lsb0; 0; 16];

                value.set(1, self.display_enabled);
                // Displaying left framebuffer 0
                value.set(
                    2,
                    self.render_state.drawing_framebuffer_1
                        && self.current_displaying == DisplayState::Left,
                );
                value.set(
                    3,
                    self.render_state.drawing_framebuffer_1
                        && self.current_displaying == DisplayState::Right,
                );
                // Displaying left framebuffer 1
                value.set(
                    4,
                    !self.render_state.drawing_framebuffer_1
                        && self.current_displaying == DisplayState::Left,
                );
                value.set(
                    5,
                    !self.render_state.drawing_framebuffer_1
                        && self.current_displaying == DisplayState::Right,
                );

                // Scan ready
                // TODO: This should probably only be set after a delay?
                value.set(6, true);

                value.set(7, self.fclk);
                value.set(8, self.refresh_ram);
                value.set(9, self.sync_enabled);
                value.set(10, self.lock_column_table);

                value.load()
            }
            0x5_F824..=0x5_F825 => self.render_state.brightness_control_reg_a as u16,
            0x5_F826..=0x5_F827 => self.render_state.brightness_control_reg_b as u16,
            0x5_F828..=0x5_F829 => self.render_state.brightness_control_reg_c as u16,
            0x5_F82A..=0x5_F82B => self.render_state.led_rest_duration as u16,
            0x5_F82E..=0x5_F82F => self.frmcyc as u16,
            0x5_F830..=0x5_F831 => {
                ((self.last_right_column_table_index as u16) << 8)
                    | (self.last_left_column_table_index as u16)
            }
            0x5_F840..=0x5_F841 => {
                // XPSTTS Drawing control read register
                let mut value = bitarr![u16, Lsb0; 0; 16];

                value.set(1, self.drawing_enabled);
                // Drawing to framebuffer 0
                value.set(
                    2,
                    self.in_drawing && !self.render_state.drawing_framebuffer_1,
                );
                value.set(
                    3,
                    self.in_drawing && self.render_state.drawing_framebuffer_1,
                );
                // TODO: Detect when drawing would overrun. OVERTIME
                value.set(4, false);
                value.set(15, self.render_state.sbout);

                let value: u16 = value.load();

                value | ((self.render_state.sbcount as u16) << 8)
            }
            0x5_F844..=0x5_F845 => {
                // VIP Version
                // Only one version, always 2
                2
            }
            0x5_F848..=0x5_F849 => self.render_state.object_group_end0,
            0x5_F84A..=0x5_F84B => self.render_state.object_group_end1,
            0x5_F84C..=0x5_F84D => self.render_state.object_group_end2,
            0x5_F84E..=0x5_F84F => self.render_state.object_group_end3,
            0x5_F860..=0x5_F861 => self.render_state.background_palette_control0.get() as u16,
            0x5_F862..=0x5_F863 => self.render_state.background_palette_control1.get() as u16,
            0x5_F864..=0x5_F865 => self.render_state.background_palette_control2.get() as u16,
            0x5_F866..=0x5_F867 => self.render_state.background_palette_control3.get() as u16,
            0x5_F868..=0x5_F869 => self.render_state.object_palette_control0.get() as u16,
            0x5_F86A..=0x5_F86B => self.render_state.object_palette_control1.get() as u16,
            0x5_F86C..=0x5_F86D => self.render_state.object_palette_control2.get() as u16,
            0x5_F86E..=0x5_F86F => self.render_state.object_palette_control3.get() as u16,
            0x5_F870..=0x5_F871 => self.render_state.bkcol as u16,
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.vram.get_u16((address & 0x1FFF) + 0x6000)
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.vram.get_u16((address & 0x1FFF) + 0xE000)
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.vram.get_u16((address & 0x1FFF) + 0x1_6000)
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.vram.get_u16((address & 0x1FFF) + 0x1_E000)
            }
            _ => {
                println!("Read from invalid register address {address:08X}");
                0
            }
        }
    }

    pub fn set_bus(&mut self, address: u32, value: u16) {
        let address = address as usize;

        match address {
            0x0..=0x3_FFFF => self.vram.set_u16(address, value),
            0x4_0000..=0x5_DFFF => panic!("Invalid VIP address"),
            0x5_F800..=0x5_F801 => {
                // INTPND Interrupt pending
                // Writes have no effect
            }
            0x5_F802..=0x5_F803 => {
                // INTENB Interrupt enable
                self.interrupt_enabled.0 = value;
            }
            0x5_F804..=0x5F805 => {
                // INTCLEAR Interrupt clear
                self.set_intclear(value);
            }
            0x5_F822..=0x5F823 => {
                // DPCTRL Display control write register
                let array = BitArray::<_, Lsb0>::new([value]);

                // TODO: Handle
                let reset_display = *array.get(0).unwrap();

                if reset_display {
                    self.current_display_clock_cycle = 0;
                    self.current_displaying = DisplayState::None;

                    self.interrupt_pending.set_gamestart(false);
                    self.interrupt_pending.set_framestart(false);
                    self.interrupt_pending.set_lfbend(false);
                    self.interrupt_pending.set_rfbend(false);

                    self.interrupt_enabled.set_gamestart(false);
                    self.interrupt_enabled.set_framestart(false);
                    self.interrupt_enabled.set_lfbend(false);
                    self.interrupt_enabled.set_rfbend(false);
                }

                self.display_enabled = *array.get(1).unwrap();
                self.refresh_ram = *array.get(8).unwrap();
                self.sync_enabled = *array.get(9).unwrap();
                self.lock_column_table = *array.get(10).unwrap();
            }
            0x5_F824..=0x5_F825 => self.render_state.brightness_control_reg_a = value as u8,
            0x5_F826..=0x5_F827 => self.render_state.brightness_control_reg_b = value as u8,
            0x5_F828..=0x5_F829 => self.render_state.brightness_control_reg_c = value as u8,
            0x5_F82A..=0x5_F82B => self.render_state.led_rest_duration = value as u8,
            0x5_F82E..=0x5_F82F => self.frmcyc = (value & 0xF) as u8,
            0x5_F842..=0x5_F843 => {
                // XPCTRL Drawing control write register
                let array = BitArray::<_, Lsb0>::new([value]);

                let reset_drawing = *array.get(0).unwrap();

                if reset_drawing {
                    self.in_drawing = false;
                    self.drawing_cycle_count = 0;
                    self.sbout_cycle_high_count = 0;
                    self.render_state.sbout = false;
                    self.render_state.sbcount = 0;

                    self.interrupt_enabled.set_xpend(false);
                    self.interrupt_pending.set_xpend(false);
                }

                self.drawing_enabled = *array.get(1).unwrap();

                self.render_state.sbcmp = ((value >> 8) & 0x1F) as u8;
            }
            0x5_F848..=0x5_F849 => self.render_state.object_group_end0 = value & 0x3FF,
            0x5_F84A..=0x5_F84B => self.render_state.object_group_end1 = value & 0x3FF,
            0x5_F84C..=0x5_F84D => self.render_state.object_group_end2 = value & 0x3FF,
            0x5_F84E..=0x5_F84F => self.render_state.object_group_end3 = value & 0x3FF,
            0x5_F860..=0x5_F861 => self.render_state.background_palette_control0.set(value),
            0x5_F862..=0x5_F863 => self.render_state.background_palette_control1.set(value),
            0x5_F864..=0x5_F865 => self.render_state.background_palette_control2.set(value),
            0x5_F866..=0x5_F867 => self.render_state.background_palette_control3.set(value),
            0x5_F868..=0x5_F869 => self.render_state.object_palette_control0.set(value),
            0x5_F86A..=0x5_F86B => self.render_state.object_palette_control1.set(value),
            0x5_F86C..=0x5_F86D => self.render_state.object_palette_control2.set(value),
            0x5_F86E..=0x5_F86F => self.render_state.object_palette_control3.set(value),
            0x5_F870..=0x5_F871 => self.render_state.bkcol = (value & 0x3) as u8,
            0x6_0000..=0x7_7FFF => panic!("Invalid VIP address"),
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.vram.set_u16((address & 0x1FFF) + 0x6000, value);
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.vram.set_u16((address & 0x1FFF) + 0xE000, value);
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.vram.set_u16((address & 0x1FFF) + 0x1_6000, value);
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.vram.set_u16((address & 0x1FFF) + 0x1_E000, value);
            }
            _ => println!("Write to invalid register address {address:08X}"),
        }
    }

    /// Runs the VIP for `cycles_to_run`.
    ///
    /// Returns true if an interrupt is requested from the VIP
    pub fn step(&mut self, cycles_to_run: usize) -> bool {
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
                    println!("Displaying framebuffer");
                    self.display_framebuffer();

                    if self.display_enabled && self.sync_enabled {
                        self.current_displaying = DisplayState::Left;
                    }
                }
                LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End left frame buffer
                    if self.display_enabled {
                        self.interrupt_pending.set_lfbend(true);

                        self.current_displaying = DisplayState::None;
                    }
                }
                FCLK_LOW_CYCLE_OFFSET => {
                    // Lower FCLK
                    self.fclk = false;
                }
                RIGHT_FRAME_BUFFER_CYCLE_OFFSET => {
                    // Render right frame buffer
                    // TODO: Render other eye
                    if self.display_enabled && self.sync_enabled {
                        self.current_displaying = DisplayState::Right;
                    }
                }
                RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End right frame buffer
                    if self.display_enabled {
                        self.interrupt_pending.set_rfbend(true);
                    }

                    self.current_displaying = DisplayState::None;
                }
                FRAME_COMPLETE_CYCLE_OFFSET => {
                    // End frame
                    println!("End of frame");
                }
                _ => {}
            }

            // Drawing process
            if self.in_drawing {
                // Display and drawing segments run in parallel
                if self.drawing_cycle_count == DRAWING_BLOCK_CYCLE_COUNT {
                    self.drawing_cycle_count = 0;

                    if self.render_state.sbcount < DRAWING_BLOCK_COUNT as u8 {
                        // Within the frame still
                        draw_block_row(&mut self.vram, &self.render_state);

                        if self.render_state.sbcount == 0 {
                            // First set of rows just rendered, copy over BKCOL
                            self.render_state.last_bkcol = self.render_state.bkcol;
                        }

                        if self.render_state.sbcount == self.render_state.sbcmp {
                            // Found rows. Fire SBHit
                            // Fire interrupt
                            self.interrupt_pending.set_sbhit(true);
                        }

                        self.render_state.sbout = true;
                        self.sbout_cycle_high_count = 0;

                        self.render_state.sbcount += 1;
                    } else {
                        // Finished drawing
                        self.in_drawing = false;

                        println!("Ended drawing");

                        self.render_state.sbcount = 0;
                        self.interrupt_pending.set_xpend(true);
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

            if self.render_state.sbout {
                // Run SBOUT timer
                if self.sbout_cycle_high_count == SBOUT_HIGH_CYCLE_COUNT {
                    self.render_state.sbout = false;
                } else {
                    self.sbout_cycle_high_count += 1;
                }
            }
        }

        self.interrupt_pending
            .check_intersection(&self.interrupt_enabled)
    }

    fn init_display_frame(&mut self) {
        self.fclk = true;

        self.interrupt_pending.set_framestart(true);

        self.frame_count += 1;

        if self.frame_count == self.frmcyc + 1 {
            self.frame_count = 0;

            self.init_drawing_frame();
        }
    }

    fn init_drawing_frame(&mut self) {
        self.interrupt_pending.set_gamestart(true);

        if self.drawing_enabled {
            // Flip framebuffers to start writing to the currently displayed ones
            self.render_state.drawing_framebuffer_1 = !self.render_state.drawing_framebuffer_1;
            println!(
                "Starting draw. Flipped framebuffer to {}",
                self.render_state.drawing_framebuffer_1
            );

            self.render_state.sbcount = 0;
            self.drawing_cycle_count = 0;

            // Enter render mode
            self.in_drawing = true;
        } else {
            // Immediately mark drawing as ended, as we're not drawing at all
            // TODO: This should actually be after 2.8ms
            self.interrupt_pending.set_xpend(true);
            self.in_drawing = false;
        }
    }

    fn display_framebuffer(&mut self) {
        if !self.display_enabled || !self.sync_enabled {
            // Do not render
            return;
        }

        // Rendering is done with the opposite of the drawing framebuffer
        let (left_framebuffer_address, right_framebuffer_address) =
            framebuffer_addresses(!self.render_state.drawing_framebuffer_1);

        // Pixels tend to range from 0-127 in brightness, so double the value to use the full range
        // We allow the value to theoretically grow above 255 (mainly for brightness_c), and we will make
        // sure it doesn't grow over 255
        let brightness_a = (self.render_state.brightness_control_reg_a as u16) * 2;
        let brightness_b = (self.render_state.brightness_control_reg_b as u16) * 2;

        let brightness_c = ((self.render_state.brightness_control_reg_a as u16)
            + (self.render_state.brightness_control_reg_b as u16)
            + (self.render_state.brightness_control_reg_c as u16))
            * 2;

        let brightness_a = brightness_a.min(255) as u8;
        let brightness_b = brightness_b.min(255) as u8;
        let brightness_c = brightness_c.min(255) as u8;

        for x in 0..DISPLAY_WIDTH {
            for y in 0..DISPLAY_HEIGHT {
                // Pixels are stored in the framebuffer in columns, rather than in rows
                let x_index = x * FRAMEBUFFER_HEIGHT;
                // This contains the bottom three bytes as the bit offset
                let pixel_byte_index = (x_index + y) >> 2;

                // 4 pixels per word, two bits per pixel, get shift offset in pixel
                let bit_index = (y & 0x3) << 1;

                // TODO: Cache these pixels instead of refetching
                let left_pixel = (self
                    .vram
                    .get_u8(left_framebuffer_address + pixel_byte_index)
                    >> bit_index)
                    & 0x3;

                let right_pixel = (self
                    .vram
                    .get_u8(right_framebuffer_address + pixel_byte_index)
                    >> bit_index)
                    & 0x3;

                let left_pixel = match left_pixel {
                    0 => 0,
                    1 => brightness_a,
                    2 => brightness_b,
                    _ => brightness_c,
                };

                let right_pixel = match right_pixel {
                    0 => 0,
                    1 => brightness_a,
                    2 => brightness_b,
                    _ => brightness_c,
                };

                let output_framebuffer_index = y * DISPLAY_WIDTH + x;
                self.left_rendered_framebuffer[output_framebuffer_index] = left_pixel;
                self.right_rendered_framebuffer[output_framebuffer_index] = right_pixel;
            }
        }
    }

    fn set_intclear(&mut self, value: u16) {
        // Clear any interrupts that are set in `value`
        self.interrupt_pending.0 &= !value;
    }
}

impl VIPInterrupt {
    /// Intersects two sets of interrupt values. If there is at least one intersection (both sides
    /// have a true value), this method will return true.
    fn check_intersection(&self, b: &VIPInterrupt) -> bool {
        self.0 & b.0 != 0
    }
}
