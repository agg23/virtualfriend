use bitvec::field::BitField;
use bitvec::prelude::Lsb0;
use bitvec::{array::BitArray, bitarr};

use crate::constants::SBOUT_HIGH_CYCLE_COUNT;
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
    current_display_clock_cycle: usize,

    // We map the entirety of VRAM due to overlapping sections
    // (upper background maps overlap with OAM and properties).
    vram: [u16; 0x4_0000 / 2],

    left_rendered_framebuffer: [u8; DISPLAY_PIXEL_LENGTH],
    right_rendered_framebuffer: [u8; DISPLAY_PIXEL_LENGTH],

    interrupt_pending: VIPInterrupt,
    interrupt_enabled: VIPInterrupt,

    // Registers
    // DPSTTS/DPCTRL
    display_enabled: bool,
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
    drawing_enabled: bool,

    /// Game frame control register
    ///
    /// This value +1 is the number of display frames per rendered frame
    frmcyc: u8,

    /// The most recent (or locked value, if `lock_column_table`) index of the start of the column table entry
    // TODO: Unimplemented
    last_left_column_table_index: u8,
    last_right_column_table_index: u8,

    /// BG color palette control register
    ///
    /// When set, new background color is not applied until the first 8 rows of the next frame are rendered.
    bkcol: u8,
    last_bkcol: u8,

    // SB Allows user to watch for the render of a set of 8 rows
    /// The current group of 8 rows of pixels, relative to the top of the image, currently being drawn.
    sbcount: u8,
    /// The group of 8 rows of pixels, relative to the top of the image, to compare with while drawing.
    sbcmp: u8,
    sbout: bool,

    /// Tracks the number of cycles `SBOUT` is high. Brings it low after 56us
    sbout_cycle_high_count: usize,

    object_control0: u16,
    object_control1: u16,
    object_control2: u16,
    object_control3: u16,

    brightness_control_reg_a: u8,
    brightness_control_reg_b: u8,
    brightness_control_reg_c: u8,

    // TODO: Implement
    led_rest_duration: u8,

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
    current_displaying: DisplayState,

    in_drawing: bool,
    drawing_cycle_count: usize,

    frame_count: u8,
}

#[derive(PartialEq)]
pub enum DisplayState {
    Left,
    Right,
    None,
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
    pub fn new() -> Self {
        VIP {
            current_display_clock_cycle: 0,
            vram: [0; 0x4_0000 / 2],
            left_rendered_framebuffer: [0; DISPLAY_PIXEL_LENGTH],
            right_rendered_framebuffer: [0; DISPLAY_PIXEL_LENGTH],
            interrupt_pending: VIPInterrupt::new(),
            interrupt_enabled: VIPInterrupt::new(),
            display_enabled: false,
            refresh_ram: true,
            fclk: false,
            sync_enabled: false,
            lock_column_table: false,
            drawing_enabled: false,
            frmcyc: 0,
            last_left_column_table_index: 0,
            last_right_column_table_index: 0,
            bkcol: 0,
            last_bkcol: 0,
            sbcount: 0,
            sbcmp: 0,
            sbout: false,
            sbout_cycle_high_count: 0,
            object_control0: 0,
            object_control1: 0,
            object_control2: 0,
            object_control3: 0,
            brightness_control_reg_a: 0,
            brightness_control_reg_b: 0,
            brightness_control_reg_c: 0,
            led_rest_duration: 0,
            background_palette_control0: PaletteRegister::new(),
            background_palette_control1: PaletteRegister::new(),
            background_palette_control2: PaletteRegister::new(),
            background_palette_control3: PaletteRegister::new(),
            object_palette_control0: PaletteRegister::new(),
            object_palette_control1: PaletteRegister::new(),
            object_palette_control2: PaletteRegister::new(),
            object_palette_control3: PaletteRegister::new(),
            drawing_framebuffer_1: false,
            current_displaying: DisplayState::None,
            in_drawing: false,
            drawing_cycle_count: 0,
            frame_count: 0,
        }
    }

