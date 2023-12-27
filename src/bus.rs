use rand::{thread_rng, Rng};

use crate::{hardware::Hardware, rom::ROM, vip::VIP};

pub struct Bus<'a> {
    wram: [u16; 0x1_0000 / 2],
    rom: ROM,
    vip: &'a mut VIP,
    hardware: &'a mut Hardware<'a>,
}

impl<'a> Bus<'a> {
    pub fn new(rom: ROM, vip: &'a mut VIP, hardware: &'a mut Hardware<'a>) -> Self {
        let mut wram = [0; 0x1_0000 / 2];

        // Randomize starting data
        thread_rng().fill(&mut wram[..]);

        Bus {
            wram,
            rom,
            vip,
            hardware,
        }
    }

    pub fn get_u16(&self, address: u32) -> u16 {
        // Mask top 5 bits to mirror bus
        let address = address & 0x07FF_FFFF;

        // Address for bus block
        let local_address = (address as usize) & 0xFF_FFFF;
        // Remove bottom 1 (shifted out) to make halfword addresses
        let local_address = local_address >> 1;

        match address {
            0x0000_0000..=0x00FF_FFFF => self.vip.get_bus(address),
            0x0100_0000..=0x01FF_FFFF => {
                // todo!("VSU")
                0
            }
            0x0200_0000..=0x02FF_FFFF => self.hardware.get(address as u8),
            0x0400_0000..=0x04FF_FFFF => {
                todo!("Game Pak Expansion")
            }
            0x0500_0000..=0x05FF_FFFF => self.wram[local_address],
            0x0600_0000..=0x06FF_FFFF => self.rom.ram[local_address],
            0x0700_0000..=0x07FF_FFFF => self.rom.get_rom(local_address),
            _ => 0,
        }
    }

    pub fn get_u32(&self, address: u32) -> u32 {
        let lower = self.get_u16(address) as u32;
        let upper = self.get_u16(address + 2) as u32;

        (upper << 16) | lower
    }

    pub fn get_u8(&self, address: u32) -> u8 {
        let word = self.get_u16(address);

        let byte = match address & 0x1 {
            0 => word & 0xFF,
            1 => (word >> 8) & 0xFF,
            _ => unreachable!(),
        };

        byte as u8
    }

    pub fn set_u16(&mut self, address: u32, value: u16) {
        // Mask top 5 bits to mirror bus
        let address = address & 0x07FF_FFFF;

        // Address for bus block
        let local_address = (address as usize) & 0xFF_FFFF;
        // Remove bottom 1 (shifted out) to make halfword addresses
        let local_address = local_address >> 1;

        match address {
            0x0000_0000..=0x00FF_FFFF => self.vip.set_bus(address, value),
            0x0100_0000..=0x01FF_FFFF => {
                // todo!("VSU")
            }
            0x0200_0000..=0x02FF_FFFF => self.hardware.set(address as u8, value),
            0x0400_0000..=0x04FF_FFFF => {
                todo!("Game Pak Expansion")
            }
            0x0500_0000..=0x05FF_FFFF => self.wram[local_address] = value,
            0x0600_0000..=0x06FF_FFFF => self.rom.ram[local_address] = value,
            0x0700_0000..=0x07FF_FFFF => {
                // Game Pak ROM
                // Do nothing
            }
            _ => {}
        }
    }

    pub fn set_u32(&mut self, address: u32, value: u32) {
        let upper = (value >> 16) as u16;
        let lower = (value & 0xFFFF) as u16;

        self.set_u16(address, lower);
        self.set_u16(address + 2, upper);
    }

    pub fn set_u8(&mut self, address: u32, value: u8) {
        let existing_word = self.get_u16(address);

        let output_word = match address & 0x1 {
            0 => (existing_word & 0xFF00) | (value as u16),
            1 => (existing_word & 0x00FF) | ((value as u16) << 8),
            _ => unreachable!(),
        };

        self.set_u16(address, output_word);
    }
}
