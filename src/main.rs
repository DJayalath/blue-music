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
    // TEMPORARY CMD-LINE ARGS BEFORE GUI
    // Read cmd-line arguments and check errors
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        eprintln!("ERROR: Too few args");
        process::exit(1);
    }
    let dir = WalkDir::new(&args[1]);
    let valid_exts = args[2].split(',').collect::<Vec<&str>>();

    // Create playlist from songs located in directory root
    let songs = find_music(dir, &valid_exts).unwrap();
    let mut playlist = Playlist::new(songs);

    // Thread send/receive channels to communicate cmd-line
    // commands to control playlist
    let (tx, rx) = mpsc::channel();

    playlist.random_shuffle();

    // Cmd-line command collection thread
    thread::spawn(move || loop {
        let cmd: String = read!();
        tx.send(cmd).unwrap();
    });

    // Playlist runs on main thread
    loop {
        playlist.update();

        // Ensure main thread doesn't wait for new
        // commands if none are provided
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

        // Check extension exists!
        if let Some(extension) = entry.extension() {
            // Make certain it is an extension that can be played
            if valid_exts.contains(&extension.to_str().unwrap()) {
                // Find relevant Song attributes
                let file = std::fs::File::open(entry)?;
                let reader = BufReader::new(file);
                let source = rodio::Decoder::new(reader)?;
                let duration = source.total_duration().unwrap();
                let mut title = String::new();
                let mut artist = String::new();
                let mut genre: Vec<String> = Vec::new();

                match metaflac::Tag::read_from_path(entry) {
                    Ok(tag) => {
                        // The genre tag requires more work by splitting strings
                        // using certain characters because genre tagging is
                        // inconsistent. Some tags are written as a single string
                        // with delimiters. Others are done properly as separate
                        // strings in vorbis.
                        if let Some(g) = tag.get_vorbis("genre") {
                            for genre_field in g {
                                // Note this is not ideal because the standard tag 'Hip Hop', if
                                // tagged as 'Hip-Hop' is split into 'Hip' and 'Hop'
                                for genre_type in genre_field.split(&[',', '/', '\\', ';', '-'][..])
                                {
                                    genre.push(genre_type.to_string());
                                }
                            }
                        }

                        // Title and artist tags are just one string so
                        // only the first index of the vector is needed

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
