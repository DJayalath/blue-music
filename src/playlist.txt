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
        self.songs.remove(0); // FIX skipping first song
        self.stop_sink();
        self.songs[0].play(&self.sink).unwrap();
        self.reset_time();
    }

    pub fn random_shuffle(&mut self) {
        self.stop_sink();
        self.songs.shuffle(&mut thread_rng());
        self.play_next();
    }

    // A function that shuffles non-randomly to try and chain songs
    // with the most matching genres
    pub fn genre_shuffle(&mut self) {
        self.stop_sink();

        // Loop over every song currently after now playing
        for k in 1..self.songs.len() {
            // Track the index of the song with the most
            // genre matches to the song being analysed
            let mut best_score = 0;
            let mut best_i = k;

            for i in k..self.songs.len() {
                let mut score = 0;
                for genre in &self.songs[k - 1].genre {
                    if self.songs[i].genre.contains(&genre) {
                        score += 1;
                    }
                }

                if score > best_score {
                    best_score = score;
                    best_i = i;
                }
            }

            // Insert the most similar song into the spot
            // after the song currently being analysed
            let temp = self.songs.remove(best_i);
            self.songs.insert(k, temp);
        }

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

    // Check if enough time has passed that the
    // playing song has finished
    fn is_song_finished(&self) -> bool {
        let now = SystemTime::now();
        if now.duration_since(self.start).unwrap().as_millis() > self.songs[0].duration.as_millis()
        {
            return true;
        }

        false
    }

    fn stop_sink(&mut self) {
        self.sink.stop();
        self.sink = rodio::Sink::new(&self.device);
    }
}
