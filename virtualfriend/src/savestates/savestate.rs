use std::{
    fs::read,
    time::{SystemTime, UNIX_EPOCH},
};

use savefile::{load_from_mem, save_to_mem};

use crate::System;

pub struct UnparsedSavestate {
    pub left_frame: Vec<u8>,
    pub right_frame: Vec<u8>,

    pub timestamp_s: u64,

    pub contents: Vec<u8>,
}

impl UnparsedSavestate {
    pub fn load(bytes: &[u8]) -> Self {
        let mut left_frame = vec![];
        let mut right_frame = vec![];
        let mut contents: Vec<u8> = vec![];

        left_frame.extend(&bytes[0..384 * 224]);
        right_frame.extend(&bytes[384 * 224..2 * 384 * 224]);

        let timestamp_s = u64::from_le_bytes(
            bytes[2 * 384 * 224..2 * 384 * 224 + 8]
                .try_into()
                .expect("Failed to convert slice"),
        );

        contents.extend(&bytes[2 * 384 * 224 + 8..]);

        UnparsedSavestate {
            left_frame,
            right_frame,
            timestamp_s,
            contents,
        }
    }

    pub fn load_from_path(path: String) -> Option<Self> {
        let vec = read(path).ok().expect("Failed to load from path");

        Some(UnparsedSavestate::load(&vec))
    }

    pub(crate) fn build(contents: &System) -> Self {
        Self {
            left_frame: contents.bus.vip.left_rendered_framebuffer.clone(),
            right_frame: contents.bus.vip.right_rendered_framebuffer.clone(),
            timestamp_s: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            contents: save_to_mem(0, contents).expect("Could not generate savestate"),
        }
    }

    pub(crate) fn contents(&self) -> System {
        load_from_mem::<System>(&self.contents, 0).expect("Could not parse savestate contents")
    }

    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend(&self.left_frame);
        data.extend(&self.right_frame);
        data.extend(self.timestamp_s.to_le_bytes());
        data.extend(&self.contents);

        data
    }
}
