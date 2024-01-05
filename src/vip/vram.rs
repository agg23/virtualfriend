pub struct VRAM {
    // We map the entirety of VRAM due to overlapping sections
    // (upper background maps overlap with OAM and properties).
    vram: [u16; 0x4_0000 / 2],
}

impl VRAM {
    pub fn new() -> Self {
        VRAM {
            vram: [0; 0x4_0000 / 2],
        }
    }

    pub fn get_u16(&self, address: usize) -> u16 {
        // Convert byte address to halfword address
        let local_address = address >> 1;

        self.vram[local_address]
    }

    pub fn get_u8(&self, address: usize) -> u8 {
        let word = self.get_u16(address);

        let byte = match address & 0x1 {
            0 => word & 0xFF,
            1 => (word >> 8) & 0xFF,
            _ => unreachable!(),
        };

        byte as u8
    }

    pub fn set_u16(&mut self, address: usize, value: u16) {
        // Convert byte address to halfword address
        let local_address = address >> 1;

        self.vram[local_address] = value;
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        let existing_word = self.get_u16(address);

        let output_word = match address & 0x1 {
            0 => (existing_word & 0xFF00) | (value as u16),
            1 => (existing_word & 0x00FF) | ((value as u16) << 8),
            _ => unreachable!(),
        };

        self.set_u16(address, output_word);
    }

    pub fn slice_mut(
        &mut self,
        start_halfword_address: usize,
        end_halfword_address: usize,
    ) -> &mut [u16] {
        &mut self.vram[start_halfword_address..end_halfword_address]
    }
}
