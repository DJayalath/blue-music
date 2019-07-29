use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossbeam::queue::SegQueue;
use relm::Sender;

use crate::flac;
use crate::flac::FlacDecoder;
use crate::playlist::PlayerMsg::{
    self,
    PlayerPlay,
    PlayerStop,
    PlayerTime,
};
use self::Action::*;

#[cfg(target_os = "windows")]
use rodio::Source;
#[cfg(target_os = "linux")]
use pulse_simple::Playback;

const DEFAULT_RATE: u32 = 44100;

enum Action {
    Load(PathBuf),
    Skip(PathBuf, u32),
    Stop,
}

#[derive(Clone)]
struct EventLoop {
    condition_variable: Arc<(Mutex<bool>, Condvar)>,
    queue: Arc<SegQueue<Action>>,
    playing: Arc<Mutex<bool>>,
}

pub struct Player {
    event_loop: EventLoop,
    paused: Cell<bool>,
    tx: Sender<PlayerMsg>,
}

#[cfg(target_os = "windows")]
struct DecodingSystem {
    device: rodio::Device,
    sink: rodio::Sink,
}

#[cfg(target_os = "windows")]
impl DecodingSystem {
    pub fn new() -> Self {
        let device = rodio::default_output_device().unwrap();
        let sink = rodio::Sink::new(&device);
        DecodingSystem {device, sink}
    }
}

#[cfg(target_os = "linux")]
struct DecodingSystem {
    playback: Playback,
}

#[cfg(target_os = "linux")]
impl DecodingSystem {
    pub fn new() -> Self {
        let mut playback = Playback::new("Blue Music", "The free and open music player", None, DEFAULT_RATE);
        DecodingSystem {playback};
    }
}

impl Player {
    pub(crate) fn new(tx: Sender<PlayerMsg>) -> Self {

        let mut decoding_system = DecodingSystem::new();

        let condition_variable = Arc::new(
            (Mutex::new(false), Condvar::new())
        );

        let event_loop = EventLoop {
            condition_variable: condition_variable.clone(),
            queue: Arc::new(SegQueue::new()),
            playing: Arc::new(Mutex::new(false)),
        };

        {
            let mut tx = tx.clone();
            let event_loop = event_loop.clone();
            thread::spawn(move || {

                let block = || {
                    let (ref lock, ref condition_variable) = *condition_variable;
                    let mut started = lock.lock().unwrap();
                    *started = false;
                    while !*started {
                        started = condition_variable.wait(started).unwrap();
                    }
                };

                let mut source = None;

                loop {

                    if let Ok(action) = event_loop.queue.pop() {

                        match action {

                            Load(path) => {
                                
                                source = Some(FlacDecoder::new(&path));
                                let rate = source.unwrap().sample_rate();
                                println!("RATE: {}", rate);
                                source = Some(FlacDecoder::new(&path));
                                let channels = source.unwrap().num_channels;
                                println!("{}", channels);

                                #[cfg(target_os = "linux")]
                                {
                                    decoding_system.playback = Playback::new("Blue Music", "The free and open music player", None, rate);
                                }

                                #[cfg(target_os = "windows")]
                                {
                                    decoding_system.sink.stop();
                                    decoding_system.sink = rodio::Sink::new(&decoding_system.device);
                                }

                                send(&mut tx, PlayerPlay);
                                source = Some(FlacDecoder::new(&path));
                            },

                            Skip(path, time) => {
                                if let Some(ref mut source) = source {
                                    flac::skip_to(&path.as_path(), time, source);
                                }
                            }

                            Stop => {
                                source = None;
                            },
                        }
                    } else if *event_loop.playing.lock().unwrap() {

                        let mut written = false;
                        if let Some(ref mut source) = source {
                            if let Some(mut buf) = iter_to_buffer(source) {
                                if buf.len() > 0 {

                                    send(&mut tx, PlayerTime(source.current_time() as u64));

                                    #[cfg(target_os = "windows")]
                                    {
                                        let mut single_buf = Vec::with_capacity(buf.len() * 2);
                                        for sample in buf {
                                            single_buf.push(sample[0]);
                                            single_buf.push(sample[1]);
                                        }

                                        let sauce = rodio::buffer::SamplesBuffer::new(2, 44100, &single_buf[..]);

                                        decoding_system.sink.append(sauce);
                                    }

                                    #[cfg(target_os = "linux")]
                                    {
                                        decoding_system.playback.write(&buf[..]);
                                    }
                         
                                    written = true;
                                }
                            }
                        }

                        if !written {
                            send(&mut tx, PlayerStop);
                            *event_loop.playing.lock().unwrap() = false;
                            source = None;
                            block();
                        }
                    } else {
                        block();
                    }
                }

            });
        }

        Player {
            event_loop,
            paused: Cell::new(false),
            tx,
        }
    }

    pub fn compute_duration(path: &Path) -> u64 {
        flac::compute_duration(&path)
    }

    fn emit(&self, action: Action) {
        self.event_loop.queue.push(action);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }

    pub fn skip(&self, path: &Path, time: u32) {
        self.emit(Skip(path.to_path_buf(), time));
    }

    pub fn load(&self, path: &Path) {
        let pathbuf = path.to_path_buf();
        self.emit(Load(pathbuf));
        self.set_playing(true);
    }

    pub fn pause(&mut self) {
        self.paused.set(true);
        self.send(PlayerStop);
        self.set_playing(false);
    }

    pub fn resume(&mut self) {
        self.paused.set(false);
        self.send(PlayerPlay);
        self.set_playing(true);
    }

    fn set_playing(&self, playing: bool) {
        *self.event_loop.playing.lock().unwrap() = playing;
        let (ref lock, ref condition_variable) = *self.event_loop.condition_variable;
        let mut started = lock.lock().unwrap();
        *started = playing;
        if playing {
            condition_variable.notify_one();
        }
    }

    pub fn stop(&mut self) {
        self.paused.set(false);
        self.send(PlayerTime(0));
        self.send(PlayerStop);
        self.emit(Stop);
        self.set_playing(false);
    }

    fn send(&mut self, msg: PlayerMsg) {
        send(&mut self.tx, msg);
    }
}

fn iter_to_buffer(decoder: &mut FlacDecoder) -> Option<Vec<[i16; 2]>> {
    flac::next_sample(decoder)
}

fn send(tx: &mut Sender<PlayerMsg>, msg: PlayerMsg) {
    if let Ok(_) = tx.send(msg) {

    } else {
        eprintln!("Unable to send message to sender");
    }
}