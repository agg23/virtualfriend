use crate::constants::{
    FCLK_LOW_CYCLE_OFFSET, FRAME_COMPLETE_CYCLE_OFFSET, LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET,
    LEFT_FRAME_BUFFER_CYCLE_OFFSET, RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET,
    RIGHT_FRAME_BUFFER_CYCLE_OFFSET,
};

pub struct VIP {
    /// Four sets of 512 characters (16 bytes each)
    // character_tables: [[u32; 512 * 4]; 4],

    // /// 1024 objects of 8 bytes each
    // oam: [u32; 1024 * 2],

    // background_map_and_params: [u32; 0x7600],
    current_display_clock_cycle: u32,

    // We map the entirety of VRAM due to overlapping sections
    // (upper background maps overlap with OAM and properties)
    vram: [u32; 0x40000],

    fclk: bool,
}

impl VIP {
    pub fn get(&self, address: u32) -> u32 {
        let address = address as usize;

        let word_address = address >> 2;

        match address {
            0x0..=0x4_0000 => self.vram[word_address],
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)]
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)]
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)]
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)]
            }
        }
    }

    pub fn set(&self, address: u32, value: u32) {
        let address = address as usize;

        let word_address = address >> 2;

        match address {
            0x0..=0x4_0000 => self.vram[word_address] = value,
            0x7_8000..=0x7_9FFF => {
                // Character table 1 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)] = value
            }
            0x7_A000..=0x7_BFFF => {
                // Character table 2 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)] = value
            }
            0x7_C000..=0x7_DFFF => {
                // Character table 3 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)] = value
            }
            0x7_E000..=0x7_FFFF => {
                // Character table 4 remap
                self.vram[(word_address & 0x7FF) + (0x6000 >> 2)] = value
            }
        }
    }

    pub fn run_for_cycles(&mut self, cycles_to_run: usize) {
        for _ in 0..cycles_to_run {
            match self.current_display_clock_cycle {
                0 => {
                    // Raise FCLK
                    self.fclk = true;
                }
                LEFT_FRAME_BUFFER_CYCLE_OFFSET => {
                    // Render left frame buffer
                    // TODO: Perform logic for groups of 4 pixels
                    // See Display Procedure
                }
                LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET => {
                    // End left frame buffer
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
                }
                FRAME_COMPLETE_CYCLE_OFFSET => {
                    // End frame
                }
                _ => {}
            }

            if self.current_display_clock_cycle == FRAME_COMPLETE_CYCLE_OFFSET {
                self.current_display_clock_cycle = 0;
            } else {
                self.current_display_clock_cycle += 1;
            }
        }
    }
}
