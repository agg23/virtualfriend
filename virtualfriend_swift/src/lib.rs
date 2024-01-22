use ffi::FFIFrame;
use virtualfriend::{gamepad::GamepadInputs, Frame};

#[swift_bridge::bridge]
mod ffi {
    #[swift_bridge(swift_repr = "struct")]
    struct FFIFrame {
        left: Vec<u8>,
        right: Vec<u8>,
    }

    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new(rom_path: String) -> VirtualFriend;

        fn run_frame(&mut self) -> FFIFrame;
    }
}

pub struct VirtualFriend {
    core: virtualfriend::VirtualFriend,
}

impl VirtualFriend {
    fn new(rom_path: String) -> Self {
        VirtualFriend {
            core: virtualfriend::VirtualFriend::new(rom_path),
        }
    }

    fn run_frame(&mut self) -> FFIFrame {
        self.core
            .run_frame(GamepadInputs {
                a_button: false,
                b_button: false,
            })
            .into()
    }
}

impl FFIFrame {}

impl From<Frame> for FFIFrame {
    fn from(value: Frame) -> Self {
        FFIFrame {
            left: value.left,
            right: value.right,
        }
    }
}
