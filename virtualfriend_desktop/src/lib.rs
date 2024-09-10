mod audio_driver;
mod linear_resampler;

use std::{
    collections::VecDeque,
    fs::{self},
    path::Path,
    sync::{Arc, Mutex},
};

use audio_driver::AudioDriver;
use pixels::{Pixels, SurfaceTexture};
use single_value_channel::channel_starting_with;
use virtualfriend::{gamepad::GamepadInputs, Frame, VideoFrame, VirtualFriend};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement, run_on_demand::EventLoopExtRunOnDemand,
    },
    window::WindowBuilder,
};

const DISPLAY_WIDTH: usize = 384;
const DISPLAY_MARGIN: usize = 40;
const DISPLAY_HEIGHT: usize = 224;

const COMBO_DISPLAY_WIDTH: usize = DISPLAY_WIDTH * 2 + DISPLAY_MARGIN;

const WINDOW_WIDTH: usize = COMBO_DISPLAY_WIDTH * 3;
const WINDOW_HEIGHT: usize = DISPLAY_HEIGHT * 3;

pub struct ThreadFrame {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
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

// struct RGB {
//     red: u8,
//     green: u8,
//     blue: u8,
// }

// impl RGB {
//     fn mix(&self, a_weight: u8, b_weight: u8, b: &RGB) -> RGB {
//         let calculate = |a: u8, b: u8| -> u8 {
//             let value = (a_weight as u16) * (a as u16) + (b_weight as u16) * (b as u16);
//             (value >> 8) as u8
//         };

//         let red = calculate(self.red, b.red);
//         let green = calculate(self.green, b.green);
//         let blue = calculate(self.blue, b.blue);

//         RGB { red, green, blue }
//     }
// }

pub fn build_client<F: Fn(&ThreadFrame) -> bool>(
    event_loop: Option<EventLoop<()>>,
    rom_path: &Path,
    save_path: Option<&Path>,
    savestate_path: Option<&Path>,
    capture_callback: Option<F>,
) -> EventLoop<()> {
    // Window
    let mut event_loop =
        event_loop.map_or_else(|| EventLoop::new().unwrap(), |event_loop| event_loop);
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
    let (mut inputs_receiver, inputs_transmitter) =
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

    let (mut rewind_receiver, rewind_transmitter) = channel_starting_with::<bool>(false);

    let rom = fs::read(&rom_path).expect("Could not load ROM");
    let mut virtualfriend = VirtualFriend::new(rom);

    if let Some(save_path) = save_path {
        if let Ok(ram) = fs::read(save_path) {
            // We have save RAM. Upload it
            println!("Loading save");
            virtualfriend.load_ram(ram);
        }
    }

    let mut frame_id = 0;
    let virtualfriend = Arc::new(Mutex::new(virtualfriend));

    let virtualfriend_audio = virtualfriend.clone();

    // 41.667kHz
    let mut audio_driver = AudioDriver::new(41667, 20, move |sample_count| {
        let frame = if *rewind_receiver.latest() {
            // Rewinding
            let video = virtualfriend_audio.lock().unwrap().run_rewind_frame();

            Frame {
                video,
                audio_buffer: vec![],
            }
        } else {
            // Normal frame
            virtualfriend_audio
                .lock()
                .unwrap()
                .run_audio_frame(inputs_receiver.latest().clone(), sample_count)
        };

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

    // let red_base = RGB {
    //     red: 0xFF,
    //     green: 0,
    //     blue: 0,
    // };

    // let blue_base = RGB {
    //     red: 0,
    //     green: 0,
    //     blue: 0xFF,
    // };

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
        .run_on_demand(move |event, window_target| {
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

                        if let Some(capture_callback) = &capture_callback {
                            if capture_callback(frame) {
                                // Terminate
                                window_target.exit();
                            }
                        }
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
                        // Update save
                        let ram = virtualfriend.lock().unwrap().dump_ram();
                        if let Some(save_path) = save_path {
                            fs::write(save_path, ram).unwrap();
                        }

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
                            if pressed {
                                println!("Pressing c");
                                capture_next_frame = true;
                            }
                        }
                        Key::Character("s") => {
                            if pressed {
                                println!("Pressing s");
                                let savestate = virtualfriend.lock().unwrap().create_savestate();
                                if let Some(savestate_path) = savestate_path {
                                    fs::write(savestate_path, savestate.data()).unwrap();
                                }
                            }
                        }
                        Key::Character("p") => {
                            if pressed {
                                println!("Pressing p");
                                if let Some(savestate_path) = savestate_path {
                                    let savestate = fs::read(savestate_path).unwrap();

                                    virtualfriend
                                        .lock()
                                        .unwrap()
                                        .load_savestate_from_bytes(&savestate);
                                }
                            }
                        }
                        Key::Character("r") => {
                            rewind_transmitter.update(pressed).unwrap();
                        }
                        _ => {}
                    }

                    inputs_transmitter.update(inputs).unwrap();
                }
                _ => {}
            }
        })
        .unwrap();

    audio_driver.shutdown();

    println!("Terminating event loop");

    return event_loop;
}
