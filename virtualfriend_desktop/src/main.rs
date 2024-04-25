mod audio_driver;
mod linear_resampler;

use std::{
    collections::VecDeque,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use audio_driver::AudioDriver;
use pixels::{Pixels, SurfaceTexture};
use single_value_channel::channel_starting_with;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowBuilder,
};

use virtualfriend::gamepad::GamepadInputs;
use virtualfriend::VideoFrame;
use virtualfriend::VirtualFriend;

const DISPLAY_WIDTH: usize = 384;
const DISPLAY_MARGIN: usize = 40;
const DISPLAY_HEIGHT: usize = 224;

const COMBO_DISPLAY_WIDTH: usize = DISPLAY_WIDTH * 2 + DISPLAY_MARGIN;

const WINDOW_WIDTH: usize = COMBO_DISPLAY_WIDTH * 3;
const WINDOW_HEIGHT: usize = DISPLAY_HEIGHT * 3;

struct ThreadFrame {
    left: Vec<u8>,
    right: Vec<u8>,
    id: usize,
}

impl ThreadFrame {
    fn from(value: VideoFrame, id: usize) -> Self {
        ThreadFrame {
            left: value.left,
            right: value.right,
            id,
        }
    }
}

struct RGB {
    red: u8,
    green: u8,
    blue: u8,
}

impl RGB {
    fn mix(&self, a_weight: u8, b_weight: u8, b: &RGB) -> RGB {
        let calculate = |a: u8, b: u8| -> u8 {
            let value = (a_weight as u16) * (a as u16) + (b_weight as u16) * (b as u16);
            (value >> 8) as u8
        };

        let red = calculate(self.red, b.red);
        let green = calculate(self.green, b.green);
        let blue = calculate(self.blue, b.blue);

        RGB { red, green, blue }
    }
}

