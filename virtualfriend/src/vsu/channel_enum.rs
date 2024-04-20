use crate::constants::{
    ENVELOPE_CYCLE_COUNT, NOISE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT,
    SOUND_LIVE_INTERVAL_CYCLE_COUNT, WAVE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT,
};

use super::{channel::Channel, sweep_modulate::SweepModulate, waveform::Waveform};

pub enum ChannelType {
    PCM {
        channel: Channel,
        waveform_bank_index: u8,
        current_sample_index: usize,
    },
    /// Same as PCM, plus supports frequency sweep and modulation
    PCMCh5 {
        channel: Channel,
        sweep_mod: SweepModulate,
        waveform_bank_index: u8,
        current_sample_index: usize,
    },
    Noise {
        channel: Channel,
    },
}

impl ChannelType {
    pub fn new_pcm() -> Self {
        Self::PCM {
            channel: Channel::new(),
            waveform_bank_index: 0,
            current_sample_index: 0,
        }
    }

    pub fn new_pcm_ch5() -> Self {
        Self::PCMCh5 {
            channel: Channel::new(),
            sweep_mod: SweepModulate::new(),
            waveform_bank_index: 0,
            current_sample_index: 0,
        }
    }

    pub fn new_noise() -> Self {
        Self::Noise {
            channel: Channel::new(),
        }
    }

    pub fn channel(&self) -> &Channel {
        match self {
            ChannelType::PCM { channel, .. } => channel,
            ChannelType::PCMCh5 { channel, .. } => channel,
            ChannelType::Noise { channel } => channel,
        }
    }

    pub fn channel_mut(&mut self) -> &mut Channel {
        match self {
            ChannelType::PCM { channel, .. } => channel,
            ChannelType::PCMCh5 { channel, .. } => channel,
            ChannelType::Noise { channel } => channel,
        }
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        // Address has already been ANDed with 0x1F
        match address {
            0x0 => {
                // Sound interval specification register
                // On write, reset sample position
                match self {
                    Self::PCM {
                        ref mut current_sample_index,
                        ..
                    }
                    | Self::PCMCh5 {
                        ref mut current_sample_index,
                        ..
                    } => {
                        *current_sample_index = 0;

                        // TODO: Reset frequency modification timer for Ch5
                    }
                    Self::Noise { .. } => {
                        // TODO: Reset shift register
                    }
                }
            }
            0x18 => {
                // Base address setting register
                match self {
                    Self::PCM {
                        ref mut waveform_bank_index,
                        ..
                    }
                    | Self::PCMCh5 {
                        ref mut waveform_bank_index,
                        ..
                    } => *waveform_bank_index = value & 0x5,
                    _ => {}
                }
            }
            _ => {}
        }

        self.channel_mut().set_u8(address, value);

        match self {
            Self::PCMCh5 { sweep_mod, .. } => {
                sweep_mod.set_u8(address, value);
            }
            _ => {}
        }
    }

    ///
    /// Sample stereo output from channel
    ///
    pub fn sample(&self, waveforms: &[Waveform; 5]) -> (u16, u16) {
        self.channel().sample(self.output(waveforms))
    }

    ///
    /// Turn off channel after period
    ///
    pub fn step_auto_deactivate(&mut self) {
        let channel = self.channel_mut();

        // When firing, turn off channel
        if channel.auto_deactivate {
            if channel.live_interval_tick_counter >= SOUND_LIVE_INTERVAL_CYCLE_COUNT {
                // One tick of live interval
                if channel.live_interval_counter >= (channel.live_interval + 1) {
                    // Stop the channel
                    channel.enable_playback = false;
                    channel.live_interval_counter = 0;
                } else {
                    channel.live_interval_counter += 1;
                }

                channel.live_interval_tick_counter = 0;
            } else {
                channel.live_interval_tick_counter += 1;
            }
        }
    }

    ///
    /// Increment current sample for channel after period
    ///
    pub fn step_sampling_frequency(&mut self) {
        let cycles_per_frequency_tick = if let ChannelType::Noise { .. } = self {
            NOISE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT
        } else {
            WAVE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT
        };

        let channel = self.channel();

        // Sampling frequency
        if channel.sampling_frequency_tick_counter >= cycles_per_frequency_tick {
            // One tick of frequency step increment
            if channel.sampling_frequency_counter > 2047 - self.frequency() {
                // Move to next sample
                self.increment_sample();

                self.channel_mut().sampling_frequency_counter = 0;
            } else {
                self.channel_mut().sampling_frequency_counter += 1;
            }

            self.channel_mut().sampling_frequency_tick_counter = 0;
        } else {
            self.channel_mut().sampling_frequency_tick_counter += 1;
        }
    }

    ///
    /// Modify envelope after period
    ///
    pub fn step_envelope(&mut self) {
        let channel = self.channel_mut();

        if channel.envelope_tick_counter >= ENVELOPE_CYCLE_COUNT {
            // One tick of envelope
            if channel.enable_envelope_modification {
                if channel.envelope_step_counter >= channel.envelope_interval + 1 {
                    if channel.envelope_direction && channel.envelope_level < 15 {
                        // Increment volume
                        channel.envelope_level += 1;
                    } else if !channel.envelope_direction && channel.envelope_level > 0 {
                        // Decrement volume
                        channel.envelope_level -= 1;
                    } else if channel.loop_envelope {
                        // We must be at 0 or 15. We're set to repeat
                        channel.envelope_level = channel.envelope_reload_value;
                    }

                    channel.envelope_step_counter = 0;
                }
            }

            channel.envelope_tick_counter = 0;
        } else {
            channel.envelope_tick_counter += 1;
        }
    }

    ///
    /// Update sweep/modulation after period
    ///
    pub fn step_sweep_modulate(&mut self, modulation_data: &[i8]) {
        match self {
            ChannelType::PCMCh5 {
                channel, sweep_mod, ..
            } => {
                sweep_mod.step(channel, modulation_data);
            }
            _ => (),
        }
    }

    ///
    /// Moves to the next sample in the waveform. Does nothing on Noise channel
    ///
    fn increment_sample(&mut self) {
        match self {
            ChannelType::PCM {
                current_sample_index,
                ..
            }
            | ChannelType::PCMCh5 {
                current_sample_index,
                ..
            } => {
                // Update sample index
                *current_sample_index = (*current_sample_index + 1) & 0x1F;
            }
            Self::Noise { .. } => (),
        }
    }

    ///
    /// Gets the latest output for this channel
    ///
    fn output(&self, waveforms: &[Waveform; 5]) -> u8 {
        match self {
            ChannelType::PCM {
                channel: _,
                waveform_bank_index,
                current_sample_index,
            }
            | ChannelType::PCMCh5 {
                waveform_bank_index,
                current_sample_index,
                ..
            } => {
                // Update sample index
                if *waveform_bank_index > 4 {
                    // Out of range. Nothing plays
                    0
                } else {
                    waveforms[*waveform_bank_index as usize].get_indexed(*current_sample_index)
                }
            }
            Self::Noise { .. } => {
                // TODO: Add noise sample
                0
            }
        }
    }

    fn frequency(&self) -> u16 {
        match self {
            ChannelType::PCM { channel, .. } => channel.sampling_frequency,
            ChannelType::PCMCh5 { sweep_mod, .. } => sweep_mod.frequency(),
            ChannelType::Noise { .. } => {
                // TODO: Handle noise
                return 0;
            }
        }
    }
}
