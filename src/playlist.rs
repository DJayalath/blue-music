use gdk_pixbuf::{InterpType, Pixbuf, PixbufLoader, PixbufLoaderExt};
use gtk;
use gtk::{
    CellLayoutExt, CellRendererPixbuf, CellRendererText, GtkListStoreExt, GtkListStoreExtManual,
    ListStore, ToValue, TreeIter, TreeModelExt, TreeSelectionExt, TreeViewColumn,
    TreeViewColumnExt, TreeViewExt, WidgetExt,
};
use gtk::{StaticType, Type};
use m3u;
use metaflac::Tag;
use relm::Relm;
use relm::Widget;
use relm_derive::widget;
use rodio::Sink;
use std::cmp::max;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::sync::mpsc;
use lazy_static;

use crate::player::Player;

use self::Msg::*;
use self::Visibility::*;

#[derive(Clone)]
pub enum PlayerMsg {
    PlayerPlay,
    PlayerStop,
    PlayerTime(u64),
}

#[derive(PartialEq)]
enum Visibility {
    Invisible,
    Visible,
}

const INTERP_HYPER: InterpType = InterpType::Hyper;
const IMAGE_SIZE: i32 = 256;
const THUMBNAIL_SIZE: i32 = 64;

const THUMBNAIL_COLUMN: u32 = 0;
const TITLE_COLUMN: u32 = 1;
const ARTIST_COLUMN: u32 = 2;
const ALBUM_COLUMN: u32 = 3;
const GENRE_COLUMN: u32 = 4;
const YEAR_COLUMN: u32 = 5;
const TRACK_COLUMN: u32 = 6;
const PATH_COLUMN: u32 = 7;
const PIXBUF_COLUMN: u32 = 8;

#[derive(Msg)]
pub enum Msg {
    AddSong(PathBuf),
    LoadSong(PathBuf),
    NextSong,
    PauseSong,
    PlaySong,
    PreviousSong,
    RemoveSong,
    SaveSong(PathBuf),
    SongStarted(Option<Pixbuf>),
    StopSong,
}

pub struct Model {
    current_song: Option<String>,
    model: ListStore,
    relm: Relm<Playlist>,
    player: Player,
}

unsafe impl Send for Model {}

#[widget]
impl Widget for Playlist {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            current_song: None,
            model: ListStore::new(&[
                Pixbuf::static_type(),
                Type::String,
                Type::String,
                Type::String,
                Type::String,
                Type::String,
                Type::String,
                Type::String,
                Pixbuf::static_type(),
            ]),
            relm: relm.clone(),
            player: Player::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            AddSong(path) => self.add(&path),
            LoadSong(path) => self.load(&path),
            NextSong => self.next(),
            PauseSong => self.pause(),
            PlaySong => self.play(),
            PreviousSong => self.previous(),
            RemoveSong => self.remove_selection(),
            SaveSong(path) => self.save(&path),

            // Listened by Win
            SongStarted(_) => (),
            StopSong => self.stop(),
        }
    }

    fn init_view(&mut self) {
        self.create_columns();
    }

    view! {
        #[name="treeview"]
        gtk::TreeView {
            hexpand: true,
            model: &self.model.model,
            vexpand: true,
        }
    }
}

impl Playlist {

    fn pause(&mut self) {
        self.model.player.pause();
    }

    fn next(&mut self) {
        let selection = self.treeview.get_selection();
        let next_iter = if let Some((_, iter)) = selection.get_selected() {
            if !self.model.model.iter_next(&iter) {
                return;
            }
            Some(iter)
        } else {
            self.model.model.get_iter_first()
        };
        if let Some(ref iter) = next_iter {
            selection.select_iter(iter);
            self.play();
        }
    }

    fn previous(&mut self) {
        let selection = self.treeview.get_selection();
        let previous_iter = if let Some((_, iter)) = selection.get_selected() {
            if !self.model.model.iter_previous(&iter) {
                return;
            }
            Some(iter)
        } else {
            self.model
                .model
                .iter_nth_child(None, max(0, self.model.model.iter_n_children(None) - 1))
        };
        if let Some(ref iter) = previous_iter {
            selection.select_iter(iter);
            self.play();
        }
    }

    fn save(&self, path: &Path) {
        let mut file = File::create(path).unwrap();
        let mut writer = m3u::Writer::new(&mut file);

        let mut write_iter = |iter: &TreeIter| {
            let value = self.model.model.get_value(&iter, PATH_COLUMN as i32);
            let path = value.get::<String>().unwrap();
            writer.write_entry(&m3u::path_entry(path)).unwrap();
        };

        if let Some(iter) = self.model.model.get_iter_first() {
            write_iter(&iter);
            while self.model.model.iter_next(&iter) {
                write_iter(&iter);
            }
        }
    }

    fn stop(&mut self) {
        self.model.current_song = None;
        self.model.player.stop();
    }

    fn remove_selection(&self) {
        let selection = self.treeview.get_selection();
        if let Some((_, iter)) = selection.get_selected() {
            self.model.model.remove(&iter);
        }
    }

