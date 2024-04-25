use std::f32::consts::E;

use tartan_bitfield::bitfield;

use crate::constants::{SWEEP_FAST_CYCLE_COUNT, SWEEP_SLOW_CYCLE_COUNT};

use super::channel::Channel;

pub struct SweepModulate {
    enable: bool,

    loop_modulation: bool,
    ///
    /// FUNC
    ///
    /// If true, operation will modulate. Otherwise, will sweep
    ///
    should_modulate: bool,

    ///
    /// CLK
    ///
    /// If true, base clock is 130.2Hz. Otherwise, 1041.6Hz
    should_use_fast_clock: bool,

    ///
    /// INTERVAL
    ///
    /// How long each frequency lasts before being modified
    ///
    modification_interval: u8,

    ///
    /// DIR
    ///
    /// If true, the sweep will add. Otherwise, it subtracts
    sweep_direction: bool,

    ///
    /// SHIFT
    ///
    /// The number of bits to shift the current frequency by
    ///
    sweep_shift: u8,

    /// Counting up to one `should_use_fast_clock` interval
    step_counter: usize,
    /// The number of times the period has been counted
    interval_counter: u8,

    /// The index of the current modulation data
    modulation_index: usize,

    /// Track the last register set value of frequency for modulation calculation
    last_written_frequency: u16,
    current_frequency: u16,
    next_frequency: u16,
}

bitfield! {
    struct SweepEnableRegister(u8) {
        [4] should_modulate,
        [5] loop_modulation,
        [6] enable,
    }
}

bitfield! {
    struct SweepRegister(u8) {
        [0..=2] sweep_shift: u8,
        [3] sweep_direction,
        [4..=6] modification_interval: u8,
        [7] should_use_fast_clock
    }
}

impl SweepModulate {
    pub fn new() -> Self {
        Self {
            enable: false,
            loop_modulation: false,
            should_modulate: false,
            should_use_fast_clock: false,
            modification_interval: 0,
            sweep_direction: false,
            sweep_shift: 0,
            step_counter: 0,
            interval_counter: 0,
            modulation_index: 0,
            last_written_frequency: 0,
            current_frequency: 0,
            next_frequency: 0,
        }
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        match address {
            0x0 => {
                // Sound interval specification register
                // Reset sweep/mod counter
                self.step_counter = 0;

                // Reset modulation index
                self.modulation_index = 0;
            }
            0x8 => {
                // Frequency low register
                self.last_written_frequency &= 0xFF00;
                self.last_written_frequency |= value as u16;

                self.next_frequency &= 0xFF00;
                self.next_frequency |= value as u16;
            }
            0xC => {
                // Frequency high register
                self.last_written_frequency &= 0xFF;
                self.last_written_frequency |= (value as u16 & 0x7) << 8;

                self.next_frequency &= 0xFF;
                self.next_frequency |= (value as u16 & 0x7) << 8;
            }
            0x14 => {
                // Envelope specification register
                // Sweep/mod enable

                let register = SweepEnableRegister(value);

                self.should_modulate = register.should_modulate();
                self.loop_modulation = register.loop_modulation();
                self.enable = register.enable();
            }
            0x1C => {
                // Sweep/mod register
                let register = SweepRegister(value);

                self.sweep_shift = register.sweep_shift();
                self.sweep_direction = register.sweep_direction();
                self.modification_interval = register.modification_interval();
                self.should_use_fast_clock = register.should_use_fast_clock();
            }
            _ => {}
        }
    }

    pub fn step(&mut self, channel: &mut Channel, modulation_data: &[i8]) {
        let period = if self.should_use_fast_clock {
            SWEEP_FAST_CYCLE_COUNT
        } else {
            SWEEP_SLOW_CYCLE_COUNT
        };

        self.step_counter += 1;

        if self.step_counter >= period {
            self.interval_counter += 1;

            if self.interval_counter >= self.modification_interval {
                self.increment_sweep_mod(channel, modulation_data);

                self.interval_counter = 0;
            }

            self.step_counter = 0;
        }
    }

    pub fn frequency(&self) -> u16 {
        self.current_frequency
    }

    fn increment_sweep_mod(&mut self, channel: &mut Channel, modulation_data: &[i8]) {
        // Update to latest frequency
        self.current_frequency = self.next_frequency;

        if self.current_frequency > 2047 {
            // Immediately stop this channel
            // A hardware bug causes this to occur even if the sweep/mod is disabled
            channel.enable_playback = false;

            return;
        }

        // Halt if interval is set to 0
        if !channel.enable_playback || !self.enable || self.modification_interval == 0 {
            return;
        }

        self.next_frequency = if self.should_modulate {
            // Modulate
            let mod_data = modulation_data[self.modulation_index];

            // Use last register frequency to add to modulation data
            let new_frequency = self.last_written_frequency.wrapping_add(mod_data as u16) & 0x7FF;

            if self.modulation_index < 32 || self.loop_modulation {
                // If loop modulation and 31, add 1 and wrap to 0
                self.modulation_index = (self.modulation_index + 1) & 0x1F;
            }

            new_frequency
        } else {
            // Sweep
            let shift = self.current_frequency >> (self.sweep_shift as usize);
            if self.sweep_direction {
                self.current_frequency.wrapping_add(shift) & 0x7FF
            } else {
                self.current_frequency.wrapping_sub(shift) & 0x7FF
            }
        }
    }
}
