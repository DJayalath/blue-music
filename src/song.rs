use rodio::Sink;
use std::error::Error;
use std::io::BufReader;
use std::time::Duration;
use walkdir::DirEntry;

pub struct Song {
    pub loc: DirEntry,
    pub title: String,
    pub artist: String,
    pub genre: Vec<String>,
    pub duration: Duration,
}

impl Song {
    pub fn new(
        loc: DirEntry,
        title: String,
        artist: String,
        genre: Vec<String>,
        duration: Duration,
    ) -> Self {
        Song {
            loc,
            title,
            artist,
            genre,
            duration,
        }
    }

    fn add_to_sink(&self, sink: &Sink) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::open(self.loc.path())?;
        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)?;
        sink.append(source);

        Ok(())
    }

    pub fn play(&self, sink: &Sink) -> Result<(), Box<dyn Error>> {
        self.add_to_sink(sink).unwrap();
        println!("Now playing: {} by {}", self.title, self.artist);
        if !self.genre.is_empty() {
            println!("Genre: {:?}", self.genre);
        }
        println!(
            "Duration: {} mins {} secs",
            self.duration.as_secs() / 60,
            self.duration.as_secs() % 60
        );

        Ok(())
    }
}
