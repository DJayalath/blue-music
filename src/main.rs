#[macro_use]
extern crate text_io;
#[macro_use]
extern crate lazy_static;
extern crate metaflac;
extern crate rodio;
extern crate walkdir;

extern crate gdk_pixbuf;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use gtk::{
    Adjustment, AdjustmentExt, BoxExt, ButtonsType, DialogExt, DialogFlags, FileChooserAction,
    FileChooserDialog, FileChooserExt, FileFilter, GtkWindowExt, Image, ImageExt, Inhibit,
    LabelExt, MessageDialog, MessageType, OrientableExt, ScaleExt, ToolButtonExt, WidgetExt,
    Window,
};
use relm::{interval, Relm, Update, Widget};
use std::path::PathBuf;
// use gtk::prelude::*;
use gtk::Orientation::{Horizontal, Vertical};
// use gtk::Image;
// use gtk::{OrientableExt, ToolButtonExt};
// use gtk::{GtkWindowExt, Inhibit, WidgetExt};
use gdk_pixbuf::Pixbuf;
use playlist::Msg::{
    AddSong, LoadSong, NextSong, PauseSong, PlaySong, PreviousSong, RemoveSong, SaveSong,
    SongDuration, SongStarted, StopSong, UpdateTime,
};
use playlist::Playlist;
use relm_derive::widget;

use gtk_sys::{GTK_RESPONSE_ACCEPT, GTK_RESPONSE_CANCEL};
pub const PAUSE_ICON: &str = "gtk-media-pause";
pub const PLAY_ICON: &str = "gtk-media-play";

// use playlist::Playlist;
// use song::Song;
use rodio::Source;
use std::env;
use std::error::Error;
use std::io::BufReader;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use walkdir::WalkDir;

// mod player;
mod player;
mod playlist;

// mod playlist;
// mod song;

fn main() {
    // // TEMPORARY CMD-LINE ARGS BEFORE GUI
    // // Read cmd-line arguments and check errors
    // let args: Vec<String> = env::args().collect();
    // if args.len() <= 2 {
    //     eprintln!("ERROR: Too few args");
    //     process::exit(1);
    // }
    // let dir = WalkDir::new(&args[1]);
    // let valid_exts = args[2].split(',').collect::<Vec<&str>>();

    // // Create playlist from songs located in directory root
    // let songs = find_music(dir, &valid_exts).unwrap();
    // let mut playlist = Playlist::new(songs);

    Win::run(()).unwrap();

    // // Thread send/receive channels to communicate cmd-line
    // // commands to control playlist
    // let (tx, rx) = mpsc::channel();

    // playlist.random_shuffle();

    // // Cmd-line command collection thread
    // thread::spawn(move || loop {
    //     let cmd: String = read!();
    //     tx.send(cmd).unwrap();
    // });

    // // Playlist runs on main thread
    // loop {
    //     playlist.update();

    //     // Ensure main thread doesn't wait for new
    //     // commands if none are provided
    //     if let Ok(c) = &rx.try_recv() {
    //         match &c[..] {
    //             "skip" => playlist.play_next(),
    //             "shuffle" => playlist.random_shuffle(),
    //             "genre-shuffle" => playlist.genre_shuffle(),
    //             _ => (),
    //         }
    //     }
    // }
}

// fn find_music(path: WalkDir, valid_exts: &[&str]) -> Result<Vec<Song>, Box<dyn Error>> {
//     let mut songs: Vec<Song> = Vec::new();
//     for entry in path {
//         let entry = entry?;
//         let loc = entry.clone();
//         let entry = entry.path();

//         // Check extension exists!
//         if let Some(extension) = entry.extension() {
//             // Make certain it is an extension that can be played
//             if valid_exts.contains(&extension.to_str().unwrap()) {
//                 // Find relevant Song attributes
//                 let file = std::fs::File::open(entry)?;
//                 let reader = BufReader::new(file);
//                 let source = rodio::Decoder::new(reader)?;
//                 let duration = source.total_duration().unwrap();
//                 let mut title = String::new();
//                 let mut artist = String::new();
//                 let mut genre: Vec<String> = Vec::new();

