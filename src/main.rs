use std::{fs::OpenOptions, path::Path, rc::Rc, sync::Mutex};

use bus::Bus;
use cpu_v810::CpuV810;
use image::{ImageBuffer, Luma};
use rom::ROM;

use crate::{hardware::Hardware, interrupt::InterruptRequest, timer::Timer, vip::VIP};

pub mod bus;
pub mod constants;
pub mod cpu_internals;
mod cpu_v810;
pub mod gamepad;
pub mod hardware;
pub mod interrupt;
pub mod rom;
pub mod timer;
pub mod util;
pub mod vip;
mod virtualfriend;

fn main() {
    println!("Started");
    let rom = ROM::load_from_file(Path::new("/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Virtual Boy Wario Land (Japan, USA).vb"));

    let mut cpu = CpuV810::new();

    let mut vip = VIP::new();

    let mut hardware = Hardware::new();
    let mut bus = Bus::new(rom, &mut vip, &mut hardware);

    let mut log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("instructions.log")
        .unwrap();
    let mut cycle_count = 0;

    // TODO: Remove
    cpu.debug_init();
    bus.debug_init();

    let mut line_number = 0;

    let mut interrupt_request: Option<InterruptRequest> = None;

    loop {
        cpu.log_instruction(Some(&mut log_file), cycle_count);
        line_number += 1;

        // TODO: Mednafen has a really weird log, execute, interrupt order. This weird arrangement allows matching logs
        if let Some(request) = interrupt_request {
            cpu.request_interrupt(request);
            interrupt_request = None;
            continue;
        }

        let step_cycle_count = cpu.step(&mut bus);

        let mut fake_interrupt: Option<InterruptRequest> = None;

        if line_number == 705314 || line_number == 952618 + 8 {
            // TODO: Remove
            // Force timer to fire to match Mednafen's timing
            while !bus.hardware.timer.step(1) {
                // Run until it fires
            }

            fake_interrupt = Some(InterruptRequest::TimerZero);
        }

        // TODO: Remove
        interrupt_request = if let Some(interrupt_request) = bus.step(step_cycle_count) {
            Some(interrupt_request)
        } else {
            fake_interrupt
        };

        cycle_count += step_cycle_count;
        if cycle_count >= 10_000_000 {
            bus.debug_dump();

            // Dump image
            let mut image = ImageBuffer::<Luma<u8>, _>::new(384, 224);

            let mut pixel_count = 0;

            for x in 0..384 {
                for y in 0..224 {
                    let pixel = vip.left_rendered_framebuffer[(y * 384 + x) as usize];

                    if pixel > 0 {
                        pixel_count += 1;
                    }

                    image.put_pixel(x, y, Luma([pixel]));
                }
            }

            println!(
                "Render enabled: {} {}, pixel count: {pixel_count}",
                vip.drawing_enabled, vip.display_enabled
            );

            image.save("output.png").unwrap();

            break;
        }
    }
}
