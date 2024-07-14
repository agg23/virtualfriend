use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub struct Manifest {
    pub left_frame: Vec<u8>,
    pub right_frame: Vec<u8>,

    pub metadata: Option<Metadata>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,

    pub developer: String,
    pub publisher: String,
    pub year: String,

    pub region: Vec<String>,
}

impl Manifest {
    // TODO: Can this be made a result?
    pub fn load(base_path: String) -> Option<Self> {
        let base_path = Path::new(&base_path);
        let frame_path = base_path.with_extension("vf");
        let metadata_path = base_path.with_extension("json");

        let mut frame_file = File::open(frame_path).ok()?;

        let mut left_frame = vec![0; 384 * 224];
        let mut right_frame = vec![0; 384 * 224];

        frame_file.read(left_frame.as_mut_slice()).ok()?;
        frame_file.read(right_frame.as_mut_slice()).ok()?;

        let metadata = Self::load_metadata(metadata_path);

        Some(Manifest {
            left_frame,
            right_frame,

            metadata,
        })
    }

    fn load_metadata(metadata_path: PathBuf) -> Option<Metadata> {
        let mut metadata_file = File::open(metadata_path).ok()?;
        let mut metadata_json = String::new();

        metadata_file.read_to_string(&mut metadata_json).ok()?;

        let metadata = serde_json::from_str::<Metadata>(&metadata_json).ok()?;

        Some(metadata)
    }
}
