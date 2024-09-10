use std::slice::from_raw_parts;

use savefile::{
    Deserialize, Packed, Schema, SchemaPrimitive, Serialize, VecOrStringLayout, WithSchema,
};

use crate::constants::{MAX_ROM_RAM_SIZE, MAX_ROM_SIZE, MIN_ROM_RAM_SIZE};

#[derive(SavefileIntrospectOnly)]
pub struct Cartridge {
    rom: ROM,

    ram: Vec<u16>,

    /// The maximum observed size of the RAM. If `None`, RAM has not been used.
    ram_size: Option<usize>,
}

#[derive(SavefileIntrospectOnly)]
struct ROM {
    // Max 16MB
    // Buffers must be heap allocated, as stack allocation of large buffers causes segfaults on non-x86 platforms
    rom_buffer: Box<[u8]>,

    /// ROM address mask for word addresses
    rom_address_mask: usize,
}

impl ROM {
    fn new(rom_vec: Vec<u8>) -> Self {
        let rom_buffer = rom_vec.into_boxed_slice();

        if rom_buffer.len() > MAX_ROM_SIZE {
            panic!("ROM is too large");
        }

        let rom_address_mask = (rom_buffer.len() / 2) - 1;

        ROM {
            rom_buffer,
            rom_address_mask,
        }
    }
}

impl Cartridge {
    pub fn load_from_vec(vec: Vec<u8>) -> Self {
        // Initialize RAM to 0
        let ram = vec![0; MAX_ROM_RAM_SIZE / 2];

        Cartridge {
            rom: ROM::new(vec),
            ram: ram,
            ram_size: None,
        }
    }

    pub fn populate_rom(&mut self, rom_vec: Vec<u8>) {
        self.rom = ROM::new(rom_vec);
    }

    /// TODO: This is debug init to match with Mednafen
    // pub fn debug_init(&mut self) {
    //     for i in 0..3 {
    //         for j in 0..0x24 {
    //             // Set RAM without updating size
    //             self.ram[i * 0x50 / 2 + j] = 0xFFFF;
    //         }
    //     }
    // }

    pub fn dump_ram(&self) -> Vec<u8> {
        let array = unsafe {
            from_raw_parts(
                self.ram.as_ptr() as *const u8,
                self.ram_size.unwrap_or(self.ram.len() * 2),
            )
        };

        Vec::from(array)
    }

    pub fn load_ram(&mut self, ram: Vec<u8>) {
        let array = unsafe { from_raw_parts(ram.as_ptr() as *const u16, ram.len() / 2) };

        self.load_ram_u16(array);
    }

    fn load_ram_u16(&mut self, ram_array: &[u16]) {
        // Reinit RAM
        self.ram = vec![0; MAX_ROM_RAM_SIZE / 2];

        // Track the highest address in use
        // This lets us shrink saves created in the first release of VirtualFriend
        let mut max_value_address = 0;

        // Copy save words
        for i in 0..ram_array.len() {
            let value = ram_array[i];

            if value > 0 {
                max_value_address = i;
            }

            self.ram[i] = value;
        }

        if ram_array.len() > 0 && max_value_address > 0 {
            self.build_ram_size(max_value_address);
        } else {
            self.ram_size = None;
        }
    }

    pub fn get_rom(&self, address: usize) -> u16 {
        let rom = unsafe {
            from_raw_parts(
                self.rom.rom_buffer.as_ptr() as *const u16,
                self.rom.rom_buffer.len() / 2,
            )
        };

        let address = address & self.rom.rom_address_mask;

        rom[address]
    }

    pub fn get_ram(&mut self, address: usize) -> u16 {
        self.build_ram_size(address);

        self.ram[address]
    }

    pub fn set_ram(&mut self, address: usize, value: u16) {
        self.build_ram_size(address);

        self.ram[address] = value;
    }

    fn build_ram_size(&mut self, address: usize) {
        let size = self.ram_size.unwrap_or(0);

        if address >= size {
            // Address is outside of expected RAM bounds. Increase our internal representation of its size
            // Addresses are 2 byte, and our min size is in bytes
            let mut size = self.ram_size.unwrap_or(MIN_ROM_RAM_SIZE / 2);

            while size <= address {
                // RAM access is out of range. Up the RAM size
                // Size cannot go over 16MB because address will never be larger than that
                // If `address` equals `size`, then the address is at the beginning of the next `size` bank
                size = size * 2;
            }

            self.ram_size = Some(size);
        }
    }
}

impl WithSchema for Cartridge {
    fn schema(_version: u32, _context: &mut savefile::WithSchemaContext) -> Schema {
        Schema::Vector(
            Box::new(Schema::Primitive(SchemaPrimitive::schema_u16)),
            VecOrStringLayout::default(),
        )
    }
}

impl Serialize for Cartridge {
    fn serialize(
        &self,
        serializer: &mut savefile::Serializer<impl std::io::Write>,
    ) -> Result<(), savefile::SavefileError> {
        let length = self.ram_size.unwrap_or(0);
        let mut new_vec = Vec::<u16>::with_capacity(length);

        new_vec.extend(&self.ram[0..length]);
        new_vec.serialize(serializer)?;

        Ok(())
    }
}

impl Deserialize for Cartridge {
    fn deserialize(
        deserializer: &mut savefile::Deserializer<impl std::io::Read>,
    ) -> Result<Self, savefile::SavefileError> {
        let ram = Vec::<u16>::deserialize(deserializer)?;

        let mut cartridge = Cartridge::load_from_vec(vec![]);

        cartridge.load_ram_u16(&ram);

        Ok(cartridge)
    }
}

impl Packed for Cartridge {
    unsafe fn repr_c_optimization_safe(_version: u32) -> savefile::IsPacked {
        savefile::IsPacked::yes()
    }
}
