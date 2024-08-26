use std::{cmp::Ordering, collections::VecDeque};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    OutputCallbackInfo, Stream,
};
use virtualfriend::vsu::traits::AudioFrame;

use crate::linear_resampler::LinearResampler;

pub struct AudioDriver {
    stream: Stream,
}

impl AudioDriver {
    pub fn new<TFunc>(
        sample_rate: u32,
        _desired_latency_ms: u32,
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

        println!("Resampling to {}", config.max_sample_rate().0);

        // let buffer_frames = (sample_rate * desired_latency_ms / 1000 * 2) as usize;

        // let mut resampler: Option<SincFixedOut<f32>> = None;
        // let mut resampler: Option<FftFixedInOut<f32>> = None;
        // let mut resample_buffers: Option<(Vec<Vec<f32>>, Vec<Vec<f32>>)> = None;

        let mut resampler = LinearResampler::new(sample_rate, config.max_sample_rate().0);

        let rate_ratio = (config.max_sample_rate().0 as f32) / (sample_rate as f32);

        println!("Converting with ratio: {rate_ratio}");

        let stream = output_device
            .build_output_stream(
                &config.with_max_sample_rate().into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    // let local_resampler = resampler.get_or_insert_with(|| {
                    //     SincFixedOut::new(
                    //         (config.max_sample_rate().0 as f64) / (sample_rate as f64),
                    //         10.0,
                    //         SincInterpolationParameters {
                    //             sinc_len: 256,
                    //             f_cutoff: 0.95,
                    //             oversampling_factor: 128,
                    //             interpolation: SincInterpolationType::Quadratic,
                    //             window: WindowFunction::Blackman,
                    //         },
                    //         data.len() / 2,
                    //         2,
                    //     )
                    //     .unwrap()
                    // });
                    // let local_resampler = resampler.get_or_insert_with(|| {
                    //     FftFixedOut::new(
                    //         sample_rate as usize,
                    //         config.max_sample_rate().0 as usize,
                    //         data.len() / 2 as usize,
                    //         100,
                    //         2,
                    //     )
                    //     .unwrap()
                    // });
                    // let local_resampler = resampler.get_or_insert_with(|| {
                    //     FftFixedInOut::new(
                    //         sample_rate as usize,
                    //         config.max_sample_rate().0 as usize,
                    //         484,
                    //         2,
                    //     )
                    //     .unwrap()
                    // });

                    // let (input_resample_buffer, output_resample_buffer) = resample_buffers
                    //     .get_or_insert_with(|| {
                    //         (
                    //             local_resampler.input_buffer_allocate(false),
                    //             vec![vec![0.0f32; local_resampler.output_frames_max()]; 2],
                    //         )
                    //     });

                    // println!("Frames requested: {}", local_resampler.input_frames_next());
                    let frame_buffer =
                        on_frame_request(((data.len() / 2) as f32 / rate_ratio).ceil() as usize);

                    // // Save buffer
                    // input_resample_buffer[0].clear();
                    // input_resample_buffer[1].clear();

                    // for frame in frame_buffer.into_iter() {
                    //     // input_resample_buffer[0].push((frame.0 as f32) / 32768.0);
                    //     // input_resample_buffer[1].push((frame.1 as f32) / 32768.0);

                    // }

                    let mut input_frame_iter = frame_buffer.into_iter();

                    for output_frame in data.chunks_mut(2) {
                        let mut frame_iter = output_frame.iter_mut();
                        let channel0 = frame_iter.next().unwrap();
                        let channel1 = frame_iter.next().unwrap();

                        let (left, right) = resampler.next(&mut input_frame_iter);

                        *channel0 = (left as f32) / 32768.0;
                        *channel1 = (right as f32) / 32768.0;
                    }

                    // // Resample to system format
                    // local_resampler
                    //     .process_into_buffer(
                    //         input_resample_buffer.as_slice(),
                    //         output_resample_buffer.as_mut_slice(),
                    //         None,
                    //     )
                    //     .expect("Could not resample");

                    // // if initial_buffer {
                    // //     initial_buffer = false;

                    // //     println!("Writing initial buffer");

                    // //     for value in data.iter_mut() {
                    // //         *value = 0.0;
                    // //     }

                    // //     return;
                    // // }

                    // // Copy frames from emulator buffer to soundcard buffer
                    // for (output_frame, input_frame) in
                    //     data.chunks_mut(2).zip(output_resample_buffer.iter())
                    // {
                    //     let mut frame_iter = output_frame.iter_mut();
                    //     let channel0 = frame_iter.next().unwrap();
                    //     let channel1 = frame_iter.next().unwrap();

                    //     // Remove entries from buffer as we use them
                    //     *channel0 = input_frame[0];
                    //     *channel1 = input_frame[1];

                    //     // println!("{channel0}");

                    //     // if *channel0 > 0.0 {
                    //     //     println!("{channel0}");
                    //     // }
                    // }
                },
                move |err| println!("Audio error: {err}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Self { stream }
    }

    pub fn shutdown(&mut self) {
        self.stream.pause().unwrap();
    }
}
