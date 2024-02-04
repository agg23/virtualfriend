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
#[macro_use]
mod log;
pub mod manifest;
pub mod rom;
pub mod timer;
pub mod util;
pub mod vip;
mod virtualfriend;
pub mod vsu;

pub struct VirtualFriend {
    cpu: CpuV810,
    bus: Bus,

    frame_serviced: bool,
}

pub struct Frame {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}

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

impl VirtualFriend {
    pub fn new(vec: Vec<u8>) -> Self {
        println!("Loading ROM at {rom_path}");

        let rom = ROM::load_from_vec(vec);

        let cpu = CpuV810::new();

        let vip = VIP::new();
        let vsu = VSU::new();

        let hardware = Hardware::new();
        let bus = Bus::new(rom, vip, vsu, hardware);

        VirtualFriend {
            cpu,
            bus,
            frame_serviced: false,
        }
    }

    pub fn run_frame(&mut self, inputs: GamepadInputs) -> Frame {
        let mut emu_audio_sink = SimpleAudioFrameSink::new();

        // let mut log_file = OpenOptions::new()
        //     .write(true)
        //     .create(true)
        //     .open("instructions.log")
        //     .unwrap();

        let mut cycle_count = 0;

        // let mut writer = BufWriter::new(log_file);

        loop {
            // cpu.log_instruction(Some(&mut writer), cycle_count, None);

            let step_cycle_count = self.cpu.step(&mut self.bus);

            cycle_count += step_cycle_count;

            if let Some(request) = self
                .bus
                .step(step_cycle_count, &mut emu_audio_sink, &inputs)
            {
                self.cpu.request_interrupt(request);
                continue;
            }

            if self.bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
                if !self.frame_serviced {
                    // Render framebuffer
                    self.frame_serviced = true;

                    println!("Sending frame");

                    return Frame {
                        left: self.bus.vip.left_rendered_framebuffer.clone(),
                        right: self.bus.vip.right_rendered_framebuffer.clone(),
                    };
                }
            } else {
                self.frame_serviced = false;
            }

            // base_audio_sink.append(emu_audio_sink.inner.as_slices().0);
            emu_audio_sink.inner.clear();
        }
    }
}
