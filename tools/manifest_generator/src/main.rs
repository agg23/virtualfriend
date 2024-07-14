use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use virtualfriend::manifest::Metadata;
use virtualfriend_desktop::{build_client, ThreadFrame};
use winit::event_loop::EventLoop;

fn process_input(prompt: String) -> Option<String> {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic(prompt),
        DefaultPromptSegment::Empty,
    );

    match line_editor.read_line(&prompt) {
        Ok(Signal::Success(buffer)) => Some(buffer),
        Ok(Signal::CtrlC) => None,
        other => {
            println!("Error: {other:?}");

            None
        }
    }
}

fn force_process_input(prompt: String) -> String {
    loop {
        if let Some(result) = process_input(prompt.clone()) {
            return result;
        }
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 3 {
        println!("Usage: manifest_generator [path to ROMs] [path to manifest output]");

        std::process::exit(1)
    }

    let rom_path = Path::new(&args[1]);
    let manifest_path = Path::new(&args[2]);

    let paths = fs::read_dir(rom_path)
        .expect("Could not find ROM directory")
        .filter(|entry| {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "vb" {
                        return true;
                    }
                }
            }

            return false;
        });

    let mut event_loop: Option<EventLoop<()>> = None;

    for path in paths {
        // We can assert it's Ok here as it was verified in the above filter
        let path = path.unwrap();

        let rom_data = fs::read(&path.path()).expect("Could not load ROM");

        let folder_hash = format!("{:x}", md5::compute(rom_data));

        let rom_path = path.path().with_extension("");
        let rom_name = rom_path.file_name().unwrap().to_str().unwrap();

        let mut manifest_folder_path = manifest_path.to_path_buf();
        manifest_folder_path.push(folder_hash.clone());

        if manifest_folder_path.exists() {
            // Path already exists, don't process this file
            println!("Skipping {rom_name} as it already exists as {folder_hash}");
            continue;
        }

        if !process_input(format!(
            "Process \"{rom_name}\" (Enter to process, Ctrl-C to skip)? "
        ))
        .is_some()
        {
            // Skip this item

            if process_input(format!(
                "Stop processing entirely (Enter to continue processing, Ctrl-C to quit)?"
            ))
            .is_some()
            {
                // Continue processing
                continue;
            } else {
                // We're quitting
                std::process::exit(0);
            }
        }

        let title = force_process_input(format!("Title:")).trim().to_string();

        let developer = force_process_input(format!("Developer:"));
        let publisher = force_process_input(format!("Publisher:"));
        let year = force_process_input(format!("Year:"));
        let region = force_process_input(format!("Regions (comma separated):"));

        let region = region
            .split(",")
            .map(|r| r.trim().to_string())
            .collect::<Vec<String>>();

        fs::create_dir_all(&manifest_folder_path).expect("Could not create manifest folder");

        let metadata = Metadata {
            title: title.clone(),

            developer: developer.trim().to_string(),
            publisher: publisher.trim().to_string(),
            year: year.trim().to_string(),

            region,
        };

        let mut named_path = manifest_folder_path.clone();
        named_path.push(&title);

        let json_string = serde_json::to_string_pretty(&metadata).unwrap();

        let mut file = File::create(named_path.with_extension("json")).unwrap();
        file.write_all(json_string.as_bytes()).unwrap();

        event_loop = Some(build_client(
            event_loop,
            &path.path(),
            None,
            Some(|frame: &ThreadFrame| {
                let mut file = File::create(named_path.with_extension("vf")).unwrap();

                file.write(frame.left.as_slice()).unwrap();
                file.write(frame.right.as_slice()).unwrap();

                true
            }),
        ));
    }

    println!("Finished processing");
}
