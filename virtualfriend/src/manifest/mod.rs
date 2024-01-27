use std::{fs::File, io::Read};

pub struct Manifest {
    pub left_frame: Vec<u8>,
    pub right_frame: Vec<u8>,
}

impl Manifest {
    pub fn load(path: String) -> Option<Self> {
        let mut file = File::open(path).ok()?;

        let mut left_frame = vec![0; 384 * 224];
        let mut right_frame = vec![0; 384 * 224];

        file.read(left_frame.as_mut_slice()).ok()?;
        file.read(right_frame.as_mut_slice()).ok()?;

        Some(Manifest {
            left_frame,
            right_frame,
        })
    }
}
