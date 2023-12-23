use rand::{thread_rng, Rng};

use crate::rom::ROM;

pub struct Bus {
    wram: [u32; 0x1_0000 / 4],
    rom: ROM,
}

impl Bus {
    pub fn new(rom: ROM) -> Self {
        let mut wram = [0; 0x1_0000 / 4];

        // Randomize starting data
        thread_rng().fill(&mut wram[..]);

        Bus { wram, rom }
    }

    pub fn get_u32(&self, address: u32) -> u32 {
        // Mask top 5 bits to mirror bus
        let address = address & 0x07FF_FFFF;

        // Address for bus block
        let local_address = (address as usize) & 0xFF_FFFF;
        // Remove bottom 2 (shifted out) to make word addresses
        let local_address = local_address >> 2;

        match address {
            0x0000_0000..=0x00FF_FFFF => {
                todo!("VIP")
            }
            0x0100_0000..=0x01FF_FFFF => {
                todo!("VSU")
            }
            0x0200_0000..=0x02FF_FFFF => {
                todo!("Miscellaneous Hardware")
            }
            0x0400_0000..=0x04FF_FFFF => {
                todo!("Game Pak Expansion")
            }
            0x0500_0000..=0x05FF_FFFF => self.wram[local_address],
            0x0600_0000..=0x06FF_FFFF => self.rom.ram[local_address],
            0x0700_0000..=0x07FF_FFFF => self.rom.get_rom(local_address),
            _ => 0,
        }
    }

    pub fn get_u16(&self, address: u32) -> u16 {
        let word = self.get_u32(address);

        let halfword = if address & 0x2 != 0 {
            word >> 16
        } else {
            word & 0xFFFF
        };

        halfword as u16
    }

    pub fn get_u8(&self, address: u32) -> u8 {
        let word = self.get_u32(address);

        let byte = match address & 0x3 {
            0 => word & 0xFF,
            1 => (word >> 8) & 0xFF,
            2 => (word >> 16) & 0xFF,
            3 => word >> 24,
            _ => unreachable!(),
        };

        byte as u8
    }

    pub fn set_u32(&mut self, address: u32, value: u32) {
        // Mask top 5 bits to mirror bus, and bottom 2 to word addresses
        let address = address & 0x07FF_FFFC;

        match address {
            0x0000_0000..=0x00FF_FFFF => {
                todo!("VIP")
            }
            0x0100_0000..=0x01FF_FFFF => {
                todo!("VSU")
            }
            0x0200_0000..=0x02FF_FFFF => {
                todo!("Miscellaneous Hardware")
            }
            0x0400_0000..=0x04FF_FFFF => {
                todo!("Game Pak Expansion")
            }
            0x0500_0000..=0x05FF_FFFF => self.wram[(address as usize) & 0xFF_FFFF] = value,
            0x0600_0000..=0x06FF_FFFF => self.rom.ram[address as usize] = value,
            0x0700_0000..=0x07FF_FFFF => {
                // Game Pak ROM
                // Do nothing
            }
            _ => {}
        }
    }

    pub fn set_u16(&mut self, address: u32, value: u16) {
        let existing_word = self.get_u32(address);

        let output_word = if address & 1 != 0 {
            (existing_word & 0xFFFF) | ((value as u32) << 16)
        } else {
            (existing_word & 0xFFFF_0000) | (value as u32)
        };

        self.set_u32(address, output_word);
    }

    pub fn set_u8(&mut self, address: u32, value: u8) {
        let existing_word = self.get_u32(address);

        let output_word = match address & 0x3 {
            0 => (existing_word & 0xFFFF_FF00) | (value as u32),
            1 => (existing_word & 0xFFFF_00FF) | ((value as u32) << 8),
            2 => (existing_word & 0xFF00_FFFF) | ((value as u32) << 16),
            3 => (existing_word & 0x00FF_FFFF) | ((value as u32) << 24),
            _ => unreachable!(),
        };

        self.set_u32(address, output_word);
    }
}
