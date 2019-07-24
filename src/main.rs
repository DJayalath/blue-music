#[macro_use]
extern crate text_io;
extern crate metaflac;
extern crate rodio;
extern crate walkdir;

extern crate gdk_pixbuf;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use relm::{Relm, Update, Widget};
use gtk::prelude::*;
use gtk::Orientation::{Vertical, Horizontal};
use gtk::Image;
use gtk::{OrientableExt, ToolButtonExt};
use gtk::{GtkWindowExt, Inhibit, WidgetExt};
use gdk_pixbuf::Pixbuf;
use gtk::{Adjustment, BoxExt, ImageExt, LabelExt, ScaleExt};
use relm_derive::widget;

pub const PAUSE_ICON: &str = "gtk-media-pause";
pub const PLAY_ICON: &str = "gtk-media-play";

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

    Win::run(()).unwrap();

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


#[derive(Msg)]
pub enum Msg {
    Open,
    PlayPause,
    Previous,
    Stop,
    Next,
    Remove,
    Save,
    Quit,
}

pub struct Model {
    adjustment: Adjustment,
    cover_pixbuf: Option<Pixbuf>,
    cover_visible: bool,
    current_duration: u64,
    current_time: u64,
    play_image: Image,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            adjustment: Adjustment::new(
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            ),
            cover_pixbuf: None,
            cover_visible: false,
            current_duration: 0,
            current_time: 0,
            play_image: new_icon(PLAY_ICON),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            // A call to self.label1.set_text() is automatically inserted by the
            // attribute every time the model.counter attribute is updated.
            Msg::Open => (),
            Msg::PlayPause => (),
            Msg::Previous => (),
            Msg::Stop => (),
            Msg::Next => (),
            Msg::Remove => (),
            Msg::Save => (),
            Msg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            title: "Blue Music",
            gtk::Box {
                orientation: Vertical,
                #[name="toolbar"]
                gtk::Toolbar {
                    gtk::ToolButton {
                        icon_widget: &new_icon("document-open"),
                        clicked => Msg::Open,
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("document-save"),
                        clicked => Msg::Save,
                    },
                    gtk::SeparatorToolItem {
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("gtk-media-previous"),
                        clicked => Msg::Previous,
                    },
                    gtk::ToolButton {
                        icon_widget: &self.model.play_image,
                        clicked => Msg::PlayPause,
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("gtk-media-stop"),
                        clicked => Msg::Stop,
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("gtk-media-next"),
                        clicked => Msg::Next,
                    },
                    gtk::SeparatorToolItem {
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("remove"),
                        clicked => Msg::Remove,
                    },
                    gtk::SeparatorToolItem {
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("gtk-quit"),
                        clicked => Msg::Quit,
                    },
                },
                gtk::Image {
                    from_pixbuf: self.model.cover_pixbuf.as_ref(),
                    visible: self.model.cover_visible,
                },
                gtk::Box {
                    orientation: Horizontal,
                    spacing: 10,
                    gtk::Scale(Horizontal, &self.model.adjustment) {
                        draw_value: false,
                        hexpand: true,
                    },
                    gtk::Label {
                        text: &millis_to_minutes(self.model.current_time),
                    },
                    gtk::Label {
                        text: "/",
                    },
                    gtk::Label {
                        // TODO: margin_right: 10,
                        text: &millis_to_minutes(self.model.current_duration),
                    },
                }
            },
            // Use a tuple when you want to both send a message and return a value to
            // the GTK+ callback.
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }

    fn init_view(&mut self) {
        self.toolbar.show_all();
    }
}

fn millis_to_minutes(millis: u64) -> String {
    let mut seconds = millis / 1_000;
    let minutes = seconds / 60;
    seconds %= 60;
    format!("{}:{:02}", minutes, seconds)
}

fn new_icon(icon: &str) -> Image {
    Image::new_from_file(
        format!("./assets/{}.png", icon)
    )
}