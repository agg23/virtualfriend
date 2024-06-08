use rand::{thread_rng, Rng};

use crate::{
    gamepad::GamepadInputs,
    hardware::Hardware,
    interrupt::InterruptRequest,
    rom::ROM,
    vip::VIP,
    vsu::{
        traits::{AudioFrame, Sink},
        VSU,
    },
};

pub struct Bus {
    wram: [u16; 0x1_0000 / 2],
    pub rom: ROM,
    pub vip: VIP,
    vsu: VSU,
    // TODO: Remove pub
    pub hardware: Hardware,
}

impl Bus {
    pub fn new(rom: ROM, vip: VIP, vsu: VSU, hardware: Hardware) -> Self {
        let mut wram = [0; 0x1_0000 / 2];

        // Randomize starting data
        thread_rng().fill(&mut wram[..]);

        Bus {
            wram,
            rom,
            vip,
            vsu,
            hardware,
        }
    }

    /// TODO: This is debug init to match with Mednafen
    pub fn debug_init(&mut self) {
        self.rom.debug_init();

        // Don't randomize WRAM
        self.wram = [0; 0x1_0000 / 2];
    }

    pub fn debug_dump(&self) {
        self.rom.debug_dump();
    }

    pub fn step(
        &mut self,
        cycles_to_run: usize,
        audio_sink: &mut dyn Sink<AudioFrame>,
        inputs: &GamepadInputs,
    ) -> Option<InterruptRequest> {
        let mut request = None;

        self.hardware.gamepad.step(cycles_to_run, inputs);
        self.vsu.step(cycles_to_run, audio_sink);

        // Priority 1
        if self.hardware.timer.step(cycles_to_run) {
            request = Some(InterruptRequest::TimerZero);
        }

        // 4: Highest priority
        if self.vip.step(cycles_to_run) {
            request = Some(InterruptRequest::VIP);
        }

        request
    }

    pub fn get_u16(&mut self, address: u32) -> u16 {
        // Mask top 5 bits to mirror bus
        let address = address as usize & 0x07FF_FFFF;

        match address {
            0x0000_0000..=0x00FF_FFFF => self.vip.get_bus(address),
            // 0x0100_0000..=0x01FF_FFFF => {
            //     // VSU
            //     // All reads are undefined
            //     0
            // }
            0x0200_0000..=0x02FF_FFFF => self.hardware.get(address as u8),
            // 0x0400_0000..=0x04FF_FFFF => {
            //     todo!("Game Pak Expansion")
            // }
            0x0500_0000..=0x05FF_FFFF => self.wram[(address >> 1) & 0x7FFF],
            0x0600_0000..=0x06FF_FFFF => self.rom.get_ram((address >> 1) & 0x7F_FFFF),
            0x0700_0000..=0x07FF_FFFF => self.rom.get_rom((address >> 1) & 0x7F_FFFF),
            _ => 0,
        }
    }

    /// Hack to optimize PC fetch
    pub fn get_rom(&self, address: u32) -> u16 {
        self.rom.get_rom(address as usize)
    }

    pub fn get_u32(&mut self, address: u32) -> u32 {
        let lower = self.get_u16(address) as u32;
        let upper = self.get_u16(address + 2) as u32;

        (upper << 16) | lower
    }

    pub fn get_u8(&mut self, address: u32) -> u8 {
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
        let local_address_u16 = local_address >> 1;

        match address {
            0x0000_0000..=0x00FF_FFFF => self.vip.set_bus(address, value),
            0x0100_0000..=0x01FF_FFFF => self.vsu.set_u8(local_address, value as u8),
            0x0200_0000..=0x02FF_FFFF => self.hardware.set(address as u8, value),
            // 0x0400_0000..=0x04FF_FFFF => {
            //     todo!("Game Pak Expansion")
            // }
            0x0500_0000..=0x05FF_FFFF => self.wram[local_address_u16 & 0x7FFF] = value,
            0x0600_0000..=0x06FF_FFFF => self.rom.set_ram(local_address_u16, value),
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
