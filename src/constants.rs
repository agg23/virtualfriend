pub const CLOCK_SPEED: u32 = 20_000_000;

pub const CYCLES_PER_MS: u32 = CLOCK_SPEED / 1000;

pub const LEFT_FRAME_BUFFER_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 3;
pub const LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 8;
pub const FCLK_LOW_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 10;
pub const RIGHT_FRAME_BUFFER_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 13;
pub const RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 18;
pub const FRAME_COMPLETE_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 20;