//                 match metaflac::Tag::read_from_path(entry) {
//                     Ok(tag) => {
//                         // The genre tag requires more work by splitting strings
//                         // using certain characters because genre tagging is
//                         // inconsistent. Some tags are written as a single string
//                         // with delimiters. Others are done properly as separate
//                         // strings in vorbis.
//                         if let Some(g) = tag.get_vorbis("genre") {
//                             for genre_field in g {
//                                 // Note this is not ideal because the standard tag 'Hip Hop', if
//                                 // tagged as 'Hip-Hop' is split into 'Hip' and 'Hop'
//                                 for genre_type in genre_field.split(&[',', '/', '\\', ';', '-'][..])
//                                 {
//                                     genre.push(genre_type.to_string());
//                                 }
//                             }
//                         }

//                         // Title and artist tags are just one string so
//                         // only the first index of the vector is needed

//                         if let Some(t) = tag.get_vorbis("title") {
//                             title = t[0].clone();
//                         }

//                         if let Some(a) = tag.get_vorbis("artist") {
//                             artist = a[0].clone();
//                         }

//                         songs.push(Song::new(loc, title, artist, genre, duration));
//                     }

//                     Err(e) => panic!("{}", e),
//                 }
//             }
//         }
//     }

//     Ok(songs)
// }

#[derive(Msg)]
pub enum Msg {
    Open,
    PlayPause,
    Previous,
    Stop,
    Next,
    Remove,
    Save,
    Started(Option<Pixbuf>),
    Quit,
    Update(u128),
    Duration(u128),
    Tick,
}

pub struct Model {
    adjustment: Adjustment,
    cover_pixbuf: Option<Pixbuf>,
    cover_visible: bool,
    current_duration: u128,
    current_time: u128,
    play_image: Image,
    stopped: bool,
    paused: bool,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            adjustment: Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            cover_pixbuf: None,
            cover_visible: false,
            current_duration: 0,
            current_time: 0,
            play_image: new_icon(PLAY_ICON),
            stopped: true,
            paused: false,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 1000, || Msg::Tick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Tick => {
                if !self.model.paused && !self.model.stopped {
                    self.set_current_time(self.model.current_time + 1000);
                    self.elapsed
                        .set_text(&format!("{}", millis_to_minutes(self.model.current_time)));

                    if self.model.current_time > self.model.current_duration {
                        self.set_current_time(0);
                        self.model.current_duration = 0;
                        self.playlist.emit(StopSong);
                        self.model.cover_visible = false;
                        self.set_play_icon(PLAY_ICON);
                        self.model.stopped = true;
                        self.model.paused = false;
                    }
                }
            }
            // A call to self.label1.set_text() is automatically inserted by the
            // attribute every time the model.counter attribute is updated.
            Msg::Open => self.open(),
            Msg::PlayPause => {
                if self.model.stopped {
                    self.model.paused = false;
                    self.playlist.emit(PlaySong);
                    self.model.stopped = false;
                } else {
                    self.model.paused = true;
                    self.playlist.emit(PauseSong);
                    self.set_play_icon(PLAY_ICON);
                    self.model.stopped = true;
                }
            }
            Msg::Update(t) => self.set_current_time(t),
            Msg::Previous => (),
            Msg::Stop => {
                self.set_current_time(0);
                self.model.current_duration = 0;
                self.playlist.emit(StopSong);
                self.model.cover_visible = false;
                self.set_play_icon(PLAY_ICON);
                self.model.stopped = true;
                self.model.paused = false;
            }
            Msg::Next => (),
            Msg::Remove => (),
            Msg::Save => {
                let file = show_save_dialog(&self.window);
                if let Some(file) = file {
                    self.playlist.emit(SaveSong(file));
                }
            }
            Msg::Started(pixbuf) => {
                self.set_play_icon(PAUSE_ICON);
                self.model.cover_visible = true;
                self.model.cover_pixbuf = pixbuf;
                self.model.stopped = false;
            }
            Msg::Duration(duration) => {
                self.model.current_duration = duration;
                self.model.adjustment.set_upper(duration as f64);
            }
            Msg::Quit => gtk::main_quit(),
        }
    }

    fn init_view(&mut self) {
        self.toolbar.show_all();
    }

    fn set_current_time(&mut self, time: u128) {
        self.model.current_time = time;
        self.model.adjustment.set_value(time as f64);
    }

    fn set_play_icon(&self, icon: &str) {
        self.model
            .play_image
            .set_from_file(format!("assets/{}.png", icon));
    }

    view! {
        #[name="window"]
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
                        clicked => playlist@PreviousSong,
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
                        clicked => playlist@NextSong,
                    },
                    gtk::SeparatorToolItem {
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("remove"),
                        clicked => playlist@RemoveSong,
                    },
                    gtk::SeparatorToolItem {
                    },
                    gtk::ToolButton {
                        icon_widget: &new_icon("gtk-quit"),
                        clicked => Msg::Quit,
                    },
                },
                #[name="playlist"]
                Playlist {
                    SongStarted(ref pixbuf) => Msg::Started(pixbuf.clone()),
                    SongDuration(duration) => Msg::Duration(duration),
                    UpdateTime(t) => Msg::Update(t),
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
                    #[name="elapsed"]
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
}

