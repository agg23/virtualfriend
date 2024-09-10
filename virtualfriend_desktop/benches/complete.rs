use std::{collections::VecDeque, path::Path};

use virtualfriend::{
    bus::Bus,
    cartridge::ROM,
    cpu_v810::CpuV810,
    gamepad::GamepadInputs,
    hardware::Hardware,
    vip::VIP,
    vsu::{
        traits::{AudioFrame, Sink},
        VSU,
    },
};

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

fn main() {
    let rom = ROM::load_from_file(Path::new(
        "/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Mario's Tennis (Japan, USA).vb",
    ));

    let mut audio_sink = SimpleAudioFrameSink::new();

    let mut cpu = CpuV810::new();

    let mut vip = VIP::new();
    let mut vsu = VSU::new();

    let mut hardware = Hardware::new();
    let mut bus = Bus::new(rom, &mut vip, &mut vsu, &mut hardware);

    let mut inputs = GamepadInputs {
        a_button: false,
        b_button: false,
    };

    let mut line_count = 0;

    loop {
        line_count += 1;

        if line_count % 20_000_000 == 0 {
            inputs.a_button = !inputs.a_button;
        } else if line_count == 200_000_000 {
            return;
        }

        let step_cycle_count = cpu.step(&mut bus);

        if let Some(request) = bus.step(step_cycle_count, &mut audio_sink, &inputs) {
            cpu.request_interrupt(request);
            continue;
        }

        audio_sink.inner.clear();
    }
}
