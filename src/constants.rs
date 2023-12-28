pub const CLOCK_SPEED: u32 = 20_000_000;

const CYCLES_PER_MS: u32 = CLOCK_SPEED / 1000;
const CYCLES_PER_US: u32 = CLOCK_SPEED / 1000000;

pub const LEFT_FRAME_BUFFER_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 3;
pub const LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 8;
pub const FCLK_LOW_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 10;
pub const RIGHT_FRAME_BUFFER_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 13;
pub const RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 18;
pub const FRAME_COMPLETE_CYCLE_OFFSET: u32 = CYCLES_PER_MS * 20;

pub const DISPLAY_WIDTH: usize = 384;
pub const DISPLAY_HEIGHT: usize = 224;
pub const DISPLAY_PIXEL_LENGTH: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

pub const FRAMEBUFFER_HEIGHT: usize = 256;

// Rustual-boy assumes 5ms per draw frame (one side)
// Docs indicate affine worlds go over the time allotment, and an empty frame takes 2.8ms
// We follow Rustual-boy's lead for now
pub const FRAME_RENDER_TIME_MS: u32 = 5;

// Drawing occurs in vertical chunks. There are 8 rows of pixels processed at once
// We break it up like this, assuming that the entire 8 rows of pixels are drawn in one go.
// It is broken up to keep track of `SBCOUNT`
pub const DRAWING_BLOCK_COUNT: u32 = (DISPLAY_HEIGHT as u32) / 8;

/// Total time (in cycles) taken to draw one block
pub const DRAWING_BLOCK_CYCLE_COUNT: u32 =
    CYCLES_PER_MS * FRAME_RENDER_TIME_MS / DRAWING_BLOCK_COUNT;

/// Total time SBOUT remains high
pub const SBOUT_HIGH_CYCLE_COUNT: u32 = CYCLES_PER_US * 56;
