use rand::{thread_rng, Rng};

pub struct WRAM {
    ram: [u16; 0x1_0000 * 2],
}

impl WRAM {
    pub fn new() -> Self {
        let mut ram = [0; 0x1_0000 * 2];

        // Randomize starting data
        thread_rng().fill(&mut ram[..]);

        WRAM { ram }
    }

    pub fn get_u16(&self, address: u32) -> u16 {
        self.ram[word_16_address(address)]
    }

    pub fn get_u32(&self, address: u32) -> u32 {
        let address = word_16_address(address);

        // TODO: Should we unsafely treat the array as a u32 array?
        let lower = self.ram[address] as u32;
        let upper = self.ram[address + 1] as u32;

        upper << 16 | lower
    }

    pub fn set_u8(&mut self, address: u32, value: u8) {
        let existing_value = &mut self.ram[word_16_address(address)];

        if address & 1 != 0 {
            // Upper byte
            *existing_value = ((value as u16) << 8) | (*existing_value & 0xFF);
        } else {
            // Lower byte
            *existing_value = (*existing_value & 0xFF00) | (value as u16);
        }
    }

    pub fn set_u16(&mut self, address: u32, value: u16) {
        let existing_value = &mut self.ram[word_16_address(address)];

        *existing_value = value;
    }

    pub fn set_u32(&mut self, address: u32, value: u32) {
        let address = word_16_address(address);

        let lower_value = &mut self.ram[address];
        *lower_value = (value & 0xFFFF) as u16;

        let upper_value = &mut self.ram[address + 1];
        *upper_value = ((value >> 16) & 0xFFFF) as u16;
    }
}

fn word_16_address(address: u32) -> usize {
    ((address as usize) & 0x1FFFE) >> 1
}
