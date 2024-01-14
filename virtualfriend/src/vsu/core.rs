use crate::constants::SOUND_SAMPLE_RATE_CYCLE_COUNT;

use super::{
    channel::Channel,
    channel_enum::ChannelType,
    traits::{AudioFrame, Sink},
    waveform::Waveform,
};

pub struct VSU {
    waveforms: [Waveform; 5],
    channels: [(Channel, ChannelType); 6],

    sample_output_counter: usize,
}

impl VSU {
    pub fn new() -> Self {
        VSU {
            waveforms: [Waveform::new(); 5],
            channels: [
                (Channel::new(), ChannelType::new_pcm()),
                (Channel::new(), ChannelType::new_pcm()),
                (Channel::new(), ChannelType::new_pcm()),
                (Channel::new(), ChannelType::new_pcm()),
                (Channel::new(), ChannelType::new_pcm_ch5()),
                (Channel::new(), ChannelType::new_noise()),
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
            0x580 => {
                // SSTOP Stop all sound register
                if value & 0x1 != 0 {
                    for (channel, _) in &mut self.channels {
                        channel.enable_playback = false;
                    }
                }
            }
            _ => self.send_channel_write(address, value),
        }
    }

    fn playback_occuring(&self) -> bool {
        self.channels
            .iter()
            .any(|(channel, _)| channel.enable_playback)
    }

    fn send_channel_write(&mut self, address: usize, value: u8) {
        let register_address = address & 0x1F;

        match address {
            0x400..=0x43F => {
                let (channel, channel_type) = &mut self.channels[0];
                channel.set_u8(register_address, value);
                channel_type.set_u8(register_address, value)
            }
            0x440..=0x47F => {
                let (channel, channel_type) = &mut self.channels[1];
                channel.set_u8(register_address, value);
                channel_type.set_u8(register_address, value)
            }
            0x480..=0x4BF => {
                let (channel, channel_type) = &mut self.channels[2];
                channel.set_u8(register_address, value);
                channel_type.set_u8(register_address, value)
            }
            0x4C0..=0x4FF => {
                let (channel, channel_type) = &mut self.channels[3];
                channel.set_u8(register_address, value);
                channel_type.set_u8(register_address, value)
            }
            0x500..=0x53F => {
                let (channel, channel_type) = &mut self.channels[4];
                channel.set_u8(register_address, value);
                channel_type.set_u8(register_address, value)
            }
            0x540..=0x57F => {
                let (channel, channel_type) = &mut self.channels[5];
                channel.set_u8(register_address, value);
                channel_type.set_u8(register_address, value)
            }
            _ => {}
        }
    }

    pub fn step(&mut self, cycles_to_run: usize, audio_sink: &mut dyn Sink<AudioFrame>) {
        for _ in 0..cycles_to_run {
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

        for (channel, _) in &self.channels {
            let (left, right) = channel.sample();

            left_acc += left;
            right_acc += right;
        }

        // Convert to signed audio
        let left_output = ((left_acc & 0xfff8) << 2) as i16;
        let right_output = ((right_acc & 0xfff8) << 2) as i16;

        audio_sink.append((left_output, right_output));
    }
}