impl Win {
    fn open(&self) {
        let file = show_open_dialog(&self.window);
        if let Some(file) = file {
            let ext = file
                .extension()
                .map(|ext| ext.to_str().unwrap().to_string());
            if let Some(ext) = ext {
                match ext.as_str() {
                    "flac" => self.playlist.emit(AddSong(file)),
                    "mp3" => (),
                    "m3u" => (),
                    extension => {
                        let dialog = MessageDialog::new(
                            Some(&self.window),
                            DialogFlags::empty(),
                            MessageType::Error,
                            ButtonsType::Ok,
                            &format!("Cannot open file with extension .{}", extension),
                        );
                        dialog.run();
                        dialog.destroy();
                    }
                }
            }
        }
    }
}

fn millis_to_minutes(millis: u128) -> String {
    let mut seconds = millis / 1_000;
    let minutes = seconds / 60;
    seconds %= 60;
    format!("{}:{:02}", minutes, seconds)
}

fn new_icon(icon: &str) -> Image {
    Image::new_from_file(format!("./assets/{}.png", icon))
}

fn show_open_dialog(parent: &Window) -> Option<PathBuf> {
    let mut file = None;
    let dialog = FileChooserDialog::new(
        Some("Select a FLAC audio file"),
        Some(parent),
        FileChooserAction::Open,
    );

    let flac_filter = FileFilter::new();
    flac_filter.add_mime_type("audio/flac");
    flac_filter.set_name("FLAC audio file");
    dialog.add_filter(&flac_filter);

    let m3u_filter = FileFilter::new();
    m3u_filter.add_mime_type("audio/x-mpegurl");
    m3u_filter.set_name("M3U playlist file");
    dialog.add_filter(&m3u_filter);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Accept", gtk::ResponseType::Accept);
    let result = dialog.run();
    if result == GTK_RESPONSE_ACCEPT {
        file = dialog.get_filename();
    }
    dialog.destroy();
    file
}

fn show_save_dialog(parent: &Window) -> Option<PathBuf> {
    let mut file = None;
    let dialog = FileChooserDialog::new(
        Some("Choose a destination M3U playlist file"),
        Some(parent),
        FileChooserAction::Save,
    );
    let filter = FileFilter::new();
    filter.add_mime_type("audio/x-mpegurl");
    filter.set_name("M3U playlist file");
    dialog.set_do_overwrite_confirmation(true);
    dialog.add_filter(&filter);
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Accept);
    let result = dialog.run();
    if result == GTK_RESPONSE_ACCEPT {
        file = dialog.get_filename();
    }
    dialog.destroy();
    file
}
