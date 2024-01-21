use std::{collections::VecDeque, path::Path};

use bus::Bus;
use cpu_v810::CpuV810;
use hardware::Hardware;
use vip::VIP;
use vsu::{
    traits::{AudioFrame, Sink},
    VSU,
};

use crate::{constants::LEFT_FRAME_BUFFER_CYCLE_OFFSET, gamepad::GamepadInputs, rom::ROM};

pub mod bus;
pub mod constants;
pub mod cpu_internals;
pub mod cpu_v810;
pub mod gamepad;
pub mod hardware;
pub mod interrupt;
pub mod rom;
pub mod timer;
pub mod util;
pub mod vip;
mod virtualfriend;
pub mod vsu;

struct SimpleAudioFrameSink {
    inner: VecDeque<AudioFrame>,
}

impl SimpleAudioFrameSink {
    fn new() -> Self {
        SimpleAudioFrameSink {
            inner: VecDeque::new(),
        }
    }
}

impl Sink<AudioFrame> for SimpleAudioFrameSink {
    fn append(&mut self, frame: AudioFrame) {
        self.inner.push_back(frame);
    }
}

pub fn run(rom_path: String) {
    println!("Loading ROM at {rom_path}");

    let mut emu_audio_sink = SimpleAudioFrameSink::new();

    let rom = ROM::load_from_file(Path::new(&rom_path));

    let mut cpu = CpuV810::new();

    let mut vip = VIP::new();
    let mut vsu = VSU::new();

    let mut hardware = Hardware::new();
    let mut bus = Bus::new(rom, &mut vip, &mut vsu, &mut hardware);

    let mut frame_id = 1;

    // TODO: Poll for input
    let mut inputs = GamepadInputs {
        a_button: false,
        b_button: false,
    };

    // let mut log_file = OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .open("instructions.log")
    //     .unwrap();

    let mut cycle_count = 0;

    // let mut writer = BufWriter::new(log_file);

    let mut frame_serviced = false;

    loop {
        // cpu.log_instruction(Some(&mut writer), cycle_count, None);

        let step_cycle_count = cpu.step(&mut bus);

        cycle_count += step_cycle_count;

        if let Some(request) = bus.step(step_cycle_count, &mut emu_audio_sink, &inputs) {
            cpu.request_interrupt(request);
            continue;
        }

        if bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
            if !frame_serviced {
                // Render framebuffer
                frame_serviced = true;

                // buffer_transmitter
                //     .update(Frame {
                //         left: bus.vip.left_rendered_framebuffer.clone(),
                //         right: bus.vip.right_rendered_framebuffer.clone(),
                //         id: frame_id,
                //     })
                //     .unwrap();

                // inputs = inputs_receiver.latest();

                frame_id += 1;
            }
        } else {
            frame_serviced = false;
        }

        // base_audio_sink.append(emu_audio_sink.inner.as_slices().0);
        emu_audio_sink.inner.clear();
    }
}
