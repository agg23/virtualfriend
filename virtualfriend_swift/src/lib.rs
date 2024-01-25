use ffi::{FFIFrame, FFIGamepadInputs};
use virtualfriend::{gamepad::GamepadInputs, Frame};

#[swift_bridge::bridge]
mod ffi {
    #[swift_bridge(swift_repr = "struct")]
    struct FFIFrame {
        left: Vec<u8>,
        right: Vec<u8>,
    }

    #[swift_bridge(swift_repr = "struct")]
    struct FFIGamepadInputs {
        a_button: bool,
        b_button: bool,
    }

    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new(rom_path: String) -> VirtualFriend;

        fn run_frame(&mut self, inputs: FFIGamepadInputs) -> FFIFrame;
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

    fn run_frame(&mut self, inputs: FFIGamepadInputs) -> FFIFrame {
        self.core.run_frame(inputs.into()).into()
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

impl FFIGamepadInputs {}

impl From<FFIGamepadInputs> for GamepadInputs {
    fn from(value: FFIGamepadInputs) -> Self {
        let FFIGamepadInputs { a_button, b_button } = value;

        GamepadInputs { a_button, b_button }
    }
}
