mod audio_driver;
mod linear_resampler;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use audio_driver::AudioDriver;
use virtualfriend::{gamepad::GamepadInputs, Frame, VideoFrame};

pub struct VirtualFriend {
    core: Arc<Mutex<virtualfriend::VirtualFriend>>,
    audio_driver: AudioDriver,
}

impl VirtualFriend {
    pub fn new<F>(rom_data: &[u8], frame_callback: F) -> Self
    where
        F: Fn(Option<VideoFrame>) -> GamepadInputs + Send + 'static,
    {
        let core = Arc::new(Mutex::new(virtualfriend::VirtualFriend::new(
            rom_data.to_vec(),
        )));

        let core_audio = core.clone();

        let mut last_input = GamepadInputs {
            a_button: false,
            b_button: false,

            right_trigger: false,
            left_trigger: false,

            right_dpad_up: false,
            right_dpad_right: false,
            right_dpad_left: false,
            right_dpad_down: false,

            left_dpad_right: false,
            left_dpad_left: false,
            left_dpad_down: false,
            left_dpad_up: false,

            start: false,
            select: false,
        };

        // 41.667kHz
        let mut audio_driver = AudioDriver::new(41667, 20, move |sample_count| {
            let frame = core_audio
                .lock()
                .unwrap()
                .run_audio_frame(last_input, sample_count);

            last_input = frame_callback(frame.video);

            // if let Some(video) = frame.video {
            // // Send updated video frame
            // frame_id += 1;

            // if buffer_transmitter
            //     .update(ThreadFrame::from(video, frame_id))
            //     .is_err()
            // {
            //     println!("Could not update frame");
            // };
            // }

            VecDeque::from(frame.audio_buffer)
        });

        audio_driver.stop();

        VirtualFriend { core, audio_driver }
    }

    pub fn load_ram(&mut self, ram: &[u8]) {
        self.core.try_lock().expect("Could not acquire mutex lock. Emulator host is misconfigured; is it running on multiple threads?").load_ram(ram.to_vec());
    }

    pub fn save_ram(&self) -> Vec<u8> {
        self.core.try_lock().expect("Could not acquire mutex lock. Emulator host is misconfigured; is it running on multiple threads?").dump_ram()
    }

    pub fn run_audio_frame(&mut self, inputs: GamepadInputs, buffer_size: usize) -> Frame {
        self.core.try_lock().expect("Could not acquire mutex lock. Emulator host is misconfigured; is it running on multiple threads?").run_audio_frame(inputs.into(), buffer_size).into()
    }

    pub fn start(&mut self) {
        self.audio_driver.play();
    }
}
