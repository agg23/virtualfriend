#[derive(Clone, Copy)]
pub struct Waveform {
    ram: [u8; 0x20],
}

impl Waveform {
    pub fn new() -> Self {
        Waveform { ram: [0; 0x20] }
    }

    pub fn get_indexed(&self, index: usize) -> u8 {
        self.ram[index]
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        let address = (address >> 2) & 0x1F;

        // Samples are 6 bits
        self.ram[address] = value & 0x3F;
    }
}
