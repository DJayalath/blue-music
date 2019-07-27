#[macro_use]
extern crate lazy_static;
extern crate metaflac;
extern crate rodio;
extern crate gdk_pixbuf;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate walkdir;
extern crate crossbeam;

use gtk::{
    BoxExt, ButtonsType, DialogExt, DialogFlags, FileChooserAction,
    FileChooserDialog, FileChooserExt, FileFilter, GtkWindowExt, Image, ImageExt, Inhibit,
    LabelExt, MessageDialog, MessageType, OrientableExt, ScaleExt, ToolButtonExt, WidgetExt,
    Window, Adjustment, AdjustmentExt, Range, RangeExt,
};
use relm::{interval, Relm, Update, Widget};
use std::path::PathBuf;
use gtk::Orientation::{Horizontal, Vertical};
use gdk_pixbuf::Pixbuf;
use playlist::Msg::{
    AddSong, LoadSong, NextSong, PauseSong, PlaySong, PreviousSong, RemoveSong, SaveSong,
    SongDuration, SongStarted, StopSong, PlayerMsgRecv,
};
use playlist::Playlist;
use relm_derive::widget;
use walkdir::{DirEntry, WalkDir};
use std::ffi::OsStr;
use pulse_simple::Playback;
use playlist::PlayerMsg;

use gtk_sys::{GTK_RESPONSE_ACCEPT, GTK_RESPONSE_CANCEL};
pub const PAUSE_ICON: &str = "gtk-media-pause";
pub const PLAY_ICON: &str = "gtk-media-play";

mod player;
mod playlist;
mod flac;

fn main() {


    let mut dec = flac::FlacDecoder::new(&std::path::Path::new("/home/hans/Music/Africa - Toto.flac"));
    flac::next_sample(&mut dec);

    Win::run(()).unwrap();

}

#[derive(Msg)]
pub enum Msg {
    Open,
    PlayPause,
    Previous,
    Stop,
    MsgRecv(PlayerMsg),
    Next,
    Remove,
    Save,
    Started(Option<Pixbuf>),
    Quit,
    Duration(u64),
    Changed,
}

pub struct Model {
    adjustment: Adjustment,
    cover_pixbuf: Option<Pixbuf>,
    cover_visible: bool,
    current_duration: u64,
    current_time: u64,
    play_image: Image,
    stopped: bool,
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
        }
    }

    // fn subscriptions(&mut self, relm: &Relm<Self>) {
    //     interval(relm.stream(), 1000, || Msg::Tick);
    // }

    fn player_message(&mut self, player_msg: PlayerMsg) {
        match player_msg {
            playlist::PlayerMsg::PlayerPlay => {
                self.model.stopped = false;
                self.set_play_icon(PAUSE_ICON);
            },
            playlist::PlayerMsg::PlayerStop => {
                self.set_play_icon(PLAY_ICON);
                self.model.stopped = true;
            },
            playlist::PlayerMsg::PlayerTime(time) => self.set_current_time(time),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::MsgRecv(player_msg) => self.player_message(player_msg),
            Msg::Changed => {
                println!("{}", self.model.adjustment.get_value());
            },
            Msg::Open => self.open(),
            Msg::PlayPause => {
                if self.model.stopped {
                    self.playlist.emit(PlaySong);
                } else {
                    self.playlist.emit(PauseSong);
                    self.set_play_icon(PLAY_ICON);
                }
            },
            Msg::Previous => self.playlist.emit(PreviousSong),
            Msg::Stop => {
                self.set_current_time(0);
                self.model.current_duration = 0;
                self.playlist.emit(StopSong);
                self.model.cover_visible = false;
                self.set_play_icon(PLAY_ICON);
            },
            Msg::Next => self.playlist.emit(NextSong),
            Msg::Remove => self.playlist.emit(RemoveSong),
            Msg::Save => {
                let file = show_save_dialog(&self.window);
                if let Some(file) = file {
                    self.playlist.emit(SaveSong(file));
                }
            },
            Msg::Started(pixbuf) => {
                self.set_play_icon(PAUSE_ICON);
                self.model.cover_visible = true;
                self.model.cover_pixbuf = pixbuf;
            },
            Msg::Duration(duration) => {
                self.model.current_duration = duration;
                self.model.adjustment.set_upper(duration as f64);
            },
            Msg::Quit => gtk::main_quit(),
        }
    }

    fn init_view(&mut self) {
        self.toolbar.show_all();
    }

    fn set_current_time(&mut self, time: u64) {
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
                    PlayerMsgRecv(ref player_msg) => Msg::MsgRecv(player_msg.clone()),
                    SongStarted(ref pixbuf) => Msg::Started(pixbuf.clone()),
                    SongDuration(duration) => Msg::Duration(duration),
                },
                gtk::Image {
                    from_pixbuf: self.model.cover_pixbuf.as_ref(),
                    visible: self.model.cover_visible,
                },
                gtk::Box {
                    orientation: Horizontal,
                    spacing: 10,
                    #[name="timing_scale"]
                    gtk::Scale {
                        orientation: Horizontal,
                        adjustment: &self.model.adjustment,
                        draw_value: false,
                        hexpand: true,
                        value_changed => Msg::Changed,
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
        let files = show_open_dialog(&self.window);
        for file in files {
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

fn millis_to_minutes(millis: u64) -> String {
    let mut seconds = millis / 1_000;
    let minutes = seconds / 60;
    seconds %= 60;
    format!("{}:{:02}", minutes, seconds)
}

fn new_icon(icon: &str) -> Image {
    Image::new_from_file(format!("./assets/{}.png", icon))
}

fn show_open_dialog(parent: &Window) -> Vec<PathBuf> {
    let mut folder = None;
    let dialog = FileChooserDialog::new(
        Some("Select a FLAC audio file"),
        Some(parent),

        FileChooserAction::SelectFolder,
    );

    // let flac_filter = FileFilter::new();
    // flac_filter.add_mime_type("audio/flac");
    // flac_filter.set_name("FLAC audio file");
    // dialog.add_filter(&flac_filter);

    // let m3u_filter = FileFilter::new();
    // m3u_filter.add_mime_type("audio/x-mpegurl");
    // m3u_filter.set_name("M3U playlist file");
    // dialog.add_filter(&m3u_filter);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Accept", gtk::ResponseType::Accept);
    let result = dialog.run();
    if result == GTK_RESPONSE_ACCEPT {
        folder = dialog.get_filename();
    }
    dialog.destroy();
    println!("Selected folder: {:?}", folder);

    let mut files = Vec::new();
    if let Some(f) = folder {

        let path = WalkDir::new(f.as_path());

        for entry in path {

            if let Ok(entry) = entry {

                let entry = entry.path();

                if let Some(extension) = entry.extension() {
                    if extension == OsStr::new("flac") {
                        files.push(entry.to_path_buf());
                    }
                }
            }
        }
    }

    files
}

fn show_save_dialog(parent: &Window) -> Option<PathBuf> {
    let mut file = None;
    let dialog = FileChooserDialog::new(
        Some("Choose a destination M3U playlist file"),
        Some(parent),
        FileChooserAction::Save,
    );
    let filter = FileFilter::new();
    // filter.add_mime_type("audio/x-mpegurl");
    // filter.set_name("M3U playlist file");
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
