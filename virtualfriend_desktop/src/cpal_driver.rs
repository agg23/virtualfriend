// From the Rustual Boy project
// Copyright (c) 2016-2020 Jake Taylor
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![allow(dead_code)]

use cpal::{default_endpoint, EventLoop, UnknownTypeBuffer, Voice};

use futures::stream::Stream;
use futures::task::{self, Executor, Run};
use virtualfriend::vsu::traits::{AudioFrame, SinkRef, TimeSource};

use std::borrow::Cow;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use std::cmp::Ordering;

pub type CpalDriverError = Cow<'static, str>;

pub struct RingBuffer {
    inner: Box<[i16]>,

    write_pos: usize,
    read_pos: usize,

    samples_read: u64,
}

impl RingBuffer {
    fn push(&mut self, value: i16) {
        self.inner[self.write_pos] = value;

        self.write_pos += 1;
        if self.write_pos >= self.inner.len() {
            self.write_pos = 0;
        }
    }
}

impl Iterator for RingBuffer {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        let ret = self.inner[self.read_pos];

        self.read_pos += 1;
        if self.read_pos >= self.inner.len() {
            self.read_pos = 0;
        }

        self.samples_read += 1;

        Some(ret)
    }
}

struct CpalDriverBufferSink {
    ring_buffer: Arc<Mutex<RingBuffer>>,
}

impl SinkRef<[AudioFrame]> for CpalDriverBufferSink {
    fn append(&mut self, buffer: &[AudioFrame]) {
        let mut ring_buffer = self.ring_buffer.lock().unwrap();
        for &(left, right) in buffer {
            ring_buffer.push(left);
            ring_buffer.push(right);
        }
    }
}

struct CpalDriverTimeSource {
    ring_buffer: Arc<Mutex<RingBuffer>>,
    sample_rate: u32,
}

impl TimeSource for CpalDriverTimeSource {
    fn time_ns(&self) -> u64 {
        let ring_buffer = self.ring_buffer.lock().unwrap();
        1_000_000_000 * (ring_buffer.samples_read / 2) / (self.sample_rate as u64)
    }
}

struct CpalDriverExecutor;

impl Executor for CpalDriverExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

pub struct CpalDriver {
    ring_buffer: Arc<Mutex<RingBuffer>>,
    sample_rate: u32,

    _voice: Voice,
    _join_handle: JoinHandle<()>,
}

