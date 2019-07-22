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
use std::ffi::OsString;
use walkdir::{DirEntry, WalkDir};
use rodio::Source;

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

    let (mut mu, mut me) = find_music(dir, &valid_exts).unwrap();

    let (mut unplayed, mut meta) = sort_music_by_mood(&mut mu, &mut me);

    match play_song(&sink, &unplayed[0], &meta[0]) {
        Ok(d) => {
            unplayed.remove(0);
            meta.remove(0);
        }

        Err(e) => panic!("{}", e)
    }

    loop {
        let cmd: String = read!();

        match &cmd[..] {
            "skip" => {
                sink.stop();
                sink = rodio::Sink::new(&device);
                let duration = match play_song(&sink, &unplayed[0], &meta[0]) {
                    Ok(d) => {
                        unplayed.remove(0);
                        meta.remove(0);
                        d
                    }

                    Err(e) => panic!("{}", e)
                };
            },
            _ => (),
        }
    }

    // let mut scores: Vec<i32> = vec![0; unplayed.len()];
    // for i in 0..unplayed.len() {
    //     if !meta[i].2.is_empty() {
    //         if meta[i].2[0].contains("Rock") {
    //             scores[i] += 1;
    //         }
    //     }
    // }

    // let mut i = 0;
    // for (j, &value) in scores.iter().enumerate() {
    //     if value > scores[i] {
    //         i = j;
    //     }
    // }

    // println!("Now playing: {} by {}", meta.get(i).unwrap().0, meta.get(i).unwrap().1);
    // if !meta.get(i).unwrap().2.is_empty() {
    //     println!("Genre: {}", meta.get(i).unwrap().2[0]);
    // }
    // unplayed.remove(i);
    // meta.remove(i);



    sink.sleep_until_end();

    println!("Hello, world!");
}

fn sort_music_by_mood(music: &mut Vec<DirEntry>, meta: &mut Vec<(String, String, Vec<String>)>) -> (Vec<DirEntry>, Vec<(String, String, Vec<String>)>) {

    let mut sorted_music = Vec::new();
    let mut sorted_meta = Vec::new();
    sorted_music.push(music[0].clone());
    sorted_meta.push(meta[0].clone());

    loop {

        if music.len() <= 1 {
            sorted_music.push(music[0].clone());
            sorted_meta.push(meta[0].clone());
            break
        }

        let mut best_index = 1;
        let mut best_score = 0;
        for j in 1..music.len() {
            let sc = score(&meta[0].2, &meta[j].2);
            if sc > best_score {
                best_score = sc.clone();
                best_index = j.clone();
            }
        }

        println!("{}", best_index);

        sorted_music.push(music[best_index].clone());
        sorted_meta.push(meta[best_index].clone());

        music.remove(best_index);
        meta.remove(best_index);
    }

    (sorted_music, sorted_meta)
}

fn score(genres_of_current: &Vec<String>, genres_of_next: &Vec<String>) -> i32 {

    let mut score = 0;
    for genre in genres_of_current {
        for gen in genres_of_next {
            if genre.contains(gen) {
                score += 1;
            }
        }
    }

    score
}

fn play_song(sink: &rodio::Sink, dir: &DirEntry, meta: &(String, String, Vec<String>)) -> Result<u64, Box<dyn Error>> {

    let file = std::fs::File::open(dir.path())?;
    let source = rodio::Decoder::new(BufReader::new(file))?;
    let duration = source.total_duration().unwrap().as_secs();
    sink.append(source);
    println!("Now playing: {} by {}", meta.0, meta.1);
    if !meta.2.is_empty() {
        println!("Genre: {}", meta.2[0]);
    }
    println!("Duration: {} mins {} secs", duration / 60, duration % 60);

    Ok(duration)
}

fn find_music(path: WalkDir, valid_exts: &Vec<&str>) -> Result<(Vec<DirEntry>, Vec<(String, String, Vec<String>)>), Box<dyn Error>> {
    let mut files = Vec::new();
    let mut TAG = Vec::new(); // Title Artist Genre NOT tag
    for entry in path {
        let dir_entry = entry?;
        let p = dir_entry.path();
        if let Some(extension) = p.extension() {
            if valid_exts.contains(&extension.to_str().unwrap()) {
                files.push(dir_entry.clone());

                match metaflac::Tag::read_from_path(p) {
                    Ok(tag) => {
                        let mut title = String::new();
                        let mut artist = String::new();
                        let mut genre: Vec<String> = Vec::new();
                        match tag.get_vorbis("genre") {
                            Some(g) => {
                                genre = g.clone();
                            }
                            None => (),
                        };
                        match tag.get_vorbis("title") {
                            Some(g) => {
                                title = g[0].clone();
                            }
                            None => (),
                        };
                        match tag.get_vorbis("artist") {
                            Some(g) => {
                                artist = g[0].clone();
                            }
                            None => (),
                        };

                        TAG.push((title, artist, genre));
                    }

                    Err(e) => panic!("{}", e)
                }
            }
        }
    }

    Ok((files, TAG))
}
