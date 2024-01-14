#[derive(Clone, Copy)]
pub struct Waveform {
    ram: [u8; 0x80],
}

impl Waveform {
    pub fn new() -> Self {
        Waveform { ram: [0; 0x80] }
    }

    pub fn get_indexed(&self, index: usize) -> u8 {
        self.ram[index]
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        let address = (address >> 2) & 0x7F;

        self.ram[address] = value;
    }
}
