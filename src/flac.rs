use claxon::{FlacReader, frame::{FrameReader, FrameResult, Block}, input::ReadBytes};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::convert::AsRef;
use std::path::Path;
use std::time::Duration;
use rodio::Sink;
use rodio::Source;
use rodio::static_buffer::StaticSamplesBuffer;
use rodio::Sample;
use lazy_static;
use std::sync::Mutex;
use pulse_simple::Playback;
extern crate pulse_simple;
use pulse_simple::ChannelCount;

lazy_static! {
    static ref SAMPLER: Mutex<Vec<i16>> = Mutex::new(Vec::new());
}

pub struct FlacDecoder {
    reader: FlacReader<File>,
    current_frame: Block,
    current_frame_channel: u32,
    current_frame_sample_pos: u32,
    current_time: u32,
    sample_rate: u32,
    max_block_len: usize,
}

impl FlacDecoder {

    pub fn new(mut data: &Path) -> Self {

        let mut reader = FlacReader::open(data).expect("failed to open FLAC stream");
        let num_channels = reader.streaminfo().channels;
        let sample_rate = reader.streaminfo().sample_rate;
        let max_block_len = reader.streaminfo().max_block_size as usize * num_channels as usize;
        let mut f_reader = reader.blocks();
        let mut sample_buffer = Vec::with_capacity(max_block_len);
        let current_frame = f_reader.read_next_or_eof(sample_buffer).unwrap().unwrap();
        let current_time = current_frame.duration() / sample_rate;

        FlacDecoder {
            reader: FlacReader::open(data).expect("failed to open FLAC stream"),
            current_frame,
            current_frame_channel: 0,
            current_frame_sample_pos: 0,
            current_time,
            sample_rate,
            max_block_len,
        }
    }

    pub fn compute_duration(&mut self) -> u64 {
        let samples = self.reader.streaminfo().samples;
        let mut total_duration = samples.unwrap() / self.sample_rate as u64;
        total_duration
    }

    pub fn current_time(&self) -> u32 {
        self.current_time
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn next(&mut self) -> Option<i32> {
        next_sample(self)
    }
}

pub fn next_sample(decoder: &mut FlacDecoder) -> Option<i32> {

    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    // let mut f_reader = decoder.reader.samples();
    // let mut data = Vec::new();
    // if let Some(sample) = f_reader.next() {
    //     if let Ok(sample) = sample {
    //         data.push([sample as i16]);
    //         // return Some(sample);
    //     }
    // }

    let p = Playback::new("FLAC", "Play FLAC", None, decoder.reader.streaminfo().sample_rate);
    println!("RATE: {}", decoder.reader.streaminfo().sample_rate);

    let mut f_reader = decoder.reader.blocks();
    let mut sample_buffer = Vec::with_capacity(decoder.max_block_len);
    loop {
        match f_reader.read_next_or_eof(sample_buffer) {
            Ok(Some(block)) => {

                let mut data = Vec::new();
                for s in block.stereo_samples() {
                    // println!("SAMPLE: {} {}", s.0, s.1);
                    data.push([s.0 as i16, s.1 as i16]);
                }

                p.write(&data[..]);

                sample_buffer = block.into_buffer();

            },
            Ok(None) => break,
            Err(_) => panic!("Failed to decode"),
        }
    }

    None
}