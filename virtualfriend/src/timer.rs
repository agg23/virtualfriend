use bitvec::array::BitArray;
use bitvec::bitarr;
use bitvec::field::BitField;
use bitvec::prelude::Lsb0;

use crate::constants::TIMER_MIN_INTERVAL_CYCLE_COUNT;

pub struct Timer {
    reload: u16,
    counter: u16,

    enabled: bool,
    did_zero: bool,
    interrupt_enabled: bool,
    /// If true, 20us timer. If false, 100us timer
    timer_interval: bool,

    tick_interval_counter: usize,
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
            tick_interval_counter: 0,
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

        // Reset counter to current reload
        self.counter = self.reload;
        // TODO: Unsure if this is correct
        // Reset timer tick count
        self.tick_interval_counter = 0;
    }

    pub fn get_config(&self) -> u8 {
        // TCR Timer control register
        // Default all bits to set
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

            // TODO: Do we need to do something special to acknowledge the interrupt?
        }

        self.interrupt_enabled = *array.get(3).unwrap();
        self.timer_interval = *array.get(4).unwrap();
    }

    /// Run the timer for 1 cycle.
    ///
    /// Timer does not tick every cycle, so this will run every so often.
    ///
    /// Returns true if an interrupt should fire.
    pub fn step(&mut self, cycles_to_run: usize) -> bool {
        if !self.enabled {
            // Do nothing
            return false;
        }

        let mut request_interrupt = false;

        for _ in 0..cycles_to_run {
            let required_cycle_count = if self.timer_interval {
                TIMER_MIN_INTERVAL_CYCLE_COUNT
            } else {
                TIMER_MIN_INTERVAL_CYCLE_COUNT * 5
            };

            self.tick_interval_counter += 1;

            if self.tick_interval_counter >= required_cycle_count {
                // Fire timer tick
                self.tick_interval_counter = 0;

                if self.tick() {
                    // println!("Timer fire");
                    // This technically allows the interrupt to become desynced with the timer, as it fires, but the timer can keep running
                    request_interrupt = true;
                }
            }
        }

        request_interrupt
    }

    /// Tick the timer.
    ///
    /// Returns true if an interrupt should fire
    fn tick(&mut self) -> bool {
        if self.counter == 0 {
            // Reset counter
            // Separating this from the interrupt (1) case allows setting the timer to 0
            // to not infinitely interrupt
            self.counter = self.reload;

            false
        } else if self.counter == 1 {
            // Value will be 0 after this
            // Fire interrupt and zero
            self.counter -= 1;
            self.did_zero = true;

            true
        } else {
            self.counter -= 1;

            false
        }
    }
}
