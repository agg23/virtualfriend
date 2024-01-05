use std::{
    fs::OpenOptions, io::BufWriter, num::NonZeroU32, path::Path, rc::Rc, sync::Mutex, thread,
};

use bus::Bus;
use constants::LEFT_FRAME_BUFFER_CYCLE_OFFSET;
use cpu_v810::CpuV810;
use gamepad::GamepadInputs;
use image::{ImageBuffer, Luma};
use rom::ROM;
use single_value_channel::{channel_starting_with, Receiver, Updater};
use softbuffer::{Context, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::Key,
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowBuilder,
};

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

const WINDOW_WIDTH: usize = 512;
const WINDOW_HEIGHT: usize = 256;

struct Frame {
    data: [u8; 384 * 224],
    id: u64,
}

fn main() {
    // Window
    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(
        WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32))
            .build(&event_loop)
            .unwrap(),
    );
    let graphics_context = Context::new(window.clone()).unwrap();
    let mut surface = Surface::new(&graphics_context, window.clone()).unwrap();

    surface
        .resize(NonZeroU32::new(384).unwrap(), NonZeroU32::new(224).unwrap())
        .unwrap();

    // Multithreading
    let mut last_frame_id = 0;
    let (mut buffer_receiver, buffer_transmitter) = channel_starting_with::<Frame>(Frame {
        data: [0; 384 * 224],
        id: last_frame_id,
    });
    let (inputs_receiver, mut inputs_transmitter) =
        channel_starting_with::<GamepadInputs>(GamepadInputs {
            a_button: false,
            b_button: false,
        });

    create_emulator(buffer_transmitter, inputs_receiver);

    event_loop
        .run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);

            let latest_frame = buffer_receiver.latest();
            if latest_frame.id != last_frame_id {
                last_frame_id = latest_frame.id;
                println!("Drawing frame");
                window.request_redraw();
            }

            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } => {
                    let mut buffer = surface.buffer_mut().unwrap();

                    let frame = buffer_receiver.latest();

                    for i in 0..frame.data.len() {
                        let value = frame.data[i] as u32;
                        buffer[i] = (value << 16) | (value << 8) | value;
                    }

                    println!("Updaing buffer");

                    buffer.present().unwrap();
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                } => {
                    if window_id == window.id() {
                        window_target.exit();
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        },
                } => {
                    if window_id != window.id() {
                        return;
                    }

                    let mut inputs = GamepadInputs {
                        a_button: false,
                        b_button: false,
                    };

                    match event.key_without_modifiers().as_ref() {
                        Key::Character("x") => {
                            inputs.a_button = event.state == ElementState::Pressed;
                        }
                        Key::Character("z") => {
                            inputs.b_button = event.state == ElementState::Pressed;
                        }
                        _ => {}
                    }

                    inputs_transmitter.update(inputs).unwrap();
                }
                _ => {}
            }
        })
        .unwrap();
}

fn create_emulator(
    buffer_transmitter: Updater<Frame>,
    mut inputs_receiver: Receiver<GamepadInputs>,
) {
    thread::spawn(move || {
        let rom = ROM::load_from_file(Path::new("/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Virtual Boy Wario Land (Japan, USA).vb"));

        let mut cpu = CpuV810::new();

        let mut vip = VIP::new();

        let mut hardware = Hardware::new();
        let mut bus = Bus::new(rom, &mut vip, &mut hardware);

        let mut frame_id = 1;

        let mut inputs = inputs_receiver.latest();

        // let mut log_file = OpenOptions::new()
        //     .write(true)
        //     .create(true)
        //     .open("instructions.log")
        //     .unwrap();

        let mut cycle_count = 0;

        // let mut writer = BufWriter::new(log_file);
        let mut line_count = 0;

        let mut frame_serviced = false;

        loop {
            // cpu.log_instruction(Some(&mut writer), cycle_count, None);
            line_count += 1;

            // if line_count == 60_400_000 {
            //     println!("Halting");
            //     return;
            // }

            let step_cycle_count = cpu.step(&mut bus);

            cycle_count += step_cycle_count;

            if let Some(request) = bus.step(step_cycle_count, &inputs) {
                cpu.request_interrupt(request);
                continue;
            }

            if bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
                if !frame_serviced {
                    // Render framebuffer
                    frame_serviced = true;

                    buffer_transmitter
                        .update(Frame {
                            data: bus.vip.left_rendered_framebuffer.clone(),
                            id: frame_id,
                        })
                        .unwrap();

                    inputs = inputs_receiver.latest();

                    frame_id += 1;
                }
            } else {
                frame_serviced = false;
            }
        }
    });
}