fn main() {
    // Window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32))
        .with_title("Virtualfriend")
        .build(&event_loop)
        .unwrap();

    let surface_texture = SurfaceTexture::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, &window);

    let mut pixels = Pixels::new(
        COMBO_DISPLAY_WIDTH as u32,
        DISPLAY_HEIGHT as u32,
        surface_texture,
    )
    .unwrap();

    let buffer = pixels.frame_mut();

    // Border between eyes
    for y in 0..DISPLAY_HEIGHT {
        for x in DISPLAY_WIDTH..DISPLAY_WIDTH + DISPLAY_MARGIN {
            let base_address = (y * COMBO_DISPLAY_WIDTH + x) * 4;

            buffer[base_address] = 0x30;
            buffer[base_address + 1] = 0x30;
            buffer[base_address + 2] = 0x30;
        }
    }

    for i in 0..COMBO_DISPLAY_WIDTH * DISPLAY_HEIGHT {
        // Set all alphas
        buffer[i * 4 + 3] = 0xFF;
    }

    let mut initial_framebuffer = Vec::with_capacity(DISPLAY_HEIGHT * DISPLAY_WIDTH);

    for _ in 0..DISPLAY_HEIGHT * DISPLAY_WIDTH {
        initial_framebuffer.push(0);
    }

    // Multithreading
    let mut last_frame_id = 0;
    let (mut buffer_receiver, buffer_transmitter) =
        channel_starting_with::<ThreadFrame>(ThreadFrame {
            left: initial_framebuffer.clone(),
            right: initial_framebuffer.clone(),
            id: last_frame_id,
        });
    let (mut inputs_receiver, mut inputs_transmitter) =
        channel_starting_with::<GamepadInputs>(GamepadInputs {
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
        });

    let rom_path = Path::new(
        "/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Mario's Tennis (Japan, USA).vb",
        // "/Users/adam/Downloads/mednafen/Nintendo - Virtual Boy/Virtual Boy Wario Land (Japan, USA).vb"
    );

    let rom = fs::read(rom_path).unwrap();
    let mut virtualfriend = VirtualFriend::new(rom);

    let mut frame_id = 0;

    // 41.667kHz
    let audio_driver = AudioDriver::new(41667, 20, move |sample_count| {
        let frame = virtualfriend.run_audio_frame(inputs_receiver.latest().clone(), sample_count);

        if let Some(video) = frame.video {
            // Send updated video frame
            frame_id += 1;

            if buffer_transmitter
                .update(ThreadFrame::from(video, frame_id))
                .is_err()
            {
                println!("Could not update frame");
            };
        }

        VecDeque::from(frame.audio_buffer)
    });

    let red_base = RGB {
        red: 0xFF,
        green: 0,
        blue: 0,
    };

    let blue_base = RGB {
        red: 0,
        green: 0,
        blue: 0xFF,
    };

    let mut inputs = GamepadInputs {
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
    };

    let mut capture_next_frame = false;

    event_loop
        .run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);

            let latest_frame = buffer_receiver.latest();
            if latest_frame.id != last_frame_id {
                last_frame_id = latest_frame.id;
                // println!("Drawing frame");

                window.request_redraw();
            }

            match event {
                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::RedrawRequested,
                } => {
                    let buffer = pixels.frame_mut();

                    let frame = buffer_receiver.latest();

                    if capture_next_frame {
                        capture_next_frame = false;

                        let mut base_path = PathBuf::from(rom_path);
                        base_path.set_extension("vf");

                        let mut file = File::create(base_path).unwrap();

                        file.write(frame.left.as_slice());
                        file.write(frame.right.as_slice());
                    }

                    for i in 0..frame.left.len() {
                        let value = frame.left[i];

                        let y = i / DISPLAY_WIDTH;
                        let x = i % DISPLAY_WIDTH;

                        let base_address = (y * COMBO_DISPLAY_WIDTH + x) * 4;

                        // Only set red
                        buffer[base_address] = value;
                        // buffer[base_address + 1] = value;
                        // buffer[base_address + 2] = value;
                    }

                    for i in 0..frame.right.len() {
                        let value = frame.right[i];

                        let y = i / DISPLAY_WIDTH;
                        let x = i % DISPLAY_WIDTH;

                        let base_address =
                            (y * COMBO_DISPLAY_WIDTH + DISPLAY_WIDTH + DISPLAY_MARGIN + x) * 4;

                        // Only set red
                        buffer[base_address] = value;
                        // buffer[base_address + 1] = value;
                        // buffer[base_address + 2] = value;
                    }

                    // For stereoscopic 3D
                    // for i in 0..frame.left.len() {
                    //     let left_value = frame.left[i];
                    //     let right_value = frame.right[i];

                    //     let y = i / DISPLAY_WIDTH;
                    //     let x = i % DISPLAY_WIDTH;

                    //     let base_address = (y * COMBO_DISPLAY_WIDTH + x) * 4;

                    //     let color = red_base.mix(left_value, right_value, &blue_base);

                    //     // Only set red
                    //     buffer[base_address] = color.red;
                    //     buffer[base_address + 1] = color.green;
                    //     buffer[base_address + 2] = color.blue;
                    // }

                    // println!("Updaing buffer");
                    pixels.render().unwrap();
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CloseRequested,
                } => {
                    if window_id == window.id() {
                        window_target.exit();
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        },
                } => {
                    if window_id != window.id() {
                        return;
                    }

                    let pressed = event.state == ElementState::Pressed;

                    match event.key_without_modifiers().as_ref() {
                        Key::Character("x") => {
                            inputs.a_button = pressed;
                        }
                        Key::Character("z") => {
                            inputs.b_button = pressed;
                        }
                        Key::Named(NamedKey::Enter) => {
                            inputs.start = pressed;
                        }
                        Key::Named(NamedKey::Tab) => {
                            inputs.select = pressed;
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            inputs.left_dpad_up = pressed;
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            inputs.left_dpad_down = pressed;
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            inputs.left_dpad_left = pressed;
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            inputs.left_dpad_right = pressed;
                        }
                        Key::Character("i") => {
                            inputs.right_dpad_up = pressed;
                        }
                        Key::Character("k") => {
                            inputs.right_dpad_down = pressed;
                        }
                        Key::Character("j") => {
                            inputs.right_dpad_left = pressed;
                        }
                        Key::Character("l") => {
                            inputs.right_dpad_right = pressed;
                        }
                        Key::Character("q") => {
                            inputs.left_trigger = pressed;
                        }
                        Key::Character("w") => {
                            inputs.right_trigger = pressed;
                        }
                        Key::Character("c") => {
                            capture_next_frame = true;
                        }
                        _ => {}
                    }

                    inputs_transmitter.update(inputs).unwrap();
                }
                _ => {}
            }
        })
        .unwrap();
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
