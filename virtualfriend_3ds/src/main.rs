use std::{path::Path, slice::from_raw_parts_mut};

use ctru::{
    prelude::*,
    services::{
        gfx::{Flush, Screen, Swap, TopScreen3D},
        gspgpu::FramebufferFormat,
    },
};
use virtualfriend::{
    bus::Bus,
    constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH, LEFT_FRAME_BUFFER_CYCLE_OFFSET},
    cpu_v810::CpuV810,
    gamepad::GamepadInputs,
    hardware::Hardware,
    rom::ROM,
    vip::VIP,
};

const DISPLAY_WIDTH_3DS: usize = 400;
const DISPLAY_HEIGHT_3DS: usize = 240;

fn panic_hook_setup() {
    use std::panic::PanicInfo;

    let main_thread = std::thread::current().id();

    // Panic Hook setup
    let default_hook = std::panic::take_hook();
    let new_hook = Box::new(move |info: &PanicInfo| {
        default_hook(info);

        // Only for panics in the main thread
        if main_thread == std::thread::current().id() && Console::exists() {
            println!("\nThe software will exit in 5 seconds");

            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    });
    std::panic::set_hook(new_hook);
}

fn main() {
    panic_hook_setup();

    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();

    // We need to use network sockets to send the data stream back.
    let mut soc = Soc::new().expect("Couldn't obtain SOC controller");

    soc.redirect_to_3dslink(true, true)
        .expect("Cannot connect to server");

    println!("Hello, World!");
    println!("\x1b[29;16HPress Start to exit");

    let _romfs = ctru::services::romfs::RomFS::new().unwrap();

    let rom = ROM::load_from_file(Path::new("romfs:/Mario's Tennis (Japan, USA).vb"));

    let mut cpu = CpuV810::new();

    let mut vip = VIP::new();

    let mut hardware = Hardware::new();
    let mut bus = Bus::new(rom, &mut vip, &mut hardware);

    gfx.top_screen.borrow_mut().set_double_buffering(true);

    let mut top_screen = TopScreen3D::from(&gfx.top_screen);

    let mut frame_serviced = false;

    while apt.main_loop() {
        // gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            println!("Exiting");
            break;
        }

        let inputs = GamepadInputs {
            a_button: hid.keys_held().contains(KeyPad::A),
            b_button: hid.keys_held().contains(KeyPad::B),
        };

        let step_cycle_count = cpu.step(&mut bus);

        if let Some(request) = bus.step(step_cycle_count, &inputs) {
            cpu.request_interrupt(request);
            continue;
        }

        if bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
            if !frame_serviced {
                // Render framebuffer
                println!("Render frame");
                frame_serviced = true;

                let (mut left, mut right) = top_screen.split_mut();
                left.set_framebuffer_format(FramebufferFormat::Bgr8);
                right.set_framebuffer_format(FramebufferFormat::Bgr8);

                let left_framebuffer = left.raw_framebuffer();
                let left_slice = unsafe {
                    from_raw_parts_mut(
                        left_framebuffer.ptr,
                        left_framebuffer.height * left_framebuffer.width * 4,
                    )
                };

                let right_framebuffer = right.raw_framebuffer();
                let right_slice = unsafe {
                    from_raw_parts_mut(
                        right_framebuffer.ptr,
                        right_framebuffer.height * right_framebuffer.width * 4,
                    )
                };

                for y in 0..DISPLAY_HEIGHT {
                    for x in 0..DISPLAY_WIDTH {
                        let left_value = bus.vip.left_rendered_framebuffer[y * DISPLAY_WIDTH + x];
                        let right_value = bus.vip.right_rendered_framebuffer[y * DISPLAY_WIDTH + x];

                        // Graphics have inverted axises and directions
                        // Third byte is the red
                        let address = ((DISPLAY_HEIGHT_3DS - y) + DISPLAY_HEIGHT_3DS * x) * 3 + 2;

                        left_slice[address] = left_value;
                        right_slice[address] = right_value;
                    }
                }

                drop(left);
                drop(right);

                println!("Flushing");

                top_screen.flush_buffers();

                println!("Swapping");
                top_screen.swap_buffers();
            }
        } else {
            frame_serviced = false;
        }
    }
}
