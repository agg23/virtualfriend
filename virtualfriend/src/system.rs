use crate::{
    bus::Bus, cartridge::Cartridge, cpu_v810::CpuV810, hardware::Hardware, vip::VIP, vsu::VSU,
};

#[derive(Savefile)]
pub(crate) struct System {
    pub cpu: CpuV810,
    pub bus: Bus,
}

impl System {
    pub fn new(vec: Vec<u8>) -> Self {
        println!("Loading ROM");

        let rom = Cartridge::load_from_vec(vec);

        let cpu = CpuV810::new();

        let vip = VIP::new();
        let vsu = VSU::new();

        let hardware = Hardware::new();
        let bus = Bus::new(rom, vip, vsu, hardware);

        Self { cpu, bus }
    }

    pub fn replace_from_savestate(&mut self, system: System, rom: Vec<u8>) {
        self.cpu = system.cpu;
        self.bus = system.bus;

        self.bus.cart.populate_rom(rom);
    }
}
