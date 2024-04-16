use std::{fs, path::Path};

use ffi::{FFIFrame, FFIGamepadInputs, FFIManifest, FFIMetadata};
use virtualfriend::{
    gamepad::GamepadInputs,
    manifest::{Manifest, Metadata},
    Frame,
};

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
    struct FFIMetadata {
        title: String,

        developer: String,
        publisher: String,
        year: String,

        region: Vec<String>,
    }

    #[swift_bridge(swift_repr = "struct")]
    struct FFIManifest {
        left_frame: Vec<u8>,
        right_frame: Vec<u8>,

        metadata: Option<FFIMetadata>,
    }

    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new() -> VirtualFriend;

        fn run_frame(&mut self) -> FFIFrame;
    }

    extern "Rust" {
        fn load_manifest(manifest_path: String) -> Option<FFIManifest>;
    }
}

pub struct VirtualFriend {
    core: virtualfriend::VirtualFriend,
}

#[no_mangle]
pub extern "C" fn vb_make() -> *mut std::ffi::c_void {
    let b = Box::new(VirtualFriend::new());
    Box::into_raw(b).cast()
}

impl VirtualFriend {
    fn new() -> Self {
        let rom_path = Path::new(
            "/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Mario's Tennis (Japan, USA).vb",
        );

        let rom = fs::read(rom_path).unwrap();

        let mut virtual_friend = VirtualFriend {
            core: virtualfriend::VirtualFriend::new(rom),
        };

        let inputs = GamepadInputs {
            a_button: false,
            b_button: false,

            right_trigger: false,
            left_trigger: false,

            right_dpad_up: false,
            right_dpad_right: false,
            right_dpad_left: false,
            right_dpad_down: false,

            left_dpad_up: false,
            left_dpad_right: false,
            left_dpad_left: false,
            left_dpad_down: false,

            start: false,
            select: false,
        };

        loop {
            virtual_friend.core.run_frame(inputs.into());
        }

        virtual_friend
    }

    fn run_frame(&mut self) -> FFIFrame {
        let inputs = GamepadInputs {
            a_button: false,
            b_button: false,

            right_trigger: false,
            left_trigger: false,

            right_dpad_up: false,
            right_dpad_right: false,
            right_dpad_left: false,
            right_dpad_down: false,

            left_dpad_up: false,
            left_dpad_right: false,
            left_dpad_left: false,
            left_dpad_down: false,

            start: false,
            select: false,
        };

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

            metadata: value.metadata.map(|m| m.into()),
        }
    }
}

impl FFIMetadata {}

impl From<Metadata> for FFIMetadata {
    fn from(value: Metadata) -> Self {
        let Metadata {
            title,
            developer,
            publisher,
            year,
            region,
        } = value;

        FFIMetadata {
            title,
            developer,
            publisher,
            year,
            region,
        }
    }
}
