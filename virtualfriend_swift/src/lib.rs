use std::sync::Mutex;

use ffi::{
    FFIFrame, FFIGamepadInputs, FFIManifest, FFIMetadata, FFIUnparsedSavestate, FFIVideoFrame,
};
use virtualfriend::{
    gamepad::GamepadInputs,
    manifest::{Manifest, Metadata},
    savestates::savestate::UnparsedSavestate,
    Frame,
};

#[swift_bridge::bridge]
mod ffi {
    #[swift_bridge(swift_repr = "struct")]
    struct FFIVideoFrame {
        left: Vec<u8>,
        right: Vec<u8>,
    }

    #[swift_bridge(swift_repr = "struct")]
    struct FFIFrame {
        video: Option<FFIVideoFrame>,
        audio_left: Vec<i16>,
        audio_right: Vec<i16>,
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

    #[swift_bridge(swift_repr = "struct")]
    struct FFIUnparsedSavestate {
        left_frame: Vec<u8>,
        right_frame: Vec<u8>,

        timestamp_s: u64,

        contents: Vec<u8>,
    }

    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new(rom_data: &[u8]) -> VirtualFriend;

        fn load_ram(&mut self, ram: &[u8]);
        fn save_ram(&self) -> Vec<u8>;

        fn apply_savestate(&mut self, savestate: FFIUnparsedSavestate);
        fn create_savestate(&self) -> FFIUnparsedSavestate;

        fn run_audio_frame(&mut self, inputs: FFIGamepadInputs, buffer_size: usize) -> FFIFrame;
    }

    extern "Rust" {
        fn load_manifest(manifest_path: String) -> Option<FFIManifest>;

        fn load_savestate(savestate_path: String) -> Option<FFIUnparsedSavestate>;

        fn unparsed_savestate_data(savestate: FFIUnparsedSavestate) -> Vec<u8>;
    }
}

pub struct VirtualFriend {
    core: Mutex<virtualfriend::VirtualFriend>,
}

impl VirtualFriend {
    fn new(rom_data: &[u8]) -> Self {
        VirtualFriend {
            core: Mutex::new(virtualfriend::VirtualFriend::new(rom_data.to_vec())),
        }
    }

    fn load_ram(&mut self, ram: &[u8]) {
        self.core.try_lock().expect("Could not acquire mutex lock for load_ram. Emulator host is misconfigured; is it running on multiple threads?").load_ram(ram.to_vec());
    }

    fn save_ram(&self) -> Vec<u8> {
        self.core.try_lock().expect("Could not acquire mutex lock for save_ram. Emulator host is misconfigured; is it running on multiple threads?").dump_ram()
    }

    fn apply_savestate(&mut self, savestate: FFIUnparsedSavestate) {
        self.core.try_lock().expect("Could not acquire mutex lock for apply_savestate. Emulator host is misconfigured; is it running on multiple threads?").load_savestate(&savestate.into());
    }

    fn create_savestate(&self) -> FFIUnparsedSavestate {
        self.core.try_lock().expect("Could not acquire mutex lock for create_savestate. Emulator host is misconfigured; is it running on multiple threads?").create_savestate().into()
    }

    fn run_audio_frame(&mut self, inputs: FFIGamepadInputs, buffer_size: usize) -> FFIFrame {
        self.core.try_lock().expect("Could not acquire mutex lock for run_audio_frame. Emulator host is misconfigured; is it running on multiple threads?").run_audio_frame(inputs.into(), buffer_size).into()
    }
}

fn load_manifest(manifest_path: String) -> Option<FFIManifest> {
    Manifest::load(manifest_path).and_then(|m| Some(m.into()))
}

fn load_savestate(savestate_path: String) -> Option<FFIUnparsedSavestate> {
    UnparsedSavestate::load_from_path(savestate_path).map(|s| s.into())
}

impl FFIFrame {}

impl From<Frame> for FFIFrame {
    fn from(value: Frame) -> Self {
        let mut audio_left = Vec::with_capacity(value.audio_buffer.len());
        let mut audio_right = Vec::with_capacity(value.audio_buffer.len());

        for (left, right) in value.audio_buffer {
            audio_left.push(left);
            audio_right.push(right);
        }

        FFIFrame {
            video: value.video.map(|frame| FFIVideoFrame {
                left: frame.left,
                right: frame.right,
            }),
            audio_left,
            audio_right,
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

impl From<UnparsedSavestate> for FFIUnparsedSavestate {
    fn from(value: UnparsedSavestate) -> Self {
        let UnparsedSavestate {
            left_frame,
            right_frame,
            timestamp_s,
            contents,
        } = value;

        Self {
            left_frame,
            right_frame,
            timestamp_s,
            contents,
        }
    }
}

impl From<FFIUnparsedSavestate> for UnparsedSavestate {
    fn from(value: FFIUnparsedSavestate) -> Self {
        let FFIUnparsedSavestate {
            left_frame,
            right_frame,
            timestamp_s,
            contents,
        } = value;

        Self {
            left_frame,
            right_frame,
            timestamp_s,
            contents,
        }
    }
}

// TODO: I don't know how to make this a struct member, so for the sake of time it's just a standalone function
fn unparsed_savestate_data(savestate: FFIUnparsedSavestate) -> Vec<u8> {
    let savestate: UnparsedSavestate = savestate.into();
    savestate.data()
}
