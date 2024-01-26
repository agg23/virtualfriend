use bitvec::array::BitArray;
use bitvec::bitarr;
use bitvec::field::BitField;
use bitvec::prelude::Lsb0;

use crate::constants::GAMEPAD_HARDWARE_READ_CYCLE_COUNT;

pub struct Gamepad {
    /// K-Int-Inh When clear, key input interrupt is enabled.
    ///
    /// TODO: This is not implemented since it does nothing with the normal Gamepad.
    interrupt_enable: bool,

    /// Para/Si When set, reset read operation.
    reset: bool,

    /// Soft-Ck The current clock signal to the Gamepad.
    soft_clk: bool,

    /// SI-State Hardware read is in progress.
    is_hardware_reading: bool,
    hardware_read_counter: usize,
    hardware_read_button_index: usize,

    button_state: u16,
}

#[derive(Clone, Copy)]
pub struct GamepadInputs {
    pub a_button: bool,
    pub b_button: bool,

    pub right_trigger: bool,
    pub left_trigger: bool,

    pub right_dpad_up: bool,
    pub right_dpad_right: bool,
    pub right_dpad_left: bool,
    pub right_dpad_down: bool,

    pub left_dpad_up: bool,
    pub left_dpad_right: bool,
    pub left_dpad_left: bool,
    pub left_dpad_down: bool,

    pub start: bool,
    pub select: bool,
}

impl Gamepad {
    pub fn new() -> Self {
        Gamepad {
            interrupt_enable: false,
            reset: false,
            soft_clk: false,
            is_hardware_reading: false,
            hardware_read_counter: 0,
            hardware_read_button_index: 0,
            button_state: 0,
        }
    }

    pub fn step(&mut self, cycles_to_run: usize, inputs: &GamepadInputs) {
        for _ in 0..cycles_to_run {
            if self.is_hardware_reading {
                if self.hardware_read_counter == GAMEPAD_HARDWARE_READ_CYCLE_COUNT {
                    // Read next button
                    self.hardware_read_counter = 0;

                    let button_value = match self.hardware_read_button_index {
                        0 => inputs.right_dpad_down,
                        1 => inputs.right_dpad_left,
                        2 => inputs.select,
                        3 => inputs.start,
                        4 => inputs.left_dpad_up,
                        5 => inputs.left_dpad_down,
                        6 => inputs.left_dpad_left,
                        7 => inputs.left_dpad_right,
                        8 => inputs.right_dpad_right,
                        9 => inputs.right_dpad_up,
                        10 => inputs.left_trigger,
                        11 => inputs.right_trigger,
                        12 => inputs.b_button,
                        13 => inputs.a_button,
                        _ => false,
                    };
                    let button_value = if button_value { 1 } else { 0 };

                    self.button_state = (self.button_state << 1) | button_value;

                    // Reads at 31.25kHz, taking a total of 512us = 16 read operations of 640 cycles
                    if self.hardware_read_button_index == 15 {
                        self.hardware_read_button_index = 0;
                        self.is_hardware_reading = false;
                    } else {
                        self.hardware_read_button_index += 1;
                    }

                    // TODO: Read actual button
                } else {
                    self.hardware_read_counter += 1;
                }
            }
        }
    }

    /// SDLR/SDHR Serial data register
    ///
    /// Controler data
    pub fn get_serial_data(&self) -> u16 {
        // Low two bits are low battery, and signature (always set), respectively
        // `button_state` shifted by 2 already by `hardware_read_button_index`
        self.button_state | 0x2
    }

    /// SCR Serial control register
    pub fn get_control(&self) -> u16 {
        let mut value = bitarr![u16, Lsb0; 0; 16];

        // TODO: I don't know what this read value should be
        // Mednafen sets it low
        value.set(0, false);
        value.set(1, self.is_hardware_reading);
        value.set(2, true);
        value.set(3, true);
        value.set(4, self.soft_clk);
        value.set(5, self.reset);
        value.set(6, true);
        value.set(7, !self.interrupt_enable);

        value.load()
    }

    /// SCR Serial control register
    pub fn set_control(&mut self, value: u16) {
        let array = BitArray::<_, Lsb0>::new([value]);

        if *array.get(0).unwrap() {
            // Abort hardware read
            self.is_hardware_reading = false;
            self.hardware_read_counter = 0;
            self.hardware_read_button_index = 0;
        }

        if *array.get(2).unwrap() {
            // Start hardware read
            self.is_hardware_reading = true;
        }

        if *array.get(4).unwrap() {
            // Invert soft_clk
            self.soft_clk = !self.soft_clk;

            if self.soft_clk {
                // TODO: Read actual button
            }
        }
        self.reset = *array.get(5).unwrap();

        self.interrupt_enable = !*array.get(7).unwrap();
    }
}
