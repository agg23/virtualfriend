use std::collections::VecDeque;

use savestate::UnparsedSavestate;

use crate::System;

pub mod savestate;

const MAX_REWIND_HISTORY: usize = 200;

const FRAME_RECORDING_FREQUENCY: usize = 10;
const FRAME_REPLAY_FREQUENCY: usize = 4;

pub(crate) struct SavestateController {
    rewind_history: VecDeque<UnparsedSavestate>,

    state: State,
}

enum State {
    Standard { frame_count: usize },
    Rewind { frame_count: usize },
}

impl SavestateController {
    pub(crate) fn new() -> Self {
        Self {
            rewind_history: VecDeque::new(),
            state: State::Standard { frame_count: 0 },
        }
    }

    pub(crate) fn frame_tick(&mut self, state: &System) {
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
            self.create_history_savestate(state);

            if self.rewind_history.len() > MAX_REWIND_HISTORY {
                self.rewind_history.pop_front();
            }
        }
    }

    pub(crate) fn rewind_tick(&mut self) -> Option<UnparsedSavestate> {
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
            self.rewind_history.pop_back()
        } else {
            None
        }
    }

    fn create_history_savestate(&mut self, state: &System) {
        let savestate = UnparsedSavestate::build(state);

        self.rewind_history.push_back(savestate);
    }

    pub(crate) fn load_savestate_to_system(&mut self, savestate: &UnparsedSavestate) -> System {
        // Remove all rewind history when loading savestate
        self.rewind_history.clear();
        self.state = State::Standard { frame_count: 0 };

        savestate.contents()
    }
}
