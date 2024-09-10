extern crate savefile;

#[macro_use]
extern crate savefile_derive;

use savestates::{savestate::UnparsedSavestate, SavestateController};
use system::System;
use vsu::traits::{AudioFrame, Sink};

use crate::{constants::LEFT_FRAME_BUFFER_CYCLE_OFFSET, gamepad::GamepadInputs};

mod bus;
mod cartridge;
mod constants;
mod cpu_internals;
mod cpu_v810;
pub mod gamepad;
mod hardware;
mod interrupt;
#[macro_use]
mod log;
pub mod manifest;
pub mod savestates;
mod system;
mod timer;
mod util;
mod vip;
pub mod vsu;

pub struct VirtualFriend {
    system: System,

    rom: Vec<u8>,

    savestate: SavestateController,

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
    pub fn new(rom: Vec<u8>) -> Self {
        println!("Loading ROM");

        // TODO: Ideally we don't maintain two copies of the ROM in RAM for savestate loading
        let system = System::new(rom.clone());

        let savestate = SavestateController::new();

        // let mut temp_dir = env::temp_dir();

        // println!("{temp_dir:?}");

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

        // cpu.debug_init();

        Self {
            system,
            rom,
            savestate,
            // writer,
            video_frame_serviced: false,
            cycle_count: 0,
        }
    }

    pub fn run_video_frame(&mut self, inputs: GamepadInputs) -> Frame {
        let mut emu_audio_sink = SimpleAudioFrameSink::new();

        loop {
            self.system_tick(&mut emu_audio_sink, &inputs);

            if let Some(frame) = self.frame_tick() {
                return Frame {
                    video: Some(frame),
                    audio_buffer: emu_audio_sink.inner,
                };
            }
        }
    }

    pub fn run_audio_frame(&mut self, inputs: GamepadInputs, buffer_size: usize) -> Frame {
        if buffer_size == 0 {
            panic!("Invalid buffer_size {buffer_size}");
        }

        let mut emu_audio_sink = SimpleAudioFrameSink::new();

        let mut buffered_video_frame: Option<VideoFrame> = None;

        loop {
            // self.cpu
            //     .log_instruction(Some(&mut self.writer), self.cycle_count, None);

            self.system_tick(&mut emu_audio_sink, &inputs);

            if let Some(frame) = self.frame_tick() {
                buffered_video_frame = Some(frame);
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

    pub fn run_rewind_frame(&mut self) -> Option<VideoFrame> {
        if let Some(savestate) = self.savestate.rewind_tick() {
            self.system
                .replace_from_savestate(savestate.contents(), self.rom.clone());

            return Some(VideoFrame {
                left: savestate.left_frame,
                right: savestate.right_frame,
            });
        }

        None
    }

    pub fn load_ram(&mut self, ram: Vec<u8>) {
        self.system.bus.cart.load_ram(ram)
    }

    pub fn dump_ram(&self) -> Vec<u8> {
        let ram = self.system.bus.cart.dump_ram();

        println!("Dumping RAM {:X}", ram.len());

        ram
    }

    pub fn create_savestate(&mut self) -> UnparsedSavestate {
        UnparsedSavestate::build(&self.system)
    }

    pub fn load_savestate(&mut self, savestate: &UnparsedSavestate) {
        let system = self.savestate.load_savestate_to_system(savestate);

        self.system.replace_from_savestate(system, self.rom.clone());
    }

    // TODO: This should be failable
    pub fn load_savestate_from_bytes(&mut self, bytes: &[u8]) {
        let savestate = UnparsedSavestate::load(bytes);

        self.load_savestate(&savestate);
    }

    fn system_tick(&mut self, emu_audio_sink: &mut SimpleAudioFrameSink, inputs: &GamepadInputs) {
        let step_cycle_count = self.system.cpu.step(&mut self.system.bus);

        self.cycle_count += step_cycle_count;

        if let Some(request) = self
            .system
            .bus
            .step(step_cycle_count, emu_audio_sink, inputs)
        {
            self.system.cpu.request_interrupt(request);
        }
    }

    fn frame_tick(&mut self) -> Option<VideoFrame> {
        if self.system.bus.vip.current_display_clock_cycle < LEFT_FRAME_BUFFER_CYCLE_OFFSET {
            if !self.video_frame_serviced {
                // Render framebuffer
                self.video_frame_serviced = true;

                self.savestate.frame_tick(&self.system);

                return Some(VideoFrame {
                    left: self.system.bus.vip.left_rendered_framebuffer.clone(),
                    right: self.system.bus.vip.right_rendered_framebuffer.clone(),
                });
            }
        } else {
            self.video_frame_serviced = false;
        }

        None
    }
}
