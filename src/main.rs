#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate text_io;
extern crate walkdir;
extern crate rodio;
extern crate metaflac;

use std::process;
use std::error::Error;
use std::env;
use std::io::BufReader;
use walkdir::{DirEntry, WalkDir};
use rodio::Source;
use song::Song;

mod song;

fn main() {

    let id3_genres = vec![
     "Blues",
     "Hip Hop",
     "Classic Rock",
     "Country",
     "Dance",
     "Disco",
     "Funk",
     "Grunge",
     "Hip-Hop",
     "Jazz",
     "Metal",
     "New Age",
     "Oldies",
     "Other",
     "Pop",
     "R&B",
     "Rap",
     "Reggae",
     "Rock",
     "Techno",
     "Industrial",
     "Alternative",
     "Ska",
     "Death Metal",
     "Pranks",
     "Soundtrack",
     "Euro-Techno",
     "Ambient",
     "Trip-Hop",
     "Vocal",
     "Jazz+Funk",
     "Fusion",
     "Trance",
     "Classical",
     "Instrumental",
     "Acid",
     "House",
     "Game",
     "Sound Clip",
     "Gospel",
     "Noise",
     "AlternRock",
     "Bass",
     "Soul",
     "Punk",
     "Space",
     "Meditative",
     "Instrumental Pop",
     "Instrumental Rock",
     "Ethnic",
     "Gothic",
     "Darkwave",
     "Techno-Industrial",
     "Electronic",
     "Pop-Folk",
     "Eurodance",
     "Dream",
     "Southern Rock",
     "Comedy",
     "Cult",
     "Gangsta",
     "Top 40",
     "Christian Rap",
     "Pop/Funk",
     "Jungle",
     "Native American",
     "Cabaret",
     "New Wave",
     "Psychedelic",
     "Rave",
     "Showtunes",
     "Trailer",
     "Lo-Fi",
     "Tribal",
     "Acid Punk",
     "Acid Jazz",
     "Polka",
     "Retro",
     "Musical",
     "Rock & Roll",
     "Hard Rock"];

    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("ERROR: Too few args");
        process::exit(1);
    }

    let dir = WalkDir::new(&args[1]);
    let valid_exts = args[2].split(',').collect::<Vec<&str>>();
    let device = rodio::default_output_device().unwrap();
    let mut sink = rodio::Sink::new(&device);

    let mut songs = find_music(dir, &valid_exts).unwrap();

    match songs[0].play(&sink) {
        Ok(()) => {
            songs.remove(0);
        }

        Err(e) => panic!("{}", e)
    }

    loop {
        let cmd: String = read!();

        match &cmd[..] {
            "skip" => {
                sink.stop();
                sink = rodio::Sink::new(&device);
                match songs[0].play(&sink) {
                    Ok(()) => {
                        songs.remove(0);
                    }

                    Err(e) => panic!("{}", e)
                };
            },
            _ => (),
        }
    }

    sink.sleep_until_end();

    println!("Hello, world!");
}

fn find_music(path: WalkDir, valid_exts: &Vec<&str>) -> Result<Vec<Song>, Box<dyn Error>> {

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
                let duration = source.total_duration().unwrap().as_secs();
                let mut title = String::new();
                let mut artist = String::new();
                let mut genre: Vec<String> = Vec::new();

                match metaflac::Tag::read_from_path(entry) {
                    Ok(tag) => {
                        if let Some(g) = tag.get_vorbis("genre") {
                            
                            for genre_field in g {
                                for genre_type in genre_field.split(&[',', '/', '\\', ';', '-'][..]) {
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

                        songs.push(Song { loc, title, artist, genre, duration });
                    },

                    Err(e) => panic!("{}", e),
                }
            }
        }
    }

    Ok(songs)
}
