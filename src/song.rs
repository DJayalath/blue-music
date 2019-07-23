use rodio::Decoder;
use std::io::BufReader;
use std::fs::File;
use std::error::Error;
use walkdir::DirEntry;

type RSource = Decoder<BufReader<File>>;

pub struct Song {
    pub loc: DirEntry,
    pub title: String,
    pub artist: String,
    pub genre: Vec<String>,
    pub duration: u64,
}

impl Song {
    
    pub fn new(loc: DirEntry, title: String, artist: String, genre: Vec<String>, duration: u64) -> Self {
        Song { loc, title, artist, genre, duration }
    }

    pub fn play(&self, sink: &rodio::Sink) -> Result<(), Box<dyn Error>> {

        let file = std::fs::File::open(self.loc.path())?;
        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)?;
        sink.append(source);
        println!("Now playing: {} by {}", self.title, self.artist);
        if !self.genre.is_empty() {
            println!("Genre: {:?}", self.genre);
        }
        println!("Duration: {} mins {} secs", self.duration / 60, self.duration % 60);

        Ok(())
    }
}