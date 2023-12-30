use bitvec::array::BitArray;
use bitvec::prelude::Lsb0;
use bitvec::{bitarr, field::BitField};

use crate::gamepad::Gamepad;
use crate::timer::Timer;

pub struct Hardware {
    pub gamepad: Gamepad,

    // TODO: Remove pub
    pub timer: Timer,

    comm_interrupt_enable: bool,
    comm_external_clock: bool,
    comm_inprogress: bool,
}

impl Hardware {
    pub fn new() -> Self {
        Hardware {
            gamepad: Gamepad::new(),
            timer: Timer::new(),
            comm_interrupt_enable: false,
            comm_external_clock: false,
            comm_inprogress: false,
        }
    }

    pub fn get(&self, address: u8) -> u16 {
        let address = address & 0x3F;

        match address {
            0x0..=0x3 => {
                // CCR Communication control register
                let mut value = bitarr![u32, Lsb0; 0xFF; 32];

                value.set(1, self.comm_inprogress);
                value.set(4, self.comm_external_clock);
                value.set(7, self.comm_interrupt_enable);

                value.load()
            }
            0x4..=0x7 => {
                // CCSR COMCNT control register
                // TODO: Implement
                // let mut value = bitarr![u32, Lsb0; 0xFF; 32];

                // value.set(0, self.comcnt_signal_status);
                // value.set(1, self.comcnt_signal_status_write);
                0xFF
            }
            0x8..=0xB => {
                // CDTR Transmitted data register
                // TODO: Implement
                0
            }
            0xC..=0xF => {
                // CDRR Received data register
                // TODO: Implement
                0
            }
            0x10..=0x13 => {
                // Serial data low register
                self.gamepad.get_serial_data() & 0xFF
            }
            0x14..=0x17 => {
                // Serial data high register
                self.gamepad.get_serial_data() >> 8
            }
            0x18..=0x1B => {
                // TLR Timer counter low register
                (self.timer.get_counter() & 0xFF) as u16
            }
            0x1C..=0x1F => {
                // THR Timer counter high register
                (self.timer.get_counter() >> 8) as u16
            }
            0x20..=0x23 => {
                // TCR Timer control register
                self.timer.get_config() as u16
            }
            0x24..=0x27 => {
                // WCR Wait control register
                // TODO: Implement
                0
            }
            0x28..=0x2B => {
                // SCR Serial control register
                self.gamepad.get_control()
            }
            _ => unreachable!(),
        }
    }

    pub fn set(&mut self, address: u8, value: u16) {
        let address = address & 0x3F;

        match address {
            0x0..=0x3 => {
                // CCR Communication control register
                let array = BitArray::<_, Lsb0>::new([value]);

                self.comm_inprogress = *array.get(1).unwrap();
                self.comm_external_clock = *array.get(4).unwrap();
                self.comm_interrupt_enable = *array.get(7).unwrap();
            }
            0x4..=0x7 => {
                // CCSR COMCNT control register
                // TODO: Implement
            }
            0x8..=0xB => {
                // CDTR Transmitted data register
                // TODO: Implement
            }
            0xC..=0xF => {
                // CDRR Received data register
                // TODO: Implement
            }
            0x10..=0x13 => {
                // Serial data low register
            }
            0x14..=0x17 => {
                // Serial data high register
            }
            0x18..=0x1B => {
                // TLR Timer counter low register
                self.timer.set_reload(value as u8, false);
            }
            0x1C..=0x1F => {
                // THR Timer counter high register
                self.timer.set_reload(value as u8, true);
            }
            0x20..=0x23 => {
                // TCR Timer control register
                self.timer.set_config(value as u8);
            }
            0x24..=0x27 => {
                // WCR Wait control register
                // TODO: Implement
            }
            0x28..=0x2B => {
                // SCR Serial control register
                self.gamepad.set_control(value);
            }

            _ => unreachable!(),
        }
    }
}
