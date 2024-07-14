mod audio_driver;
mod linear_resampler;

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use virtualfriend_desktop::{build_client, ThreadFrame};

fn main() {
    // let rom_name = "Red Alarm (USA)";
    let rom_name = "Virtual Boy Wario Land (Japan, USA)";

    let rom_directory = Path::new("/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/");

    let rom_path = rom_directory.join(format!("{rom_name}.vb"));
    let save_path = rom_directory.join(format!("{rom_name}.sav"));

    build_client(
        None,
        &rom_path,
        Some(&save_path),
        Some(|frame: &ThreadFrame| {
            let mut base_path = PathBuf::from(&rom_path);
            base_path.set_extension("vf");

            let mut file = File::create(base_path).unwrap();

            file.write(frame.left.as_slice()).unwrap();
            file.write(frame.right.as_slice()).unwrap();

            false
        }),
    );
}

// fn create_emulator(
//     buffer_transmitter: Updater<ThreadFrame>,
//     mut inputs_receiver: Receiver<GamepadInputs>,
//     rom_path: String,
// ) {
//     thread::spawn(move || {
//         let rom = fs::read(rom_path).unwrap();
//         let mut virtualfriend = VirtualFriend::new(rom);

//         let mut frame_id = 0;

//         loop {
//             let frame = virtualfriend.run_frame(inputs_receiver.latest().clone());

//             frame_id += 1;

//             buffer_transmitter
//                 .update(ThreadFrame::from(frame, frame_id))
//                 .expect("Could not update frame");
//         }
//     });
// }
