pub const CLOCK_SPEED: usize = 20_000_000;

const CYCLES_PER_MS: usize = CLOCK_SPEED / 1_000;
const CYCLES_PER_US: usize = CLOCK_SPEED / 1_000_000;

//
// ROM
//

/// Max ROM size is 16MB
pub const MAX_ROM_SIZE: usize = 16 * 1024 * 1024;
pub const MIN_ROM_RAM_SIZE: usize = 1024;
/// Max SRAM size is 16MB
pub const MAX_ROM_RAM_SIZE: usize = 16 * 1024 * 1024;

//
// Framebuffer
//

pub const LEFT_FRAME_BUFFER_CYCLE_OFFSET: usize = CYCLES_PER_MS * 3;
pub const LEFT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET: usize = CYCLES_PER_MS * 8;
pub const FCLK_LOW_CYCLE_OFFSET: usize = CYCLES_PER_MS * 10;
pub const RIGHT_FRAME_BUFFER_CYCLE_OFFSET: usize = CYCLES_PER_MS * 13;
pub const RIGHT_FRAME_BUFFER_COMPLETE_CYCLE_OFFSET: usize = CYCLES_PER_MS * 18;
pub const FRAME_COMPLETE_CYCLE_OFFSET: usize = CYCLES_PER_MS * 20;

pub const DISPLAY_WIDTH: usize = 384;
pub const DISPLAY_HEIGHT: usize = 224;
pub const DISPLAY_PIXEL_LENGTH: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Hardware framebuffers are 256 pixels high
pub const FRAMEBUFFER_HEIGHT: usize = 256;

// Rustual-boy assumes 5ms per draw frame (one side)
// Docs indicate affine worlds go over the time allotment, and an empty frame takes 2.8ms
// We follow Rustual-boy's lead for now
pub const FRAME_RENDER_TIME_MS: usize = 5;

// Drawing occurs in vertical chunks. There are 8 rows of pixels processed at once
// We break it up like this, assuming that the entire 8 rows of pixels are drawn in one go.
// It is broken up to keep track of `SBCOUNT`
pub const DRAWING_BLOCK_COUNT: usize = (DISPLAY_HEIGHT as usize) / 8;

/// Total time (in cycles) taken to draw one block
pub const DRAWING_BLOCK_CYCLE_COUNT: usize =
    CYCLES_PER_MS * FRAME_RENDER_TIME_MS / DRAWING_BLOCK_COUNT;

/// Total time SBOUT remains high
pub const SBOUT_HIGH_CYCLE_COUNT: usize = CYCLES_PER_US * 56;

//
// Gamepad
//

pub const GAMEPAD_HARDWARE_READ_CYCLE_COUNT: usize = CLOCK_SPEED / 31_250;

//
// Timer
//

/// Minimum timer interval is 20us, and max is 100us
pub const TIMER_MIN_INTERVAL_CYCLE_COUNT: usize = CYCLES_PER_US * 20;

//
// Sound
//

// CLOCK_SPEED / 260.4
/// Number of cycles per live interval tick
pub const SOUND_LIVE_INTERVAL_CYCLE_COUNT: usize = 76_805;

/// Number of cycles per frequency tick for a waveform channel
pub const WAVE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT: usize = CLOCK_SPEED / 5_000_000;
/// Number of cycles per frequency tick for the noise channel
pub const NOISE_CHANNEL_BASE_FREQUENCY_CYCLE_COUNT: usize = CLOCK_SPEED / 500_000;
/// Number of cycles per envelope tick
pub const ENVELOPE_CYCLE_COUNT: usize = ((CLOCK_SPEED as f32) / 65.1) as usize;

/// Number of cycles in 130.2Hz sweep tick
pub const SWEEP_SLOW_CYCLE_COUNT: usize = ((CLOCK_SPEED as f32) / 130.2) as usize;
/// Number of cycles in 1041.6Hz sweep tick
pub const SWEEP_FAST_CYCLE_COUNT: usize = ((CLOCK_SPEED as f32) / 1041.6) as usize;

/// Sampled every 41.7kHz (41.666kHz so it rounds the result to 480 cycles)
pub const SOUND_SAMPLE_RATE_CYCLE_COUNT: usize = CLOCK_SPEED / 41_666;
