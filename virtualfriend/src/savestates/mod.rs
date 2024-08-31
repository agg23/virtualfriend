use std::collections::VecDeque;

use savestate::UnparsedSavestate;

use crate::System;

pub mod savestate;

const MAX_REWIND_HISTORY: usize = 200;

const FRAME_RECORDING_FREQUENCY: usize = 10;
const FRAME_REPLAY_FREQUENCY: usize = 4;

pub struct SavestateController {
    rewind_history: VecDeque<UnparsedSavestate>,

    state: State,
}

enum State {
    Standard { frame_count: usize },
    Rewind { frame_count: usize },
}

impl SavestateController {
    pub fn new() -> Self {
        Self {
            rewind_history: VecDeque::new(),
            state: State::Standard { frame_count: 0 },
        }
    }

    pub fn frame_tick(&mut self, state: &System) {
        let frame_count = match &mut self.state {
            State::Standard { frame_count } => {
                *frame_count += 1;
                *frame_count
            }
            State::Rewind { .. } => {
                self.state = State::Standard { frame_count: 0 };

                0
            }
        };

        if frame_count % FRAME_RECORDING_FREQUENCY == 0 {
            println!("Creating savestate");
            self.create_history_savestate(state);

            if self.rewind_history.len() > MAX_REWIND_HISTORY {
                self.rewind_history.pop_front();
            }
        }
    }

    pub fn rewind_tick(&mut self) -> Option<UnparsedSavestate> {
        let frame_count = match &mut self.state {
            State::Rewind { frame_count } => {
                *frame_count += 1;
                *frame_count
            }
            _ => {
                self.state = State::Rewind { frame_count: 0 };
                0
            }
        };

        if frame_count % FRAME_REPLAY_FREQUENCY == 0 {
            println!("Rewinding frame {}", self.rewind_history.len());
            self.rewind_history.pop_back()
        } else {
            None
        }
    }

    fn create_history_savestate(&mut self, state: &System) {
        let savestate = UnparsedSavestate::build(state);

        self.rewind_history.push_back(savestate);
    }
}
