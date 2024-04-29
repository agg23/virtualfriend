use crate::constants::{
    ENVELOPE_CYCLE_COUNT, NOISE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT,
    SOUND_LIVE_INTERVAL_CYCLE_COUNT, WAVE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT,
};

use super::{channel::Channel, sweep_modulate::SweepModulate, waveform::Waveform};

pub enum ChannelType {
    PCM {
        channel: Channel,
        index: usize,
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
        frequency_counter: usize,
        shift: u16,
        tap_bit: usize,
        output: bool,
    },
}

impl ChannelType {
    pub fn new_pcm(index: usize) -> Self {
        Self::PCM {
            channel: Channel::new(),
            index,
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
            frequency_counter: 0,
            shift: 0,
            // A tap value of 0 corresponds to bit 14
            tap_bit: 14,
            output: false,
        }
    }

    pub fn channel(&self) -> &Channel {
        match self {
            ChannelType::PCM { channel, .. } => channel,
            ChannelType::PCMCh5 { channel, .. } => channel,
            ChannelType::Noise { channel, .. } => channel,
        }
    }

    pub fn channel_mut(&mut self) -> &mut Channel {
        match self {
            ChannelType::PCM { channel, .. } => channel,
            ChannelType::PCMCh5 { channel, .. } => channel,
            ChannelType::Noise { channel, .. } => channel,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            ChannelType::PCM { index, .. } => *index,
            ChannelType::PCMCh5 { .. } => 4,
            ChannelType::Noise { .. } => 5,
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
                    }
                    Self::Noise { shift, output, .. } => {
                        *shift = 0;
                        *output = false;
                    }
                }
            }
            0x14 => {
                // Envelope Specification register 1
                match self {
                    Self::Noise { tap_bit, .. } => {
                        let tap = (value >> 4) & 0x7;

                        *tap_bit = match tap {
                            0 => 14,
                            1 => 10,
                            2 => 13,
                            3 => 4,
                            4 => 8,
                            5 => 6,
                            6 => 9,
                            7 => 11,
                            _ => unreachable!(),
                        }
                    }
                    _ => (),
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
                    } => *waveform_bank_index = value & 0x7,
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
        // When firing, turn off channel
        if self.channel().auto_deactivate {
            self.channel_mut().live_interval_tick_counter += 1;

            if self.channel().live_interval_tick_counter >= SOUND_LIVE_INTERVAL_CYCLE_COUNT {
                // One tick of live interval
                self.channel_mut().live_interval_counter += 1;

                if self.channel().live_interval_counter >= (self.channel().live_interval + 1) {
                    // Stop the channel
                    println!("Stopping channel {}", self.index());

                    self.channel_mut().enable_playback = false;
                    self.channel_mut().live_interval_counter = 0;
                }

                self.channel_mut().live_interval_tick_counter = 0;
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

        // Sampling frequency
        self.channel_mut().sampling_frequency_tick_counter += 1;

        if self.channel().sampling_frequency_tick_counter >= cycles_per_frequency_tick {
            // One tick of frequency step increment
            self.channel_mut().sampling_frequency_counter += 1;

            if self.channel().sampling_frequency_counter > 2047 - self.frequency() {
                // Move to next sample
                self.increment_sample();
                // println!("Increment sample {}", self.channel().enable_playback);

                self.channel_mut().sampling_frequency_counter = 0;
            }

            self.channel_mut().sampling_frequency_tick_counter = 0;
        }
    }

    ///
    /// Modify envelope after period
    ///
    pub fn step_envelope(&mut self) {
        let channel = self.channel_mut();

        channel.envelope_tick_counter += 1;
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

                    panic!("Envelope tick {}", channel.envelope_step_counter);

                    channel.envelope_step_counter = 0;
                }
            }

            channel.envelope_tick_counter = 0;
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
    /// Moves to the next sample in the waveform, or chooses the next noise value
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
            Self::Noise {
                shift,
                tap_bit,
                output,
                ..
            } => {
                let source_bit = *shift >> 7;
                let tap_bit = *shift >> *tap_bit;

                let output_bit = source_bit ^ tap_bit;
                // Bit is inverted
                let output_bit = !output_bit & 0x1;

                *shift = (*shift << 1) & 0x7FFF;
                *shift = *shift | output_bit;

                *output = output_bit == 1;
            }
        }
    }

    ///
    /// Gets the latest output for this channel
    ///
    fn output(&self, waveforms: &[Waveform; 5]) -> u8 {
        match self {
            ChannelType::PCM {
                waveform_bank_index,
                current_sample_index,
                ..
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
                    let output =
                        waveforms[*waveform_bank_index as usize].get_indexed(*current_sample_index);

                    // println!(
                    //     "Output {output:02X}, bank: {waveform_bank_index:01X} {current_sample_index:02X}"
                    // );

                    output
                }
            }
            Self::Noise { output, .. } => {
                if *output {
                    // Output, if high, is 63
                    63
                } else {
                    0
                }
            }
        }
    }

    fn frequency(&self) -> u16 {
        match self {
            ChannelType::PCM { channel, .. } => channel.sampling_frequency,
            ChannelType::PCMCh5 { sweep_mod, .. } => sweep_mod.frequency(),
            ChannelType::Noise { channel, .. } => channel.sampling_frequency,
        }
    }
}
