use tartan_bitfield::bitfield;

#[derive(Savefile)]
pub struct Channel {
    /// Sound interval specification
    pub live_interval: u8,
    pub live_interval_counter: u8,
    pub live_interval_tick_counter: usize,
    pub auto_deactivate: bool,
    pub enable_playback: bool,

    pub left_volume: u8,
    pub right_volume: u8,
    pub envelope_level: u8,

    /// Frequency specification
    pub sampling_frequency: u16,
    pub sampling_frequency_counter: u16,
    pub sampling_frequency_tick_counter: usize,

    /// Envelope specification
    pub envelope_interval: u8,
    /// When set, envelope will grow (add 1). Otherwise, shrink
    pub envelope_direction: bool,
    pub envelope_reload_value: u8,
    pub enable_envelope_modification: bool,
    pub loop_envelope: bool,
    // The global envelope timing position
    pub envelope_tick_counter: usize,
    /// The envelope's position in the envelope interval
    pub envelope_step_counter: u8,
}

bitfield! {
    struct SoundIntervalSpecRegister(u8) {
        [0..=4] live_interval: u8,
        [5] auto_deactivate,
        [7] enable_playback,
    }
}

bitfield! {
    struct EnvelopSpecificationRegister0(u8) {
        [0..=2] envelope_interval: u8,
        [3] envelope_direction,
        [4..=7] envelope_reload_value: u8
    }
}

impl Channel {
    pub fn new() -> Self {
        Channel {
            live_interval: 0,
            live_interval_counter: 0,
            live_interval_tick_counter: 0,
            auto_deactivate: false,
            enable_playback: false,

            left_volume: 0,
            right_volume: 0,
            envelope_level: 0,

            sampling_frequency: 0,
            sampling_frequency_counter: 0,
            sampling_frequency_tick_counter: 0,

            envelope_interval: 0,
            envelope_direction: false,
            envelope_reload_value: 0,
            enable_envelope_modification: false,
            loop_envelope: false,
            envelope_tick_counter: 0,
            envelope_step_counter: 0,
        }
    }

    pub fn set_u8(&mut self, address: usize, value: u8) {
        match address {
            0x0 => {
                // Sound interval specification register
                let register = SoundIntervalSpecRegister(value);

                self.live_interval = register.live_interval();
                self.auto_deactivate = register.auto_deactivate();
                self.enable_playback = register.enable_playback();

                // Reset frequency delay counter to beginning of current sample
                self.sampling_frequency_counter = 0;

                // PCM memory reset performed in `channel_enum.rs`

                // Reset envelope step counter
                self.envelope_step_counter = 0;
            }
            0x4 => {
                // Stereo level setting register
                self.left_volume = value >> 4;
                self.right_volume = value & 0xF;
            }
            0x8 => {
                // Frequency low register
                self.sampling_frequency &= 0xFF00;
                self.sampling_frequency |= value as u16;
            }
            0xC => {
                // Frequency high register
                self.sampling_frequency &= 0xFF;
                self.sampling_frequency |= (value as u16 & 0x7) << 8;
            }
            0x10 => {
                // Envelope Specification register 0
                let register = EnvelopSpecificationRegister0(value);

                self.envelope_interval = register.envelope_interval();
                self.envelope_direction = register.envelope_direction();
                self.envelope_reload_value = register.envelope_reload_value();

                self.envelope_level = self.envelope_reload_value;
            }
            0x14 => {
                // Envelope Specification register 1
                self.enable_envelope_modification = value & 0x1 != 0;
                self.loop_envelope = value & 0x2 != 0;

                // TODO: Send ext to noise
            }
            _ => {}
        }
    }

    pub fn sample(&self, output_value: u8) -> (u16, u16) {
        (
            self.sample_side(self.left_volume, output_value),
            self.sample_side(self.right_volume, output_value),
        )
    }

    fn sample_side(&self, volume: u8, output_value: u8) -> u16 {
        let amplitude = self.envelope_level * volume;
        // Take only the top 5 bits
        let mut amplitude = amplitude >> 3;

        if self.envelope_level > 0 && volume > 0 {
            amplitude += 1;
        }

        // 6 * 5 = 11 bits
        let output = (amplitude as u16) * (output_value as u16);
        // Only top 10 bits are used
        output >> 1
    }
}
