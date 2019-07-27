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
}

pub fn next_sample(decoder: &mut FlacDecoder) -> Option<Vec<[i16; 2]>> {

    let mut f_reader = decoder.reader.blocks();
    let sample_buffer = Vec::with_capacity(decoder.max_block_len);

    let mut data = Vec::new();
    match f_reader.read_next_or_eof(sample_buffer) {
        Ok(Some(block)) => {

            for s in block.stereo_samples() {
                data.push([s.0 as i16, s.1 as i16]); // Maybe i16??
            }
        },
        Ok(None) => return None,
        Err(_) => panic!("Failed to decode"),
    }

    Some(data)
}

pub fn compute_duration(data: &Path) -> u64 {

    let mut reader = FlacReader::open(data).expect("failed to open FLAC stream");
    let sample_rate = reader.streaminfo().sample_rate;
    let samples = reader.streaminfo().samples;
    let mut total_duration = samples.unwrap() / sample_rate as u64;
    total_duration

}