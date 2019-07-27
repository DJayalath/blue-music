use claxon::FlacReader;
use std::fs::File;
use std::path::Path;

pub struct FlacDecoder {
    reader: FlacReader<File>,
    current_time: u32,
    sample_rate: u32,
    sample_time: f64,
    max_block_len: usize,
}

impl FlacDecoder {

    pub fn new(data: &Path) -> Self {

        let reader = FlacReader::open(data).expect("failed to open FLAC stream");
        let num_channels = reader.streaminfo().channels;
        let sample_rate = reader.streaminfo().sample_rate;
        let max_block_len = reader.streaminfo().max_block_size as usize * num_channels as usize;
        let sample_time = 1000.0 / sample_rate as f64;

        FlacDecoder {
            reader: FlacReader::open(data).expect("failed to open FLAC stream"),
            current_time: 0,
            sample_rate,
            sample_time,
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

    let mut s_reader = decoder.reader.samples();
    let mut data = Vec::new();

    let mut count = 0;
    for _ in 0..decoder.max_block_len {
        if let Some(s1) = s_reader.next(){
            if let Ok(s1) = s1 {
                if let Some(s2) = s_reader.next() {
                    if let Ok(s2) = s2 {
                        count += 1;
                        data.push([s1 as i16, s2 as i16]);
                    }
                }
            }
        }
    }

    decoder.current_time += (decoder.sample_time * count as f64) as u32;

    Some(data)

    // let mut f_reader = decoder.reader.blocks();
    // let sample_buffer = Vec::with_capacity(decoder.max_block_len); // TODO: Re-use buffer

    // let mut data = Vec::new();
    // match f_reader.read_next_or_eof(sample_buffer) {
    //     Ok(Some(block)) => {
    //         decoder.current_time = (block.time() as u32 * 1000) / decoder.sample_rate;
    //         // decoder.current_time += (block.duration() * 1000) / decoder.sample_rate;
    //         for s in block.stereo_samples() {
    //             println!("{} {}", s.0, s.1);
    //             data.push([s.0 as i16, s.1 as i16]); // Maybe i16??
    //         }
    //     },
    //     Ok(None) => return None,
    //     Err(_) => panic!("Failed to decode"),
    // }

    // Some(data)
}

pub fn skip_to(data: &Path, time: u32, decoder: &mut FlacDecoder) {

    decoder.reader = FlacReader::open(data).expect("failed to open FLAC stream");
    let n = (time / 1000) * decoder.sample_rate;
    let mut s_reader = decoder.reader.samples();
    s_reader.nth(n as usize * 2);
    decoder.current_time = time;

    // decoder.reader = FlacReader::open(data).expect("failed to open FLAC stream");
    // let mut sample_buffer = Vec::with_capacity(decoder.max_block_len);
    // let time = ((time * decoder.sample_rate) / 1000) as u64;

    // let mut f_reader = decoder.reader.blocks();
    // loop {
    //     match f_reader.read_next_or_eof(sample_buffer) {
    //         Ok(Some(block)) => {

    //             let block_time = block.time();
    //             if block_time >= time {
    //                 decoder.current_time = (time as u32 * 100) / decoder.sample_rate;
    //                 break
    //             } 

    //             sample_buffer = block.into_buffer();
    //         },
    //         Ok(None) => panic!("Skip position out of range!"),
    //         Err(_) => panic!("Failed to decode"),

    //     }
    // }
}

pub fn compute_duration(data: &Path) -> u64 {

    let reader = FlacReader::open(data).expect("failed to open FLAC stream");
    let sample_rate = reader.streaminfo().sample_rate;
    let samples = reader.streaminfo().samples;
    let total_duration = samples.unwrap() / sample_rate as u64;
    total_duration

}