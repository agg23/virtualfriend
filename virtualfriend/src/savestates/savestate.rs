use savefile::{load_from_mem, save_to_mem};
use serde::{Deserialize, Serialize};

use crate::System;

#[derive(Serialize, Deserialize)]
pub struct UnparsedSavestate {
    pub left_frame: Vec<u8>,
    pub right_frame: Vec<u8>,

    pub contents: Vec<u8>,
}

impl UnparsedSavestate {
    pub fn build(contents: &System) -> Self {
        Self {
            left_frame: contents.bus.vip.left_rendered_framebuffer.clone(),
            right_frame: contents.bus.vip.right_rendered_framebuffer.clone(),
            contents: save_to_mem(0, contents).expect("Could not generate savestate"),
        }
    }

    pub fn contents(&self) -> System {
        load_from_mem::<System>(&self.contents, 0).expect("Could not parse savestate contents")
    }
}
