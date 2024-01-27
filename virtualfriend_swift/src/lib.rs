use ffi::{FFIFrame, FFIGamepadInputs, FFIManifest};
use virtualfriend::{gamepad::GamepadInputs, manifest::Manifest, Frame};

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

        right_trigger: bool,
        left_trigger: bool,

        right_dpad_up: bool,
        right_dpad_right: bool,
        right_dpad_left: bool,
        right_dpad_down: bool,

        left_dpad_up: bool,
        left_dpad_right: bool,
        left_dpad_left: bool,
        left_dpad_down: bool,

        start: bool,
        select: bool,
    }

    #[swift_bridge(swift_repr = "struct")]
    struct FFIManifest {
        left_frame: Vec<u8>,
        right_frame: Vec<u8>,
    }

    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new(rom_path: String) -> VirtualFriend;

        fn run_frame(&mut self, inputs: FFIGamepadInputs) -> FFIFrame;
    }

    extern "Rust" {
        fn load_manifest(manifest_path: String) -> Option<FFIManifest>;
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

fn load_manifest(manifest_path: String) -> Option<FFIManifest> {
    Manifest::load(manifest_path).and_then(|m| Some(m.into()))
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
        let FFIGamepadInputs {
            a_button,
            b_button,
            right_trigger,
            left_trigger,
            right_dpad_up,
            right_dpad_right,
            right_dpad_left,
            right_dpad_down,
            left_dpad_up,
            left_dpad_right,
            left_dpad_left,
            left_dpad_down,
            start,
            select,
        } = value;

        GamepadInputs {
            a_button,
            b_button,
            right_trigger,
            left_trigger,
            right_dpad_up,
            right_dpad_right,
            right_dpad_left,
            right_dpad_down,
            left_dpad_up,
            left_dpad_right,
            left_dpad_left,
            left_dpad_down,
            start,
            select,
        }
    }
}

impl FFIManifest {}

impl From<Manifest> for FFIManifest {
    fn from(value: Manifest) -> Self {
        FFIManifest {
            left_frame: value.left_frame,
            right_frame: value.right_frame,
        }
    }
}
