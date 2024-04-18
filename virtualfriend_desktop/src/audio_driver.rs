use std::{cmp::Ordering, collections::VecDeque};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    OutputCallbackInfo, Stream,
};
use virtualfriend::vsu::traits::AudioFrame;

pub struct AudioDriver {
    stream: Stream,
}

impl AudioDriver {
    pub fn new<TFunc>(
        sample_rate: u32,
        desired_latency_ms: u32,
        mut on_frame_request: TFunc,
    ) -> Self
    where
        TFunc: FnMut(usize) -> VecDeque<AudioFrame> + Send + 'static,
    {
        let host = cpal::default_host();

        let output_device = host
            .default_output_device()
            .expect("Could not find default audio output device");

        let config = output_device
            .supported_output_configs()
            .expect("Could not get supported audio formats")
            .filter(|config| config.channels() == 2)
            .min_by(|a, b| {
                let a_sample_rate = a.min_sample_rate().0;
                let b_sample_rate = b.min_sample_rate().0;

                if a_sample_rate < sample_rate && b_sample_rate > sample_rate {
                    return Ordering::Greater;
                } else if a_sample_rate > sample_rate && b_sample_rate < sample_rate {
                    return Ordering::Less;
                } else if a_sample_rate < sample_rate && b_sample_rate < sample_rate {
                    return a_sample_rate.cmp(&b_sample_rate).reverse();
                } else {
                    return a_sample_rate.cmp(&b_sample_rate);
                }
            })
            .expect("Could not find a suitable sample rate");

        let buffer_frames = (sample_rate * desired_latency_ms / 1000 * 2) as usize;

        // let buffer = Arc::new(Mutex::new(VecDeque::<AudioFrame>::new()));

        // let config = StreamConfig {
        //     channels: config.channels(),
        //     sample_rate: config.min_sample_rate(),
        //     buffer_size: BufferSize::Fixed(buffer_frames as u32),
        // };

        let mut buffer = VecDeque::<VecDeque<AudioFrame>>::new();

        let mut initial_buffer = true;

        let stream = output_device
            .build_output_stream(
                // &config.with_max_sample_rate().into(),
                &output_device.default_output_config().unwrap().into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    let frame_buffer = on_frame_request(data.len());

                    if initial_buffer {
                        initial_buffer = false;

                        println!("Writing initial buffer");

                        for value in data.iter_mut() {
                            *value = 0.0;
                        }

                        return;
                    }

                    // Save buffer
                    buffer.push_back(frame_buffer);

                    if let Some(mut frame_buffer) = buffer.pop_front() {
                        // Copy frames from emulator buffer to soundcard buffer
                        for frame in data.chunks_mut(2) {
                            let mut frame_iter = frame.iter_mut();
                            let channel0 = frame_iter.next().unwrap();
                            let channel1 = frame_iter.next().unwrap();

                            // Remove entries from buffer as we use them
                            if let Some(incoming_frame) = frame_buffer.pop_front() {
                                *channel0 = (incoming_frame.0 as f32) / 32768.0;
                                *channel1 = (incoming_frame.1 as f32) / 32768.0;
                            } else {
                                println!("Attempted to copy buffer that does not exist");
                            }
                        }
                    } else {
                        println!("No audio frame to send. Buffer underrun");
                    }
                },
                move |err| println!("Audio error: {err}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Self { stream }
    }
}