    pub fn get_bus(&self, address: u32) -> u16 {
        let address = address as usize;

        match address {
            0x0..=0x4_0000 => self.get_vram(address),
            0x5_F800..=0x5_F801 => {
                // INTPND Interrupt pending
                self.interrupt_pending.get()
            }
            0x5_F802..=0x5_F803 => {
                // INTENB Interrupt enable
                self.interrupt_enabled.get()
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
                    self.drawing_framebuffer_1 && self.current_displaying == DisplayState::Left,
                );
                value.set(
                    3,
                    self.drawing_framebuffer_1 && self.current_displaying == DisplayState::Right,
                );
                // Displaying left framebuffer 1
                value.set(
                    4,
                    !self.drawing_framebuffer_1 && self.current_displaying == DisplayState::Left,
                );
                value.set(
                    5,
                    !self.drawing_framebuffer_1 && self.current_displaying == DisplayState::Right,
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
            0x5_F824..=0x5_F825 => self.brightness_control_reg_a as u16,
            0x5_F826..=0x5_F827 => self.brightness_control_reg_b as u16,
            0x5_F828..=0x5_F829 => self.brightness_control_reg_c as u16,
            0x5_F82A..=0x5_F82B => self.led_rest_duration as u16,
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
                value.set(2, self.in_drawing && !self.drawing_framebuffer_1);
                value.set(3, self.in_drawing && self.drawing_framebuffer_1);
                // TODO: Detect when drawing would overrun. OVERTIME
                value.set(4, false);
                value.set(15, self.sbout);

                let value: u16 = value.load();

                value | ((self.sbcount as u16) << 8)
            }
            0x5_F844..=0x5_F845 => {
                // VIP Version
                // Only one version, always 2
                2
            }
            0x5_F848..=0x5_F849 => self.object_control0,
            0x5_F84A..=0x5_F84B => self.object_control1,
            0x5_F84C..=0x5_F84D => self.object_control2,
            0x5_F84E..=0x5_F84F => self.object_control3,
            0x5_F860..=0x5_F861 => self.background_palette_control0.get() as u16,
            0x5_F862..=0x5_F863 => self.background_palette_control1.get() as u16,
            0x5_F864..=0x5_F865 => self.background_palette_control2.get() as u16,
            0x5_F866..=0x5_F867 => self.background_palette_control3.get() as u16,
            0x5_F868..=0x5_F869 => self.object_palette_control0.get() as u16,
            0x5_F86A..=0x5_F86B => self.object_palette_control1.get() as u16,
            0x5_F86C..=0x5_F86D => self.object_palette_control2.get() as u16,
            0x5_F86E..=0x5_F86F => self.object_palette_control3.get() as u16,
            0x5_F870..=0x5_F871 => self.bkcol as u16,
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.get_vram((address & 0x1FFF) + 0x6000)
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.get_vram((address & 0x1FFF) + 0xE000)
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.get_vram((address & 0x1FFF) + 0x1_6000)
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.get_vram((address & 0x1FFF) + 0x1_E000)
            }
            _ => unimplemented!("Read address {:08X}", address),
        }
    }

    fn get_vram(&self, address: usize) -> u16 {
        // Convert byte address to halfword address
        let local_address = address >> 1;

        self.vram[local_address]
    }

    pub fn set_bus(&mut self, address: u32, value: u16) {
        let address = address as usize;

        match address {
            0x0..=0x4_0000 => self.set_vram(address, value),
            0x5_F800..=0x5_F801 => {
                // INTPND Interrupt pending
                // Writes have no effect
            }
            0x5_F802..=0x5_F803 => {
                // INTENB Interrupt enable
                self.interrupt_enabled.set(value)
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

                self.display_enabled = *array.get(1).unwrap();
                self.refresh_ram = *array.get(8).unwrap();
                self.sync_enabled = *array.get(9).unwrap();
                self.lock_column_table = *array.get(10).unwrap();
            }
            0x5_F824..=0x5_F825 => self.brightness_control_reg_a = value as u8,
            0x5_F826..=0x5_F827 => self.brightness_control_reg_b = value as u8,
            0x5_F828..=0x5_F829 => self.brightness_control_reg_c = value as u8,
            0x5_F82A..=0x5_F82B => self.led_rest_duration = value as u8,
            0x5_F82E..=0x5_F82F => self.frmcyc = (value & 0xF) as u8,
            0x5_F842..=0x5_F843 => {
                // XPCTRL Drawing control write register
                let array = BitArray::<_, Lsb0>::new([value]);

                // TODO: Handle
                let reset_drawing = *array.get(0).unwrap();
                self.drawing_enabled = *array.get(1).unwrap();

                self.sbcmp = ((value >> 8) & 0xF) as u8;
            }
            0x5_F848..=0x5_F849 => self.object_control0 = value & 0x3FF,
            0x5_F84A..=0x5_F84B => self.object_control1 = value & 0x3FF,
            0x5_F84C..=0x5_F84D => self.object_control2 = value & 0x3FF,
            0x5_F84E..=0x5_F84F => self.object_control3 = value & 0x3FF,
            0x5_F860..=0x5_F861 => self.background_palette_control0.set(value),
            0x5_F862..=0x5_F863 => self.background_palette_control1.set(value),
            0x5_F864..=0x5_F865 => self.background_palette_control2.set(value),
            0x5_F866..=0x5_F867 => self.background_palette_control3.set(value),
            0x5_F868..=0x5_F869 => self.object_palette_control0.set(value),
            0x5_F86A..=0x5_F86B => self.object_palette_control1.set(value),
            0x5_F86C..=0x5_F86D => self.object_palette_control2.set(value),
            0x5_F86E..=0x5_F86F => self.object_palette_control3.set(value),
            0x5_F870..=0x5_F871 => self.bkcol = (value & 0x3) as u8,
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.set_vram((address & 0xFFF) + 0x6000 / 2, value);
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.set_vram((address & 0xFFF) + 0xE000 / 2, value);
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.set_vram((address & 0xFFF) + 0x1_6000 / 2, value);
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.set_vram((address & 0xFFF) + 0x1_E000 / 2, value);
            }
            _ => unimplemented!("Write address {:08X}", address),
        }
    }

    fn set_vram(&mut self, address: usize, value: u16) {
        // Convert byte address to halfword address
        let local_address = address >> 1;

        self.vram[local_address] = value;
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
                    self.display_framebuffer();

                    self.current_displaying = DisplayState::Left;
                }
                LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End left frame buffer
                    self.interrupt_pending.lfbend = true;

                    self.current_displaying = DisplayState::None;
                }
                FCLK_LOW_CYCLE_OFFSET => {
                    // Lower FCLK
                    self.fclk = false;
                }
                RIGHT_FRAME_BUFFER_CYCLE_OFFSET => {
                    // Render right frame buffer
                    // TODO: Render other eye
                    self.current_displaying = DisplayState::Right;
                }
                RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End right frame buffer
                    self.interrupt_pending.rfbend = true;

                    self.current_displaying = DisplayState::None;
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

                        if self.sbcount == self.sbcmp {
                            // Found rows
                            self.sbout = true;
                            self.sbout_cycle_high_count = 0;

                            // Fire interrupt
                            self.interrupt_pending.sbhit = true;
                        }

                        // TODO: Set and clear SBOUT
                        self.sbcount += 1;
                    } else {
                        self.sbcount = 0;
                        self.interrupt_pending.xpend = true;
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

            if self.sbout {
                // Run SBOUT timer
                if self.sbout_cycle_high_count == SBOUT_HIGH_CYCLE_COUNT {
                    self.sbout = false;
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

        self.interrupt_pending.framestart = true;

        self.frame_count += 1;

        if self.frame_count == self.frmcyc + 1 {
            self.frame_count = 0;

            self.init_drawing_frame();
        }
    }

    fn init_drawing_frame(&mut self) {
        self.interrupt_pending.gamestart = true;

        if self.drawing_enabled {
            // Flip framebuffers to start writing to the currently displayed ones
            self.drawing_framebuffer_1 = !self.drawing_framebuffer_1;

            // Enter render mode
            self.in_drawing = true;
        } else {
            // Immediately mark drawing as ended, as we're not drawing at all
            // TODO: This should actually be after 2.8ms
            self.interrupt_pending.xpend = true;
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
        let clear_pixel = ((self.last_bkcol << 6)
            | (self.last_bkcol << 4)
            | (self.last_bkcol << 2)
            | self.last_bkcol) as u16;
        let clear_pixel = (clear_pixel << 8) | clear_pixel;

        let y = self.sbcount as usize * 8;

        for x in 0..DISPLAY_WIDTH {
            // Overwrite every pixel in the 384x8 segment
            // Pixels are stored in the framebuffer in columns, rather than in rows
            let x_index = x * DISPLAY_HEIGHT;
            // This contains the bottom three bytes as the bit offset
            let pixel_byte_index = (x_index + y) >> 2;
            self.set_vram(left_framebuffer_address + pixel_byte_index, clear_pixel);
            self.set_vram(right_framebuffer_address + pixel_byte_index, clear_pixel);
        }

        // Counter for total object groups
        let mut object_group_counter = 3;

        for i in 31..=0 {
            // Process world
            let world_attribute_address = 0x3_D800 + 16 * i;
            // Convert byte address into halfword addresses so we can grab a slice of memory
            let world_attribute_halfword_address = world_attribute_address >> 1;
            let bytes =
                &self.vram[world_attribute_halfword_address..world_attribute_halfword_address + 10];

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
                    sign_extend_16(self.get_vram(base_address), 13)
                } else {
                    // "The VIP appears to determine the address of HOFSTR by OR'ing the address of HOFSTL with 2.
                    // If the Param Base attribute in the world is not divisibe by 2, this will result in HOFSTL being
                    // used for both the left and right images, and HOFSTR will not be accessed."
                    sign_extend_16(self.get_vram(base_address), 13)
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
                let left_pixel =
                    (self.get_vram(left_framebuffer_address + pixel_byte_index) >> bit_index) & 0x3;

                let right_pixel = (self.get_vram(right_framebuffer_address + pixel_byte_index)
                    >> bit_index)
                    & 0x3;

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
            let character_halfword = self.get_vram(character_address);

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
            let row_halfword = self.get_vram(character_address);

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

            let existing_halfword = self.get_vram(framebuffer_address);
            let halfword = (existing_halfword & removal_mask) | ((pixel as u16) << pixel_shift);
            self.set_vram(framebuffer_address, halfword);
        }
    }

    fn set_intclear(&mut self, value: u16) {
        let mut new_interrupt = VIPInterrupt::new();
        new_interrupt.set(value);

        // Clear any interrupts that are set in `value`
        self.interrupt_pending.scanerr &= new_interrupt.scanerr;
        self.interrupt_pending.lfbend &= new_interrupt.lfbend;
        self.interrupt_pending.rfbend &= new_interrupt.rfbend;
        self.interrupt_pending.gamestart &= new_interrupt.gamestart;

        self.interrupt_pending.framestart &= new_interrupt.framestart;

        self.interrupt_pending.sbhit &= new_interrupt.sbhit;
        self.interrupt_pending.xpend &= new_interrupt.xpend;
        self.interrupt_pending.timeerr &= new_interrupt.timeerr;
    }
}

impl VIPInterrupt {
    fn new() -> Self {
        VIPInterrupt {
            scanerr: false,
            lfbend: false,
            rfbend: false,
            gamestart: false,
            framestart: false,
            sbhit: false,
            xpend: false,
            timeerr: false,
        }
    }

    fn get(&self) -> u16 {
        let mut value = bitarr![u16, Lsb0; 0; 16];
        value.set(0, self.scanerr);
        value.set(1, self.lfbend);
        value.set(2, self.rfbend);
        value.set(3, self.gamestart);

        value.set(4, self.framestart);

        value.set(13, self.sbhit);
        value.set(14, self.xpend);
        value.set(15, self.timeerr);

        value.load()
    }

    fn set(&mut self, value: u16) {
        let array = BitArray::<_, Lsb0>::new([value]);

        self.scanerr = *array.get(0).unwrap();
        self.lfbend = *array.get(1).unwrap();
        self.rfbend = *array.get(2).unwrap();
        self.gamestart = *array.get(3).unwrap();

        self.framestart = *array.get(4).unwrap();

        self.sbhit = *array.get(13).unwrap();
        self.xpend = *array.get(14).unwrap();
        self.timeerr = *array.get(15).unwrap();
    }

    ///
    /// Intersects two sets of interrupt values. If there is at least one intersection (both sides
    /// have a true value), this method will return true.
    fn check_intersection(&self, b: &VIPInterrupt) -> bool {
        (self.scanerr && b.scanerr)
            || (self.lfbend && b.lfbend)
            || (self.rfbend && b.rfbend)
            || (self.gamestart && b.gamestart)
            || (self.framestart && b.framestart)
            || (self.sbhit && b.sbhit)
            || (self.xpend && b.xpend)
            || (self.timeerr && b.timeerr)
    }
}

impl PaletteRegister {
    fn new() -> Self {
        PaletteRegister {
            character1: 0,
            character2: 0,
            character3: 0,
        }
    }

    fn get(&self) -> u16 {
        ((self.character3 << 6) | (self.character2 << 4) | (self.character1 << 2)) as u16
    }

    fn set(&mut self, value: u16) {
        self.character1 = ((value >> 2) & 0x3) as u8;
        self.character2 = ((value >> 4) & 0x3) as u8;
        self.character3 = ((value >> 6) & 0x3) as u8;
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
