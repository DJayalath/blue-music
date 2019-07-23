use super::Song;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rodio::{Device, Sink};
use std::time::SystemTime;

pub struct Playlist {
    songs: Vec<Song>,
    device: Device,
    sink: Sink,
    start: SystemTime,
}

impl Playlist {
    pub fn new(songs: Vec<Song>) -> Self {
        let device = rodio::default_output_device().unwrap();
        let sink = rodio::Sink::new(&device);
        let start = SystemTime::now();
        Playlist {
            songs,
            device,
            sink,
            start,
        }
    }

    pub fn play_next(&mut self) {
        self.songs.remove(0);
        self.stop_sink();
        self.songs[0].play(&self.sink).unwrap();
        self.reset_time();
    }

    pub fn random_shuffle(&mut self) {
        self.stop_sink();
        self.songs.shuffle(&mut thread_rng());
        self.play_next();
    }

    pub fn update(&mut self) {
        if self.is_song_finished() {
            self.play_next();
        }
    }

    fn reset_time(&mut self) {
        self.start = SystemTime::now();
    }

    fn is_song_finished(&self) -> bool {
        let now = SystemTime::now();
        if now.duration_since(self.start).unwrap().as_secs() > self.songs[0].duration.as_secs() {
            return true;
        }

        false
    }

    fn stop_sink(&mut self) {
        self.sink.stop();
        self.sink = rodio::Sink::new(&self.device);
    }
}
