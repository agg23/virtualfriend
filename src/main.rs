use std::{
    fs::{File, OpenOptions},
    path::Path,
};

use bus::Bus;
use cpu_v810::CpuV810;
use rom::ROM;

pub mod bus;
pub mod cpu_internals;
mod cpu_v810;
pub mod rom;
mod virtualfriend;

fn main() {
    println!("Started");
    let rom = ROM::load_from_file(Path::new("/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Virtual Boy Wario Land (Japan, USA).vb"));

    let mut cpu = CpuV810::new();
    let mut bus = Bus::new(rom);

    let mut log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("instructions.log")
        .unwrap();
    let mut cycle_count = 0;

    // TODO: Remove
    cpu.debug_init();

    loop {
        cpu.log_instruction(Some(&mut log_file), cycle_count);

        cycle_count += cpu.step(&mut bus);

        if cycle_count >= 1_000_000 {
            break;
        }
    }
}
