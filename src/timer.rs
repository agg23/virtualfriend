use bitvec::array::BitArray;
use bitvec::bitarr;
use bitvec::field::BitField;
use bitvec::prelude::Lsb0;

pub struct Timer {
    reload: u16,
    counter: u16,

    enabled: bool,
    did_zero: bool,
    interrupt_enabled: bool,
    timer_interval: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            reload: 0,
            counter: 0xFFFF,
            enabled: false,
            did_zero: false,
            interrupt_enabled: false,
            timer_interval: false,
        }
    }

    pub fn get_counter(&self) -> u16 {
        self.counter
    }

    pub fn set_reload(&mut self, reload_half: u8, upper: bool) {
        if upper {
            self.reload = ((reload_half as u16) << 8) | (self.reload & 0xFF);
        } else {
            self.reload = (self.reload & 0xFF00) | (reload_half as u16);
        }
    }

    pub fn get_config(&self) -> u8 {
        let mut value = bitarr![u8, Lsb0; 0xFF; 8];

        value.set(0, self.enabled);
        value.set(1, self.did_zero);
        value.set(3, self.interrupt_enabled);
        value.set(4, self.timer_interval);

        value.load()
    }

    pub fn set_config(&mut self, value: u8) {
        let array = BitArray::<_, Lsb0>::new([value]);

        self.enabled = *array.get(0).unwrap();

        if *array.get(2).unwrap() {
            // Write to Z-Stat-Clr
            self.did_zero = false;
        }

        self.interrupt_enabled = *array.get(3).unwrap();
        self.timer_interval = *array.get(4).unwrap();
    }

    pub fn tick(&mut self) {
        if self.counter == 0 {
            self.counter = self.reload;
            self.did_zero = true;

            // TODO: Fire interrupt
        } else {
            self.counter -= 1;
        }
    }
}
