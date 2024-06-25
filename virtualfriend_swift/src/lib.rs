use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    ffi::c_void,
    sync::{Mutex, RwLock},
};

use ffi::{FFIFrame, FFIGamepadInputs, FFIManifest, FFIMetadata, FFIVideoFrame};
use virtualfriend::{
    gamepad::GamepadInputs,
    manifest::{Manifest, Metadata},
    vsu::traits::AudioFrame,
    Frame, VideoFrame,
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

    extern "Rust" {
        type VirtualFriend;

        #[swift_bridge(init)]
        fn new(rom_data: &[u8]) -> VirtualFriend;

        fn load_ram(&mut self, ram: &[u8]);
        fn save_ram(&self) -> Vec<u8>;

        // fn run_audio_frame(&mut self, inputs: FFIGamepadInputs, buffer_size: usize) -> FFIFrame;
        // fn start<F>(&mut self, callback: F)
        // where
        //     F: Fn(Option<VideoFrame>) -> GamepadInputs + Send + 'static;
    }

    extern "Rust" {
        fn load_manifest(manifest_path: String) -> Option<FFIManifest>;
    }
}

#[repr(C)]
pub struct VideoCallback {
    userdata: *mut c_void,
    callback: extern "C" fn(*mut c_void, Option<FFIVideoFrame>) -> FFIGamepadInputs,
}

#[no_mangle]
pub extern "C" fn start_emulation(virtualfriend: &mut VirtualFriend, callback: VideoCallback) {
    println!("Start from Rust");
    virtualfriend.start(callback);
}

pub struct VirtualFriend {
    core: virtualfriend_runner::VirtualFriend,
    callback: RwLock<Option<VideoCallback>>,
}

impl VirtualFriend {
    fn new(rom_data: &[u8]) -> Self {
        let callback = RwLock::new(None);

        VirtualFriend {
            core: virtualfriend_runner::VirtualFriend::new(rom_data, |frame| {
                // let callback = callback.borrow();
                let callback = callback.borrow().read().unwrap();
                callback.as_ref().map_or(
                    GamepadInputs {
                        a_button: false,
                        b_button: false,

                        right_trigger: false,
                        left_trigger: false,

                        right_dpad_up: false,
                        right_dpad_right: false,
                        right_dpad_left: false,
                        right_dpad_down: false,

                        left_dpad_right: false,
                        left_dpad_left: false,
                        left_dpad_down: false,
                        left_dpad_up: false,

                        start: false,
                        select: false,
                    },
                    |callback: &VideoCallback| {
                        (callback.callback)(
                            callback.userdata,
                            frame.map(|frame| FFIVideoFrame {
                                left: frame.left,
                                right: frame.right,
                            }),
                        )
                        .into()
                    },
                )
            }),
            callback,
        }
    }

    fn load_ram(&mut self, ram: &[u8]) {
        self.core.load_ram(ram)
    }

    fn save_ram(&self) -> Vec<u8> {
        self.core.save_ram()
    }

    // fn run_audio_frame(&mut self, inputs: FFIGamepadInputs, buffer_size: usize) -> FFIFrame {
    //     self.core.try_lock().expect("Could not acquire mutex lock. Emulator host is misconfigured; is it running on multiple threads?").run_audio_frame(inputs.into(), buffer_size).into()
    // }
    fn start(&mut self, callback: VideoCallback) {
        self.callback.replace(Some(callback));
        self.core.start()
    }
}

fn load_manifest(manifest_path: String) -> Option<FFIManifest> {
    Manifest::load(manifest_path).and_then(|m| Some(m.into()))
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
