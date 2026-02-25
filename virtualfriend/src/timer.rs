use tartan_bitfield::bitfield;

use crate::constants::TIMER_MIN_INTERVAL_CYCLE_COUNT;

#[derive(Savefile)]
pub struct Timer {
    reload: u16,
    counter: u16,

    enabled: bool,
    did_zero: bool,
    interrupt_enabled: bool,
    /// If true, 20us timer. If false, 100us timer
    timer_interval: bool,

    tick_interval_counter: usize,

    /// The current tick state of the timer, 0-4. Setting the value to 0 is a 100us tick
    tick_20us_counter: usize,

    /// Reload value was set to zero by software, interrupt is queued to next tick
    deferred_interrupt: bool,

    /// Tracks whether an interrupt is pending (level-triggered behavior)
    #[savefile_versions = "1.."]
    interrupt_pending: bool,
}

bitfield! {
    #[derive(Savefile)]
    struct TCR(u8) {
        /// Timer is enabled.
        [0] enabled,
        /// [Readonly] Set when counter reaches zero.
        [1] did_zero,
        /// [Writeonly] Clears `did_zero`.
        [2] did_zero_clear,
        /// Interrupt enabled.
        [3] interrupt_enabled,
        /// Timer interval. High is 20us, low is 100us.
        [4] timer_interval,
    }
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
            tick_20us_counter: 0,
            deferred_interrupt: false,
            interrupt_pending: false,
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

        if self.enabled && self.reload == 0 {
            self.deferred_interrupt = true;
        }

        // Reset counter to current reload
        self.counter = self.reload;
        // Reset timer tick count
        // "When either register is written, the entire 16-bit value will be loaded into the counter and reset the current timer tick to the beginning of its wait interval."
        self.tick_interval_counter = 0;
    }

    pub fn get_config(&self) -> u8 {
        // TCR Timer control register
        // Default all bits to set
        let mut value = TCR(0xFF);

        value.set_enabled(self.enabled);
        value.set_did_zero(self.did_zero);
        value.set_interrupt_enabled(self.interrupt_enabled);
        value.set_timer_interval(self.timer_interval);

        value.0
    }

    pub fn set_config(&mut self, value: u8) {
        let value = TCR(value);

        let prev_enabled = self.enabled;
        self.enabled = value.enabled();

        if value.did_zero_clear() {
            // Z-Stat-Clr only clears did_zero when:
            // - Timer was already disabled, OR
            // - Timer is staying enabled AND counter is non-zero
            // If we're disabling the timer in this set, do not clear
            if !prev_enabled || (self.enabled && self.counter != 0) {
                self.did_zero = false;
            }
        }

        // Clear interrupt pending if did_zero was cleared or interrupt is being disabled
        if !self.did_zero || !value.interrupt_enabled() {
            self.interrupt_pending = false;
        }

        self.interrupt_enabled = value.interrupt_enabled();

        let new_timer_interval = value.timer_interval();

        if !self.timer_interval && new_timer_interval {
            // When switching from 100us to 20us, if internal 20us counter isn't at 0, decrement timer
            if self.tick_20us_counter != 0 {
                self.tick(true);
            }
        }

        self.timer_interval = new_timer_interval;
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

        let was_deferred_interrupt = self.deferred_interrupt;

        if self.deferred_interrupt {
            self.deferred_interrupt = false;
        }

        for _ in 0..cycles_to_run {
            // let required_cycle_count = if self.timer_interval {
            //     TIMER_MIN_INTERVAL_CYCLE_COUNT
            // } else {
            //     TIMER_MIN_INTERVAL_CYCLE_COUNT * 5
            // };

            self.tick_interval_counter += 1;

            if self.tick_interval_counter >= TIMER_MIN_INTERVAL_CYCLE_COUNT {
                // Fire 20us timer tick
                self.tick_interval_counter = 0;

                self.tick_20us_counter = (self.tick_20us_counter + 1) % 5;

                let timer_tick = self.timer_interval || self.tick_20us_counter == 0;

                if timer_tick && self.tick(false) {
                    // println!("Timer fire");
                    // This technically allows the interrupt to become desynced with the timer, as it fires, but the timer can keep running
                    if self.interrupt_enabled {
                        self.interrupt_pending = true;
                    }
                }
            }
        }

        // Set interrupt_pending if a deferred interrupt was generated
        if was_deferred_interrupt {
            self.interrupt_pending = true;
        }

        // Return the persistent interrupt state (level-triggered behavior)
        self.interrupt_pending
    }

    /// Tick the timer.
    ///
    /// Returns true if an interrupt should fire
    fn tick(&mut self, defer_interrupt: bool) -> bool {
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

            if defer_interrupt {
                self.deferred_interrupt = true;
            }

            true
        } else {
            self.counter -= 1;

            false
        }
    }
}
