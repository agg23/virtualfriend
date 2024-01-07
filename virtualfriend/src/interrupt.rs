pub enum InterruptRequest {
    GamePad,
    TimerZero,
    GamePak,
    Communication,
    VIP,
}

impl InterruptRequest {
    pub fn code(&self) -> usize {
        match self {
            Self::VIP => 0xFE40,
            Self::Communication => 0xFE30,
            Self::GamePak => 0xFE20,
            Self::TimerZero => 0xFE10,
            Self::GamePad => 0xFE00,
        }
    }
}