impl CpalDriver {
    pub fn new(sample_rate: u32, desired_latency_ms: u32) -> Result<CpalDriver, CpalDriverError> {
        if desired_latency_ms == 0 {
            return Err(format!("desired_latency_ms must be greater than 0").into());
        }

        let endpoint = default_endpoint().expect("Failed to get audio endpoint");

        let compare_sample_rates = |x: u32, y: u32| -> Ordering {
            if x < sample_rate && y > sample_rate {
                return Ordering::Greater;
            } else if x > sample_rate && y < sample_rate {
                return Ordering::Less;
            } else if x < sample_rate && y < sample_rate {
                return x.cmp(&y).reverse();
            } else {
                return x.cmp(&y);
            }
        };

        let format = endpoint
            .supported_formats()
            .expect("Failed to get supported format list for endpoint")
            .filter(|format| format.channels.len() == 2)
            .min_by(|x, y| compare_sample_rates(x.samples_rate.0, y.samples_rate.0))
            .expect("Failed to find format with 2 channels");

        let buffer_frames = (sample_rate * desired_latency_ms / 1000 * 2) as usize;
        let ring_buffer = Arc::new(Mutex::new(RingBuffer {
            inner: vec![0; buffer_frames].into_boxed_slice(),

            write_pos: 0,
            read_pos: 0,

            samples_read: 0,
        }));

        let event_loop = EventLoop::new();

        let (mut voice, stream) =
            Voice::new(&endpoint, &format, &event_loop).expect("Failed to create voice");
        voice.play();

        let mut resampler = LinearResampler::new(sample_rate as _, format.samples_rate.0 as _);

        let read_ring_buffer = ring_buffer.clone();
        task::spawn(stream.for_each(move |output_buffer| {
            let mut read_ring_buffer = read_ring_buffer.lock().unwrap();

            match output_buffer {
                UnknownTypeBuffer::I16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = resampler.next(&mut *read_ring_buffer);
                        }
                    }
                }
                UnknownTypeBuffer::U16(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = ((resampler.next(&mut *read_ring_buffer) as i32) + 32768) as u16;
                        }
                    }
                }
                UnknownTypeBuffer::F32(mut buffer) => {
                    for sample in buffer.chunks_mut(format.channels.len()) {
                        for out in sample.iter_mut() {
                            *out = (resampler.next(&mut *read_ring_buffer) as f32) / 32768.0;
                        }
                    }
                }
            }

            Ok(())
        }))
        .execute(Arc::new(CpalDriverExecutor));

        let join_handle = thread::spawn(move || {
            event_loop.run();
        });

        Ok(CpalDriver {
            ring_buffer: ring_buffer,
            sample_rate: sample_rate,

            _voice: voice,
            _join_handle: join_handle,
        })
    }

    pub fn sink(&self) -> Box<dyn SinkRef<[AudioFrame]>> {
        Box::new(CpalDriverBufferSink {
            ring_buffer: self.ring_buffer.clone(),
        })
    }

    pub fn time_source(&self) -> Box<dyn TimeSource> {
        Box::new(CpalDriverTimeSource {
            ring_buffer: self.ring_buffer.clone(),
            sample_rate: self.sample_rate,
        })
    }
}

struct LinearResampler {
    from_sample_rate: u32,
    to_sample_rate: u32,

    current_from_frame: AudioFrame,
    next_from_frame: AudioFrame,
    from_fract_pos: u32,

    current_frame_channel_offset: u32,
}

impl LinearResampler {
    fn new(from_sample_rate: u32, to_sample_rate: u32) -> LinearResampler {
        let sample_rate_gcd = {
            fn gcd(a: u32, b: u32) -> u32 {
                if b == 0 {
                    a
                } else {
                    gcd(b, a % b)
                }
            }

            gcd(from_sample_rate, to_sample_rate)
        };

        LinearResampler {
            from_sample_rate: from_sample_rate / sample_rate_gcd,
            to_sample_rate: to_sample_rate / sample_rate_gcd,

            current_from_frame: (0, 0),
            next_from_frame: (0, 0),
            from_fract_pos: 0,

            current_frame_channel_offset: 0,
        }
    }

    fn next(&mut self, input: &mut dyn Iterator<Item = i16>) -> i16 {
        fn interpolate(a: i16, b: i16, num: u32, denom: u32) -> i16 {
            (((a as i32) * ((denom - num) as i32) + (b as i32) * (num as i32)) / (denom as i32))
                as _
        }

        let ret = match self.current_frame_channel_offset {
            0 => interpolate(
                self.current_from_frame.0,
                self.next_from_frame.0,
                self.from_fract_pos,
                self.to_sample_rate,
            ),
            _ => interpolate(
                self.current_from_frame.1,
                self.next_from_frame.1,
                self.from_fract_pos,
                self.to_sample_rate,
            ),
        };

        self.current_frame_channel_offset += 1;
        if self.current_frame_channel_offset >= 2 {
            self.current_frame_channel_offset = 0;

            self.from_fract_pos += self.from_sample_rate;
            while self.from_fract_pos > self.to_sample_rate {
                self.from_fract_pos -= self.to_sample_rate;

                self.current_from_frame = self.next_from_frame;

                let left = input.next().unwrap_or(0);
                let right = input.next().unwrap_or(0);
                self.next_from_frame = (left, right);
            }
        }

        ret
    }
}
