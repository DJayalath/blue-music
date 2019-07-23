#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate text_io;
extern crate metaflac;
extern crate rodio;
extern crate walkdir;

use playlist::Playlist;
use rodio::Source;
use song::Song;
use std::env;
use std::error::Error;
use std::io::BufReader;
use std::process;
use std::sync::mpsc;
use std::thread;
use walkdir::WalkDir;

mod playlist;
mod song;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("ERROR: Too few args");
        process::exit(1);
    }

    let dir = WalkDir::new(&args[1]);
    let valid_exts = args[2].split(',').collect::<Vec<&str>>();

    let songs = find_music(dir, &valid_exts).unwrap();
    let mut playlist = Playlist::new(songs);

    playlist.random_shuffle();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
        let cmd: String = read!();
        tx.send(cmd).unwrap();
    });

    loop {
        playlist.update();

        if let Ok(c) = &rx.try_recv() {
            match &c[..] {
                "skip" => playlist.play_next(),
                "shuffle" => playlist.random_shuffle(),
                "genre-shuffle" => playlist.genre_shuffle(),
                _ => (),
            }
        }
    }
}

fn find_music(path: WalkDir, valid_exts: &[&str]) -> Result<Vec<Song>, Box<dyn Error>> {
    let mut songs: Vec<Song> = Vec::new();
    for entry in path {
        let entry = entry?;
        let loc = entry.clone();
        let entry = entry.path();
        if let Some(extension) = entry.extension() {
            if valid_exts.contains(&extension.to_str().unwrap()) {
                let file = std::fs::File::open(entry)?;
                let reader = BufReader::new(file);
                let source = rodio::Decoder::new(reader)?;
                let duration = source.total_duration().unwrap();
                let mut title = String::new();
                let mut artist = String::new();
                let mut genre: Vec<String> = Vec::new();

                match metaflac::Tag::read_from_path(entry) {
                    Ok(tag) => {
                        if let Some(g) = tag.get_vorbis("genre") {
                            for genre_field in g {
                                for genre_type in genre_field.split(&[',', '/', '\\', ';', '-'][..])
                                {
                                    genre.push(genre_type.to_string());
                                }
                            }
                        }
                        if let Some(t) = tag.get_vorbis("title") {
                            title = t[0].clone();
                        }
                        if let Some(a) = tag.get_vorbis("artist") {
                            artist = a[0].clone();
                        }

                        songs.push(Song::new(loc, title, artist, genre, duration));
                    }

                    Err(e) => panic!("{}", e),
                }
            }
        }
    }

    Ok(songs)
}
