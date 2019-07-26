use claxon::{FlacReader, frame::{FrameReader, FrameResult, Block}, input::ReadBytes};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::convert::AsRef;
use std::path::Path;
use std::time::Duration;

pub struct FlacDecoder {
    reader: FlacReader<File>,
    current_frame: Block,
    current_frame_channel: usize,
    current_frame_sample_pos: usize,
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
}