use std::{
    fs::{self},
    path::Path,
    slice::from_raw_parts,
};

const MAX_ROM_SIZE: usize = 0x100_0000;

pub struct ROM {
    // Max 16MB
    // Buffers must be heap allocated, as stack allocation of large buffers causes segfaults on non-x86 platforms
    rom_buffer: Box<[u8]>,

    /// ROM address mask for word addresses
    rom_address_mask: usize,
    pub rom: &'static [u16],
    pub ram: Box<[u16]>,
}

impl ROM {
    pub fn load_from_file(path: &Path) -> Self {
        let rom_buffer = fs::read(path)
            .expect("Could not find file")
            .into_boxed_slice();

        if rom_buffer.len() > MAX_ROM_SIZE {
            panic!("ROM is too large");
        }

        let rom =
            unsafe { from_raw_parts(rom_buffer.as_ptr() as *const u16, rom_buffer.len() / 2) };

        let rom_address_mask = (rom_buffer.len() / 2) - 1;

        let ram = vec![0; MAX_ROM_SIZE / 2].into_boxed_slice();

        ROM {
            rom_buffer,
            rom_address_mask,
            rom,
            // TODO: Implement save loading
            ram,
        }
    }

    pub fn get_rom(&self, address: usize) -> u16 {
        let address = address & self.rom_address_mask;

        self.rom[address]
    }
}