    fn load(&self, path: &Path) {
        let mut reader = m3u::Reader::open(path).unwrap();
        for entry in reader.entries() {
            if let Ok(m3u::Entry::Path(path)) = entry {
                self.add(&path);
            }
        }
    }

    fn play(&mut self) {
        if let Some(path) = self.selected_path() {

            self.model.player.play(path.clone());
            self.model.current_song = Some(path.clone().into());
            self.model.relm.stream().emit(SongStarted(self.pixbuf()));
        }
    }

    fn pixbuf(&self) -> Option<Pixbuf> {
        let selection = self.treeview.get_selection();
        if let Some((_, iter)) = selection.get_selected() {
            let value = self.model.model.get_value(&iter, PIXBUF_COLUMN as i32);
            return value.get::<Pixbuf>();
        }
        None
    }

    fn selected_path(&self) -> Option<String> {
        let selection = self.treeview.get_selection();
        if let Some((_, iter)) = selection.get_selected() {
            let value = self.model.model.get_value(&iter, PATH_COLUMN as i32);
            return value.get::<String>();
        }
        None
    }

    fn add(&self, path: &Path) {
        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        let row = self.model.model.append();

        if let Ok(tag) = Tag::read_from_path(path) {
            let title = match tag.get_vorbis("title") {
                Some(t) => t.get(0).unwrap(),
                None => filename,
            };

            let artist = match tag.get_vorbis("artist") {
                Some(t) => t.get(0).unwrap(),
                None => "Unknown",
            };

            let album = match tag.get_vorbis("album") {
                Some(t) => t.get(0).unwrap(),
                None => "Unknown",
            };

            let genre = match tag.get_vorbis("genre") {
                Some(t) => t.get(0).unwrap(),
                None => "Unknown",
            };

            let year = match tag.get_vorbis("year") {
                Some(t) => t.get(0).unwrap(),
                None => "Unknown",
            };

            let track = match tag.get_vorbis("tracknumber") {
                Some(t) => t.get(0).unwrap(),
                None => "Unknown",
            };

            let total_tracks = "??";

            let track_value = format!("{} / {}", track, total_tracks);

            self.set_pixbuf(&row, &tag);

            self.model
                .model
                .set_value(&row, TITLE_COLUMN, &title.to_value());

            self.model
                .model
                .set_value(&row, ARTIST_COLUMN, &artist.to_value());

            self.model
                .model
                .set_value(&row, ALBUM_COLUMN, &album.to_value());

            self.model
                .model
                .set_value(&row, GENRE_COLUMN, &genre.to_value());

            self.model
                .model
                .set_value(&row, YEAR_COLUMN, &year.to_value());

            self.model
                .model
                .set_value(&row, TRACK_COLUMN, &track_value.to_value());
        } else {
            self.model
                .model
                .set_value(&row, TITLE_COLUMN, &filename.to_value());
        }

        let path = path.to_str().unwrap_or_default();

        self.model
            .model
            .set_value(&row, PATH_COLUMN, &path.to_value());
    }

    fn add_pixbuf_column(&self, column: i32, visibility: Visibility) {
        let view_column = TreeViewColumn::new();
        if visibility == Visible {
            let cell = CellRendererPixbuf::new();
            view_column.pack_start(&cell, true);
            view_column.add_attribute(&cell, "pixbuf", column);
        }
    }

    fn add_text_column(&self, title: &str, column: i32) {
        let view_column = TreeViewColumn::new();
        view_column.set_title(title);
        let cell = CellRendererText::new();
        view_column.set_expand(true);
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", column);
        self.treeview.append_column(&view_column);
    }

    fn create_columns(&self) {
        self.add_pixbuf_column(THUMBNAIL_COLUMN as i32, Visible);
        self.add_text_column("Title", TITLE_COLUMN as i32);
        self.add_text_column("Artist", ARTIST_COLUMN as i32);
        self.add_text_column("Album", ALBUM_COLUMN as i32);
        self.add_text_column("Genre", GENRE_COLUMN as i32);
        self.add_text_column("Year", YEAR_COLUMN as i32);
        self.add_text_column("Track", TRACK_COLUMN as i32);
        self.add_pixbuf_column(PIXBUF_COLUMN as i32, Invisible);
    }

    fn set_pixbuf(&self, row: &TreeIter, tag: &Tag) {
        println!("{}", tag.pictures().len());
        if let Some(picture) = tag.pictures().get(0) {
            let pixbuf_loader = PixbufLoader::new();
            pixbuf_loader.set_size(IMAGE_SIZE, IMAGE_SIZE);
            pixbuf_loader.write(&picture.data).unwrap();
            if let Some(pixbuf) = pixbuf_loader.get_pixbuf() {
                let thumbnail = pixbuf
                    .scale_simple(THUMBNAIL_SIZE, THUMBNAIL_SIZE, INTERP_HYPER)
                    .unwrap();
                self.model
                    .model
                    .set_value(row, THUMBNAIL_COLUMN, &thumbnail.to_value());
                self.model
                    .model
                    .set_value(row, PIXBUF_COLUMN, &pixbuf.to_value());
            }
            pixbuf_loader.close().unwrap();
        }
    }
}
