use crate::constants::{
    NOISE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT, SOUND_LIVE_INTERVAL_CYCLE_COUNT,
    WAVE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT,
};

use super::{channel::Channel, waveform::Waveform};

pub enum ChannelType {
    PCM {
        waveform_bank_index: u8,
        current_sample_index: usize,
    },
    /// Same as PCM, plus supports frequency sweep and modulation
    PCMCh5 {
        waveform_bank_index: u8,
        current_sample_index: usize,
    },
    Noise {},
}

impl ChannelType {
    pub fn new_pcm() -> Self {
        Self::PCM {
            waveform_bank_index: 0,
            current_sample_index: 0,
        }
    }

    pub fn new_pcm_ch5() -> Self {
        Self::PCMCh5 {
            waveform_bank_index: 0,
            current_sample_index: 0,
        }
    }

    pub fn new_noise() -> Self {
        Self::Noise {}
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
                    }
                    Self::Noise {} => {
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
    }

    pub fn step(&mut self, cycles_to_run: usize, channel: &mut Channel, waveforms: &[Waveform; 5]) {
        let cycles_per_frequency_tick = if let Self::Noise {} = self {
            NOISE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT
        } else {
            WAVE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT
        };

        let mut needs_next_sample = false;

        // Common channel properties
        for _ in 0..cycles_to_run {
            if !channel.enable_playback {
                continue;
            }

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

            if channel.sampling_frequency_tick_counter >= cycles_per_frequency_tick {
                // One tick of frequency step increment
                if channel.sampling_frequency_counter >= 2048 - channel.sampling_frequency {
                    // Move to next sample
                    needs_next_sample = true;

                    channel.sampling_frequency_counter = 0;
                } else {
                    channel.sampling_frequency_counter += 1;
                }

                channel.sampling_frequency_tick_counter = 0;
            } else {
                channel.sampling_frequency_tick_counter += 1;
            }
        }

        // The two separate for loops running serially relies on the fact that operations from one will not
        // modify the other
        match self {
            ChannelType::PCM {
                waveform_bank_index,
                current_sample_index,
            }
            | ChannelType::PCMCh5 {
                waveform_bank_index,
                current_sample_index,
            } => {
                // for _ in 0..cycles_to_run {
                //     if !channel.enable_playback {
                //         continue;
                //     }
                // }
                if needs_next_sample {
                    // Update sample index
                    *current_sample_index = (*current_sample_index + 1) & 0x1F;

                    if *waveform_bank_index > 4 {
                        // Out of range. Nothing plays
                        channel.sampled_value = 0;
                    } else {
                        channel.sampled_value = waveforms[*waveform_bank_index as usize]
                            .get_indexed(*current_sample_index)
                    }
                }
            }
            ChannelType::Noise {} => {
                // TODO
            }
        }
    }
}
