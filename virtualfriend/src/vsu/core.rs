use crate::constants::SOUND_SAMPLE_RATE_CYCLE_COUNT;

use super::{
    channel_enum::ChannelType,
    traits::{AudioFrame, Sink},
    waveform::Waveform,
};

pub struct VSU {
    waveforms: [Waveform; 5],
    modulation: [i8; 32],
    channels: [ChannelType; 6],

    sample_output_counter: usize,
}

impl VSU {
    pub fn new() -> Self {
        VSU {
            waveforms: [Waveform::new(); 5],
            modulation: [0; 32],
            channels: [
                ChannelType::new_pcm(),
                ChannelType::new_pcm(),
                ChannelType::new_pcm(),
                ChannelType::new_pcm(),
                ChannelType::new_pcm_ch5(),
                ChannelType::new_noise(),
            ],

            sample_output_counter: 0,
        }
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        match address {
            0x0..=0x7F => {
                if !self.playback_occuring() {
                    self.waveforms[0].set_u8(address, value)
                }
            }
            0x80..=0xFF => {
                if !self.playback_occuring() {
                    self.waveforms[1].set_u8(address, value);
                }
            }
            0x100..=0x17F => {
                if !self.playback_occuring() {
                    self.waveforms[2].set_u8(address, value)
                }
            }
            0x180..=0x1FF => {
                if !self.playback_occuring() {
                    self.waveforms[3].set_u8(address, value)
                }
            }
            0x200..=0x27F => {
                if !self.playback_occuring() {
                    self.waveforms[4].set_u8(address, value)
                }
            }
            0x280..=0x2FF => {
                if !self.channels[4].channel().enable_playback {
                    self.modulation[(address - 0x280) >> 2] = value as i8;
                }
            }
            0x580 => {
                // SSTOP Stop all sound register
                if value & 0x1 != 0 {
                    for channel in &mut self.channels {
                        channel.channel_mut().enable_playback = false;
                    }
                }
            }
            _ => self.send_channel_write(address, value),
        }
    }

    fn playback_occuring(&self) -> bool {
        self.channels
            .iter()
            .any(|channel| channel.channel().enable_playback)
    }

    fn send_channel_write(&mut self, address: usize, value: u8) {
        let register_address = address & 0x1F;

        match address {
            0x400..=0x43F => {
                let channel = &mut self.channels[0];
                channel.set_u8(register_address, value);
            }
            0x440..=0x47F => {
                let channel = &mut self.channels[1];
                channel.set_u8(register_address, value);
            }
            0x480..=0x4BF => {
                let channel = &mut self.channels[2];
                channel.set_u8(register_address, value);
            }
            0x4C0..=0x4FF => {
                let channel = &mut self.channels[3];
                channel.set_u8(register_address, value);
            }
            0x500..=0x53F => {
                let channel = &mut self.channels[4];
                channel.set_u8(register_address, value);
            }
            0x540..=0x57F => {
                let channel = &mut self.channels[5];
                channel.set_u8(register_address, value);
            }
            _ => {}
        }
    }

    pub fn step(&mut self, cycles_to_run: usize, audio_sink: &mut dyn Sink<AudioFrame>) {
        for _ in 0..cycles_to_run {
            // Written as seperate iter loops to allow for unrolling
            self.channels
                .iter_mut()
                .for_each(|channel| channel.step_auto_deactivate());

            self.channels
                .iter_mut()
                .for_each(|channel| channel.step_sampling_frequency());

            self.channels
                .iter_mut()
                .for_each(|channel| channel.step_envelope());

            self.channels[4].step_sweep_modulate(&self.modulation);

            // Actually take samples
            if self.sample_output_counter >= SOUND_SAMPLE_RATE_CYCLE_COUNT {
                // Take sample
                self.sample(audio_sink);

                self.sample_output_counter = 0;
            } else {
                self.sample_output_counter += 1;
            }
        }
    }

    fn sample(&mut self, audio_sink: &mut dyn Sink<AudioFrame>) {
        let mut left_acc = 0;
        let mut right_acc = 0;

        for channel in &self.channels {
            if !channel.channel().enable_playback {
                continue;
            }

            let (left, right) = channel.sample(&self.waveforms);

            left_acc += left;
            right_acc += right;
        }

        // Convert to signed audio
        let left_output = ((left_acc & 0xfff8) << 2) as i16;
        let right_output = ((right_acc & 0xfff8) << 2) as i16;

        audio_sink.append((left_output, right_output));
    }
}
