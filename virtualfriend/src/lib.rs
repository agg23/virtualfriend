use std::{
    collections::VecDeque,
    env,
    fs::{File, OpenOptions},
    io::{self, BufWriter},
};

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

    // writer: BufWriter<File>,
    video_frame_serviced: bool,
    cycle_count: usize,
}

pub struct VideoFrame {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}

pub struct Frame {
    pub video: Option<VideoFrame>,
    pub audio_buffer: Vec<AudioFrame>,
}

struct SimpleAudioFrameSink {
    inner: Vec<AudioFrame>,
}

impl SimpleAudioFrameSink {
    fn new() -> Self {
        SimpleAudioFrameSink { inner: Vec::new() }
    }
}

impl Sink<AudioFrame> for SimpleAudioFrameSink {
    fn append(&mut self, frame: AudioFrame) {
        self.inner.push(frame);
    }
}

impl VirtualFriend {
    pub fn new(vec: Vec<u8>) -> Self {
        println!("Loading ROM");

        let rom = ROM::load_from_vec(vec);

        let mut cpu = CpuV810::new();

        let vip = VIP::new();
        let vsu = VSU::new();

        let hardware = Hardware::new();
        let bus = Bus::new(rom, vip, vsu, hardware);

        let mut temp_dir = env::temp_dir();

        println!("{temp_dir:?}");

        // fs::create_dir_all(&temp_dir).unwrap();
        // temp_dir.push("instructions.log");

        // println!("Logging to {:?}", temp_dir);

        // let log_file = OpenOptions::new()
        //     .write(true)
        //     .create(true)
        //     .open(
        //         // temp_dir
        //         "instructions.log",
        //     )
        //     .unwrap();

        // let writer = BufWriter::with_capacity(4000, log_file);

        cpu.debug_init();

        VirtualFriend {
            cpu,
            bus,
            // writer,
            video_frame_serviced: false,
            cycle_count: 0,
        }
    }

    pub fn run_video_frame(&mut self, inputs: GamepadInputs) -> Frame {
        let mut emu_audio_sink = SimpleAudioFrameSink::new();

        loop {
            self.system_tick(&mut emu_audio_sink, &inputs);

            if self.bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
                // Render framebuffer
                return Frame {
                    video: Some(VideoFrame {
                        left: self.bus.vip.left_rendered_framebuffer.clone(),
                        right: self.bus.vip.right_rendered_framebuffer.clone(),
                    }),
                    audio_buffer: emu_audio_sink.inner,
                };
            }
        }
    }

    pub fn run_audio_frame(&mut self, inputs: GamepadInputs, buffer_size: usize) -> Frame {
        let mut emu_audio_sink = SimpleAudioFrameSink::new();

        let mut buffered_video_frame: Option<VideoFrame> = None;

        loop {
            // self.cpu
            //     .log_instruction(Some(&mut self.writer), self.cycle_count, None);

            self.system_tick(&mut emu_audio_sink, &inputs);

            if self.bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {}

            if self.bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
                if !self.video_frame_serviced {
                    // Render framebuffer
                    self.video_frame_serviced = true;

                    buffered_video_frame = Some(VideoFrame {
                        left: self.bus.vip.left_rendered_framebuffer.clone(),
                        right: self.bus.vip.right_rendered_framebuffer.clone(),
                    });
                }
            } else {
                self.video_frame_serviced = false;
            }

            if emu_audio_sink.inner.len() >= buffer_size {
                // Audio buffer is filled. Return what we have
                return Frame {
                    video: buffered_video_frame,
                    audio_buffer: emu_audio_sink.inner,
                };
            }
        }
    }

    fn system_tick(&mut self, emu_audio_sink: &mut SimpleAudioFrameSink, inputs: &GamepadInputs) {
        let step_cycle_count = self.cpu.step(&mut self.bus);

        self.cycle_count += step_cycle_count;

        if let Some(request) = self.bus.step(step_cycle_count, emu_audio_sink, inputs) {
            self.cpu.request_interrupt(request);
        }
    }
}
