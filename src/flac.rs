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
use std::convert::TryInto;

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
        let sample_buffer = Vec::with_capacity(max_block_len);
        let current_frame = f_reader.read_next_or_eof(sample_buffer).unwrap().unwrap();

        FlacDecoder {
            reader: FlacReader::open(data).expect("failed to open FLAC stream"),
            current_frame,
            current_frame_channel: 0,
            current_frame_sample_pos: 0,
            current_time: 0,
            sample_rate,
            max_block_len,
        }
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
    let sample_buffer = Vec::with_capacity(decoder.max_block_len); // TODO: Re-use buffer

    let mut data = Vec::new();
    match f_reader.read_next_or_eof(sample_buffer) {
        Ok(Some(block)) => {
            decoder.current_time = (block.time() as u32 * 1000) / decoder.sample_rate;
            // decoder.current_time += (block.duration() * 1000) / decoder.sample_rate;
            for s in block.stereo_samples() {
                data.push([s.0 as i16, s.1 as i16]); // Maybe i16??
            }
        },
        Ok(None) => return None,
        Err(_) => panic!("Failed to decode"),
    }

    Some(data)
}

pub fn skip_to(data: &Path, time: u32, decoder: &mut FlacDecoder) {

    decoder.reader = FlacReader::open(data).expect("failed to open FLAC stream");
    let mut sample_buffer = Vec::with_capacity(decoder.max_block_len);
    let time = ((time * decoder.sample_rate) / 1000) as u64;

    let mut f_reader = decoder.reader.blocks();
    loop {
        match f_reader.read_next_or_eof(sample_buffer) {
            Ok(Some(block)) => {

                let block_time = block.time();
                if block_time >= time {
                    decoder.current_time = (time as u32 * 100) / decoder.sample_rate;
                    break
                } 

                sample_buffer = block.into_buffer();
            },
            Ok(None) => panic!("Skip position out of range!"),
            Err(_) => panic!("Failed to decode"),

        }
    }
}

pub fn compute_duration(data: &Path) -> u64 {

    let reader = FlacReader::open(data).expect("failed to open FLAC stream");
    let sample_rate = reader.streaminfo().sample_rate;
    let samples = reader.streaminfo().samples;
    let total_duration = samples.unwrap() / sample_rate as u64;
    total_duration

}